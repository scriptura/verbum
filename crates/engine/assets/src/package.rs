// crates/engine/assets/src/package.rs

//! Paquet `VPKG` : enveloppe, extraction et chargement à entrée unique (V2-P).
//!
//! Layout, little-endian explicite, sans alignement ni bourrage :
//!
//! | Zone | Offset | Taille | Contenu |
//! |---|---:|---:|---|
//! | signature [`SIGNATURE`] | 0 | 4 | identité de l'enveloppe |
//! | version, `u16` ([`VERSION`]) | 4 | 2 | version de l'enveloppe |
//! | nombre d'artefacts `n'`, `u32` | 6 | 4 | lu avant la table : il en fixe la taille |
//! | table : début de l'artefact `i`, `u64` | 10 + 8·i | 8 | offset absolu depuis le début du paquet |
//! | manifeste `VMAN` v2 | `T = 10 + 8·n'` | variable | voir [`manifest_v2`](crate::manifest_v2) |
//! | artefacts | `s_i` | variable | contigus, jusqu'à la fin du paquet |
//!
//! La section de manifeste est `[T, s_0)` (le reste du paquet si `n' = 0`) ; l'artefact `i`
//! est `[s_i, s_{i+1})`, le dernier s'arrêtant à la fin du paquet. Les offsets sont
//! strictement croissants : une section d'artefact est donc jamais vide.
//!
//! L'extraction est une préparation technique avant le niveau 1, non un niveau : elle ne
//! lit aucun octet du manifeste et ne calcule pas sa longueur attendue (celle-ci relève du
//! lecteur v2). Elle ne valide que ce qu'il faut pour découper des tranches sûres, en une
//! seule boucle bornée par `n'`, sans allocation. Toute taille et toute conversion sont
//! vérifiées ; aucun `as`, aucune saturation.

use core::num::NonZeroU64;

use crate::format::{Asset, Format};
use crate::load::{LoadError, load_core};
use crate::manifest::SlotIndex;
use crate::manifest_v2::{ManifestV2Error, read_manifest_v2};

/// Signature de l'enveloppe, distincte de celle du manifeste (`VMAN`).
pub const SIGNATURE: [u8; 4] = *b"VPKG";

/// Version de l'enveloppe, indépendante de la version du manifeste qu'elle contient.
pub const VERSION: u16 = 1;

/// Longueur de l'en-tête d'enveloppe : signature (4) + version (2) + `n'` (4).
pub const HEADER_LEN: usize = 10;

/// Largeur d'un offset de la table : `u64`.
pub const OFFSET_WIDTH: usize = 8;

// Cohérence des constantes de layout, vérifiée à la compilation.
const _: () = {
    assert!(HEADER_LEN == SIGNATURE.len() + size_of::<u16>() + size_of::<u32>());
    assert!(OFFSET_WIDTH == size_of::<u64>());
};

/// Rejet d'un paquet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PackageError {
    /// L'en-tête, la table ou le premier octet du manifeste manquent physiquement.
    Truncated,
    /// Signature d'enveloppe différente de [`SIGNATURE`] (un manifeste v1 nu y compris).
    Signature,
    /// Version d'enveloppe différente de [`VERSION`].
    Version,
    /// Table incohérente : offsets non strictement croissants, borne hors du paquet,
    /// offset non représentable, section d'artefact vide, section de manifeste déclarée
    /// vide. Les écarts de longueur de la section de manifeste relèvent du lecteur
    /// (`Load(Manifest(..))`).
    Offsets,
    /// Le quantum `q_ns` du manifeste vaut zéro.
    ZeroQuantum,
    /// Validations existantes : manifeste (niveau 1), puis niveaux 2 à 4.
    Load(LoadError),
}

/// Enveloppe extraite : tranches du paquet, toutes de durée `'a`. Valeur `Copy`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Envelope<'a> {
    /// Paquet complet.
    package: &'a [u8],
    /// Table : exactement `8·n'` octets.
    table: &'a [u8],
    /// Section de manifeste : non vide, bornée par le premier offset.
    manifest: &'a [u8],
    /// `n'`, converti en `usize`.
    count: usize,
}

/// Offset `u64` little-endian d'un bloc de 8 octets, converti en `usize` ; `None` si le bloc
/// n'a pas 8 octets ou si la valeur n'est pas représentable.
fn read_offset(block: &[u8]) -> Option<usize> {
    let &[b0, b1, b2, b3, b4, b5, b6, b7] = block else {
        return None;
    };
    usize::try_from(u64::from_le_bytes([b0, b1, b2, b3, b4, b5, b6, b7])).ok()
}

impl<'a> Envelope<'a> {
    /// Extraction de l'enveloppe, dans l'ordre : en-tête complet, signature, version, `n'`,
    /// `T = 10 + 8·n'` (`T ≥ len` : `Truncated`, rejet en O(1) d'un `n'` hostile), table
    /// puis chaîne `T < s_0 < … < s_{n'−1} < len` (`Offsets`). Le contrôle physique
    /// précède toujours celui de la table.
    pub(crate) fn extract(package: &'a [u8]) -> Result<Self, PackageError> {
        let [s0, s1, s2, s3, v0, v1, n0, n1, n2, n3, ..] = package else {
            return Err(PackageError::Truncated);
        };
        if [*s0, *s1, *s2, *s3] != SIGNATURE {
            return Err(PackageError::Signature);
        }
        if u16::from_le_bytes([*v0, *v1]) != VERSION {
            return Err(PackageError::Version);
        }
        let count = usize::try_from(u32::from_le_bytes([*n0, *n1, *n2, *n3]))
            .map_err(|_| PackageError::Truncated)?;
        let end_of_table = count
            .checked_mul(OFFSET_WIDTH)
            .and_then(|table_len| HEADER_LEN.checked_add(table_len))
            .ok_or(PackageError::Truncated)?;
        if end_of_table >= package.len() {
            return Err(PackageError::Truncated);
        }
        let table = package
            .get(HEADER_LEN..end_of_table)
            .ok_or(PackageError::Truncated)?;

        // Chaîne `T < s_0 < … < s_{n'−1} < len` ; `first` est la fin de la section de
        // manifeste (la fin du paquet si la table est vide).
        let mut previous = end_of_table;
        let mut first = package.len();
        for (position, block) in table.as_chunks::<OFFSET_WIDTH>().0.iter().enumerate() {
            let start = read_offset(block).ok_or(PackageError::Offsets)?;
            if start <= previous {
                return Err(PackageError::Offsets);
            }
            if position == 0 {
                first = start;
            }
            previous = start;
        }
        if previous >= package.len() {
            return Err(PackageError::Offsets);
        }
        let manifest = package
            .get(end_of_table..first)
            .ok_or(PackageError::Offsets)?;
        Ok(Self {
            package,
            table,
            manifest,
            count,
        })
    }

    /// Nombre d'artefacts `n'` annoncé par l'enveloppe.
    pub(crate) fn count(&self) -> usize {
        self.count
    }

    /// Début de l'artefact `index` (offset de la table), `None` hors de la table.
    fn offset(&self, index: usize) -> Option<usize> {
        let at = index.checked_mul(OFFSET_WIDTH)?;
        read_offset(self.table.get(at..at.checked_add(OFFSET_WIDTH)?)?)
    }

    /// Octets de l'artefact `index`, `None` si `index ≥ n'`. La durée du résultat est celle
    /// du paquet, non celle de `&self`.
    pub(crate) fn artifact(&self, index: usize) -> Option<&'a [u8]> {
        if index >= self.count {
            return None;
        }
        let package = self.package;
        let start = self.offset(index)?;
        let next = index.checked_add(1)?;
        let end = if next < self.count {
            self.offset(next)?
        } else {
            package.len()
        };
        package.get(start..end)
    }
}

/// Conversion de l'erreur interne du lecteur v2 en erreur publique.
fn from_manifest_error(error: ManifestV2Error) -> PackageError {
    match error {
        ManifestV2Error::Manifest(error) => PackageError::Load(LoadError::Manifest(error)),
        ManifestV2Error::ZeroQuantum => PackageError::ZeroQuantum,
    }
}

/// Charge un paquet `VPKG` : extrait l'enveloppe, lit le manifeste v2, puis exécute les
/// niveaux 2 à 4 du noyau partagé avec [`load`](crate::load). Les assets sortent par
/// `deposit` ; le quantum `q_ns` sort par la valeur de retour.
///
/// `Ok(q)` n'existe qu'après les niveaux 1 à 4 **et** tous les dépôts : le quantum ne peut
/// être ni obtenu sans chargement ni dissocié des artefacts. Aucun dépôt n'a lieu si
/// l'extraction ou l'un des niveaux 1 à 3 échoue. Priorité des erreurs : enveloppe,
/// manifeste v2 (signature, version, quantum, longueur, doublons), liaisons, format de
/// slot, artefacts, références. Mêmes préconditions que [`load`](crate::load) sur
/// `slot_formats` et sur les slots visés ; aucune allocation. `Asset<'a>` emprunte le
/// paquet.
///
/// Les assets déposés empruntent le paquet : le libérer avant eux est refusé.
///
/// ```compile_fail,E0597
/// use verbum_assets::{Format, load_package};
///
/// let mut deposited = None;
/// {
///     let package = [
///         0x56, 0x50, 0x4B, 0x47, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, // VPKG, v1, n' = 1
///         0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // s_0 = 52
///         0x56, 0x4D, 0x41, 0x4E, 0x02, 0x00, // VMAN, v2
///         0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // q_ns = 10
///         0x01, 0x00, 0x00, 0x00, // n = 1
///         0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // id
///         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // artefact 0, slot 0
///         0x56, 0x46, 0x4E, 0x54, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, // VFNT, charge 4
///         0x00, 0x00, 0x0B, 0x0A, // 0 glyphe, hauteur de ligne
///     ];
///     load_package(&package, &[Format::Font], |_, asset| deposited = Some(asset)).unwrap();
/// }
/// let _ = deposited;
/// ```
///
/// Jumeau valide : le paquet vit aussi longtemps que les assets déposés.
///
/// ```
/// use verbum_assets::{Asset, Format, load_package};
///
/// let package = [
///     0x56, 0x50, 0x4B, 0x47, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, // VPKG, v1, n' = 1
///     0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // s_0 = 52
///     0x56, 0x4D, 0x41, 0x4E, 0x02, 0x00, // VMAN, v2
///     0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // q_ns = 10
///     0x01, 0x00, 0x00, 0x00, // n = 1
///     0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // id
///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // artefact 0, slot 0
///     0x56, 0x46, 0x4E, 0x54, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, // VFNT, charge 4
///     0x00, 0x00, 0x0B, 0x0A, // 0 glyphe, hauteur de ligne
/// ];
/// let mut deposited = None;
/// let quantum = load_package(&package, &[Format::Font], |_, asset| deposited = Some(asset))
///     .expect("paquet conforme");
/// assert_eq!(quantum.get(), 10);
/// let Some(Asset::Font(font)) = deposited else {
///     panic!("une police était attendue");
/// };
/// assert_eq!(font.line_height(), 0x0A0B);
/// ```
///
/// Le noyau, l'extraction et le lecteur v2 ne sont pas accessibles de l'extérieur.
///
/// ```compile_fail,E0603
/// use verbum_assets::package::Envelope;
/// ```
///
/// ```compile_fail,E0603
/// use verbum_assets::manifest_v2::read_manifest_v2;
/// ```
///
/// ```compile_fail,E0603
/// use verbum_assets::load::load_core;
/// ```
pub fn load_package<'a>(
    package: &'a [u8],
    slot_formats: &[Format],
    deposit: impl FnMut(SlotIndex, Asset<'a>),
) -> Result<NonZeroU64, PackageError> {
    let envelope = Envelope::extract(package)?;
    let (quantum, manifest) = read_manifest_v2(envelope.manifest).map_err(from_manifest_error)?;
    load_core(
        &manifest,
        envelope.count(),
        |index| envelope.artifact(index),
        slot_formats,
        deposit,
    )
    .map_err(PackageError::Load)?;
    Ok(quantum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::{ReferenceError, load};
    use crate::manifest::ManifestError;
    use crate::sprite::{ReadError, Sprite};

    const Q: u64 = 0x0102_0304_0506_0708;
    const ID_0: u64 = 0x1122_3344_5566_7788;
    const ID_1: u64 = 0x99AA_BBCC_DDEE_FF00;
    const MAP_ID: u64 = 0x0A0B_0C0D_0E0F_1011;
    const UNKNOWN_ID: u64 = 0x1111_1111_1111_1111;

    /// Paquet de l'exemple chiffré de V2-C § 3.4, écrit à la main, indépendant des
    /// constantes : `n' = n = 2`, `T = 26`, manifeste de 50 octets, `s_0 = 76`, `s_1 = 90`,
    /// `len = 104`, `q_ns = 10`. Artefact 0 : sprite 32×16 (slot 1) ; artefact 1 : sprite
    /// 96×48 (slot 0).
    const PACKAGE: [u8; 104] = [
        0x56, 0x50, 0x4B, 0x47, // VPKG
        0x01, 0x00, // version d'enveloppe 1
        0x02, 0x00, 0x00, 0x00, // n' = 2
        0x4C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // s_0 = 76
        0x5A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // s_1 = 90
        0x56, 0x4D, 0x41, 0x4E, // VMAN
        0x02, 0x00, // version du manifeste 2
        0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // q_ns = 10
        0x02, 0x00, 0x00, 0x00, // n = 2
        0x8C, 0xEC, 0x01, 0x86, 0x4C, 0xDC, 0x63, 0xAF, // id 0
        0x00, 0x00, 0x00, 0x00, // artifact_index 0
        0x01, 0x00, 0x00, 0x00, // slot_index 1
        0xE8, 0x67, 0x39, 0xF7, 0x71, 0x41, 0x94, 0x85, // id 1
        0x01, 0x00, 0x00, 0x00, // artifact_index 1
        0x00, 0x00, 0x00, 0x00, // slot_index 0
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x20, 0x00, 0x10,
        0x00, // 32×16
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x60, 0x00, 0x30,
        0x00, // 96×48
    ];

    /// Paquet vide : `n' = n = 0`, 28 octets, `q_ns` = `Q`.
    const EMPTY_PACKAGE: [u8; 28] = [
        0x56, 0x50, 0x4B, 0x47, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // enveloppe, n' = 0
        0x56, 0x4D, 0x41, 0x4E, 0x02, 0x00, // VMAN, version 2
        0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // q_ns
        0x00, 0x00, 0x00, 0x00, // n = 0
    ];

    const SPRITE_32X16: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x20, 0x00, 0x10, 0x00,
    ];
    const SPRITE_96X48: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x60, 0x00, 0x30, 0x00,
    ];
    const TILESET_16X8: [u8; 18] = [
        0x56, 0x54, 0x4C, 0x53, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x08, 0x00, 0x04,
        0x00, 0x0C, 0x00,
    ];
    /// Carte 3×2 référençant `ID_0` (identifiant du tileset, octets 10 à 18).
    const MAP_BYTES: [u8; 34] = [
        0x56, 0x4D, 0x41, 0x50, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, // préfixe, charge 24
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // identifiant du tileset
        0x03, 0x00, 0x02, 0x00, // 3 × 2
        0x01, 0x00, 0x03, 0x02, 0x05, 0x04, 0x07, 0x06, 0x09, 0x08, 0x0B, 0x0A,
    ];

    const SPRITES: [Format; 2] = [Format::Sprite; 2];

    /// Dépôts de test : compteur d'appels et sprites reçus par slot.
    #[derive(Debug, Default, PartialEq, Eq)]
    struct Deposits {
        calls: usize,
        slots: [Option<Sprite>; 4],
    }

    impl Deposits {
        fn record(&mut self, slot: SlotIndex, asset: Asset<'_>) {
            self.calls += 1;
            if let Asset::Sprite(sprite) = asset {
                self.slots[slot.get() as usize] = Some(sprite);
            }
        }
    }

    fn sprite(width: u16, height: u16) -> Sprite {
        Sprite { width, height }
    }

    fn quantum(value: u64) -> NonZeroU64 {
        NonZeroU64::new(value).expect("quantum non nul")
    }

    fn manifest_error(error: ManifestError) -> PackageError {
        PackageError::Load(LoadError::Manifest(error))
    }

    fn run(bytes: &[u8], formats: &[Format]) -> (Result<NonZeroU64, PackageError>, Deposits) {
        let mut deposits = Deposits::default();
        let result = load_package(bytes, formats, |slot, asset| deposits.record(slot, asset));
        (result, deposits)
    }

    /// Résultat attendu, sans aucun dépôt.
    fn assert_rejected(bytes: &[u8], expected: PackageError, context: impl core::fmt::Display) {
        let (result, deposits) = run(bytes, &SPRITES);
        assert_eq!(result, Err(expected), "{context}");
        assert_eq!(deposits.calls, 0, "{context} : dépôt inattendu");
    }

    fn assert_artifact<T: Copy + core::fmt::Debug>(
        result: Result<T, PackageError>,
        index: u32,
        error: ReadError,
    ) {
        let Err(PackageError::Load(LoadError::Artifact {
            index: found,
            error: found_error,
        })) = result
        else {
            panic!("attendu : erreur d'artefact, obtenu {result:?}");
        };
        assert_eq!(found.get(), index);
        assert_eq!(found_error, error);
    }

    /// Manifeste v2 : `q`, compteur déclaré `declared`, entrées `(id, artefact, slot)`.
    fn manifest_v2_bytes<'b>(
        buf: &'b mut [u8; 128],
        q: u64,
        declared: u32,
        entries: &[(u64, u32, u32)],
    ) -> &'b [u8] {
        buf[..4].copy_from_slice(b"VMAN");
        buf[4..6].copy_from_slice(&2u16.to_le_bytes());
        buf[6..14].copy_from_slice(&q.to_le_bytes());
        buf[14..18].copy_from_slice(&declared.to_le_bytes());
        for (i, (id, artifact, slot)) in entries.iter().enumerate() {
            let at = 18 + 16 * i;
            buf[at..at + 8].copy_from_slice(&id.to_le_bytes());
            buf[at + 8..at + 12].copy_from_slice(&artifact.to_le_bytes());
            buf[at + 12..at + 16].copy_from_slice(&slot.to_le_bytes());
        }
        &buf[..18 + 16 * entries.len()]
    }

    /// Manifeste v1 des mêmes entrées.
    fn manifest_v1_bytes<'b>(buf: &'b mut [u8; 128], entries: &[(u64, u32, u32)]) -> &'b [u8] {
        buf[..4].copy_from_slice(b"VMAN");
        buf[4..6].copy_from_slice(&1u16.to_le_bytes());
        buf[6..10].copy_from_slice(&(entries.len() as u32).to_le_bytes());
        for (i, (id, artifact, slot)) in entries.iter().enumerate() {
            let at = 10 + 16 * i;
            buf[at..at + 8].copy_from_slice(&id.to_le_bytes());
            buf[at + 8..at + 12].copy_from_slice(&artifact.to_le_bytes());
            buf[at + 12..at + 16].copy_from_slice(&slot.to_le_bytes());
        }
        &buf[..10 + 16 * entries.len()]
    }

    /// Assemble une enveloppe valide : `n' = artifacts.len()`, offsets calculés.
    fn assemble<'b>(buf: &'b mut [u8; 256], manifest: &[u8], artifacts: &[&[u8]]) -> &'b [u8] {
        let table = 10 + 8 * artifacts.len();
        buf[..4].copy_from_slice(b"VPKG");
        buf[4..6].copy_from_slice(&1u16.to_le_bytes());
        buf[6..10].copy_from_slice(&(artifacts.len() as u32).to_le_bytes());
        let mut offset = table + manifest.len();
        for (i, artifact) in artifacts.iter().enumerate() {
            buf[10 + 8 * i..18 + 8 * i].copy_from_slice(&(offset as u64).to_le_bytes());
            offset += artifact.len();
        }
        buf[table..table + manifest.len()].copy_from_slice(manifest);
        let mut at = table + manifest.len();
        for artifact in artifacts {
            buf[at..at + artifact.len()].copy_from_slice(artifact);
            at += artifact.len();
        }
        &buf[..at]
    }

    /// Paquet complet : manifeste v2 de `q` et des `entries`, puis `artifacts`.
    fn package_of<'b>(
        buf: &'b mut [u8; 256],
        q: u64,
        entries: &[(u64, u32, u32)],
        artifacts: &[&[u8]],
    ) -> &'b [u8] {
        let mut scratch = [0u8; 128];
        let manifest = manifest_v2_bytes(&mut scratch, q, entries.len() as u32, entries);
        assemble(buf, manifest, artifacts)
    }

    const TWO: [(u64, u32, u32); 2] = [(ID_0, 0, 1), (ID_1, 1, 0)];

    #[test]
    fn pk1_loads_the_chiffred_example_and_returns_the_quantum() {
        let (result, deposits) = run(&PACKAGE, &SPRITES);
        assert_eq!(result, Ok(quantum(10)));
        assert_eq!(deposits.calls, 2);
        assert_eq!(deposits.slots[1], Some(sprite(32, 16)));
        assert_eq!(deposits.slots[0], Some(sprite(96, 48)));
        assert_eq!(deposits.slots[2], None);

        // `q_ns` est lu en little-endian.
        let mut bytes = PACKAGE;
        bytes[32..40].copy_from_slice(&Q.to_le_bytes());
        assert_eq!(run(&bytes, &SPRITES).0, Ok(quantum(Q)));
    }

    #[test]
    fn pk2_an_empty_package_loads_without_deposit_and_short_ones_are_truncated() {
        let (result, deposits) = run(&EMPTY_PACKAGE, &SPRITES);
        assert_eq!(result, Ok(quantum(Q)));
        assert_eq!(deposits.calls, 0);

        // `len ≤ 10` : aucun octet de manifeste, absence physique.
        for len in 0..=10 {
            assert_rejected(&EMPTY_PACKAGE[..len], PackageError::Truncated, len);
        }
        // Section de manifeste non vide mais insuffisante : troncature du lecteur.
        for len in 11..EMPTY_PACKAGE.len() {
            assert_rejected(
                &EMPTY_PACKAGE[..len],
                manifest_error(ManifestError::Truncated),
                len,
            );
        }
    }

    #[test]
    fn pk3_every_prefix_of_the_chiffred_example_is_classified_exactly() {
        for len in 0..=26 {
            assert_rejected(&PACKAGE[..len], PackageError::Truncated, len);
        }
        for len in 27..=90 {
            assert_rejected(&PACKAGE[..len], PackageError::Offsets, len);
        }
        for len in 91..=103 {
            let (result, deposits) = run(&PACKAGE[..len], &SPRITES);
            assert_artifact(result, 1, ReadError::Truncated);
            assert_eq!(deposits.calls, 0, "préfixe de {len} octets");
        }
        assert_eq!(run(&PACKAGE, &SPRITES).0, Ok(quantum(10)));
    }

    #[test]
    fn pk4_a_bare_v1_manifest_and_altered_envelope_identity_are_rejected() {
        let mut scratch = [0u8; 128];
        let v1 = manifest_v1_bytes(&mut scratch, &TWO);
        assert_rejected(v1, PackageError::Signature, "manifeste v1 nu");

        for index in 0..4 {
            let mut bytes = PACKAGE;
            bytes[index] ^= 0xFF;
            assert_rejected(&bytes, PackageError::Signature, "octet de signature");
        }
        for version in [[0x00, 0x00], [0x02, 0x00], [0x00, 0x01], [0xFF, 0xFF]] {
            let mut bytes = PACKAGE;
            bytes[4..6].copy_from_slice(&version);
            assert_rejected(&bytes, PackageError::Version, "version d'enveloppe");
        }
    }

    fn with_offsets(first: u64, second: u64) -> [u8; 104] {
        let mut bytes = PACKAGE;
        bytes[10..18].copy_from_slice(&first.to_le_bytes());
        bytes[18..26].copy_from_slice(&second.to_le_bytes());
        bytes
    }

    #[test]
    fn pk5_inconsistent_offsets_are_rejected_and_hostile_counts_cost_nothing() {
        for (first, second) in [
            (90, 90),
            (76, 76),
            (90, 76),
            (26, 90),
            (20, 90),
            (0, 90),
            (76, 104),
            (76, 105),
            (76, u64::MAX),
            (u64::MAX, u64::MAX),
        ] {
            assert_rejected(
                &with_offsets(first, second),
                PackageError::Offsets,
                "offsets incohérents",
            );
        }

        // `n'` hostile : `T ≥ len`, rejet immédiat sans lire la table.
        for count in [u32::MAX, 0x2000_0000, 12] {
            let mut bytes = PACKAGE;
            bytes[6..10].copy_from_slice(&count.to_le_bytes());
            assert_rejected(&bytes, PackageError::Truncated, "n' hostile");
        }
        // `n' = 11` : `T = 98 < 104`, la table lue pointe dans la table.
        let mut bytes = PACKAGE;
        bytes[6..10].copy_from_slice(&11u32.to_le_bytes());
        assert_rejected(&bytes, PackageError::Offsets, "n' = 11");
    }

    #[test]
    fn pk6_an_envelope_count_different_from_the_manifest_count_is_a_binding_error() {
        let mismatch = PackageError::Load(LoadError::Binding(
            crate::manifest::BindingError::EntryCountMismatch,
        ));
        // `n' = 2`, `n = 1` : section de longueur exacte pour `n`.
        let mut buf = [0u8; 256];
        let bytes = package_of(
            &mut buf,
            5,
            &[(ID_0, 0, 1)],
            &[&SPRITE_32X16, &SPRITE_96X48],
        );
        assert_rejected(bytes, mismatch, "n' = 2, n = 1");
        // `n' = 1`, `n = 2`.
        let mut buf = [0u8; 256];
        let bytes = package_of(&mut buf, 5, &TWO, &[&SPRITE_32X16]);
        assert_rejected(bytes, mismatch, "n' = 1, n = 2");
    }

    #[test]
    fn pk7_a_v1_manifest_or_a_zero_quantum_inside_a_valid_envelope_is_rejected() {
        let version = manifest_error(ManifestError::Version);
        let mut scratch = [0u8; 128];
        let empty_v1 = manifest_v1_bytes(&mut scratch, &[]);
        let mut buf = [0u8; 256];
        assert_rejected(assemble(&mut buf, empty_v1, &[]), version, "v1 vide");

        let mut scratch = [0u8; 128];
        let v1 = manifest_v1_bytes(&mut scratch, &TWO);
        let mut buf = [0u8; 256];
        let bytes = assemble(&mut buf, v1, &[&SPRITE_32X16, &SPRITE_96X48]);
        assert_rejected(bytes, version, "v1 à deux entrées");

        let mut buf = [0u8; 256];
        let bytes = package_of(&mut buf, 0, &TWO, &[&SPRITE_32X16, &SPRITE_96X48]);
        assert_rejected(bytes, PackageError::ZeroQuantum, "q nul");
    }

    /// Même lot par `load` (manifeste v1) et par `load_package` : mêmes dépôts, même
    /// résultat (enveloppé).
    fn differential(
        entries: &[(u64, u32, u32)],
        artifacts: &[&[u8]],
        formats: &[Format],
    ) -> Result<(), PackageError> {
        let mut scratch = [0u8; 128];
        let v1 = manifest_v1_bytes(&mut scratch, entries);
        let mut old = Deposits::default();
        let old_result = load(v1, artifacts, formats, |slot, asset| {
            old.record(slot, asset)
        });

        let mut buf = [0u8; 256];
        let package = package_of(&mut buf, 7, entries, artifacts);
        let (new_result, new) = run(package, formats);

        assert_eq!(new, old, "dépôts divergents");
        assert_eq!(
            new_result.map(|_| ()),
            old_result.map_err(PackageError::Load),
            "résultats divergents"
        );
        new_result.map(|_| ())
    }

    #[test]
    fn pk8_the_package_path_matches_the_historical_path_at_levels_2_to_4() {
        assert_eq!(
            differential(&TWO, &[&SPRITE_32X16, &SPRITE_96X48], &SPRITES),
            Ok(())
        );

        let result = differential(&TWO, &[&SPRITE_32X16, &SPRITE_96X48[..13]], &SPRITES);
        assert_artifact(result, 1, ReadError::Truncated);

        let result = differential(&TWO, &[&TILESET_16X8, &SPRITE_96X48], &SPRITES);
        assert_eq!(
            result,
            Err(PackageError::Load(LoadError::SlotFormat {
                slot: SlotIndex::new(1)
            }))
        );

        let maps = [(MAP_ID, 0, 0), (ID_0, 1, 1)];
        let formats = [Format::Map, Format::Tileset];
        assert_eq!(
            differential(&maps, &[&MAP_BYTES, &TILESET_16X8], &formats),
            Ok(())
        );
        let mut dangling = MAP_BYTES;
        dangling[10..18].copy_from_slice(&UNKNOWN_ID.to_le_bytes());
        let result = differential(&maps, &[&dangling, &TILESET_16X8], &formats);
        let Err(PackageError::Load(LoadError::Reference { index, error })) = result else {
            panic!("attendu : erreur de référence, obtenu {result:?}");
        };
        assert_eq!(index.get(), 0);
        assert_eq!(error, ReferenceError::Unresolved);
    }

    #[test]
    fn pk9_a_failure_at_any_level_leaves_every_slot_untouched() {
        let mut buf = [0u8; 256];

        // Enveloppe.
        assert_rejected(&with_offsets(90, 90), PackageError::Offsets, "enveloppe");
        // Niveau 1 : quantum nul.
        let bytes = package_of(&mut buf, 0, &TWO, &[&SPRITE_32X16, &SPRITE_96X48]);
        assert_rejected(bytes, PackageError::ZeroQuantum, "niveau 1");
        // Niveau 2 : liaisons.
        let mut buf = [0u8; 256];
        let bytes = package_of(&mut buf, 5, &TWO, &[&SPRITE_32X16]);
        let (result, deposits) = run(bytes, &SPRITES);
        assert!(result.is_err());
        assert_eq!(deposits.calls, 0, "liaisons");
        // Niveau 2 : format de slot.
        let mut buf = [0u8; 256];
        let bytes = package_of(&mut buf, 5, &TWO, &[&TILESET_16X8, &SPRITE_96X48]);
        let (result, deposits) = run(bytes, &SPRITES);
        assert!(result.is_err());
        assert_eq!(deposits.calls, 0, "format de slot");
        // Niveau 3 : artefact invalide après un artefact valide.
        let mut buf = [0u8; 256];
        let bytes = package_of(&mut buf, 5, &TWO, &[&SPRITE_32X16, &SPRITE_96X48[..13]]);
        let (result, deposits) = run(bytes, &SPRITES);
        assert_artifact(result, 1, ReadError::Truncated);
        assert_eq!(deposits.calls, 0, "artefact");
        // Niveau 3 : référence non résolue.
        let mut dangling = MAP_BYTES;
        dangling[10..18].copy_from_slice(&UNKNOWN_ID.to_le_bytes());
        let mut buf = [0u8; 256];
        let bytes = package_of(
            &mut buf,
            5,
            &[(MAP_ID, 0, 0), (ID_0, 1, 1)],
            &[&dangling, &TILESET_16X8],
        );
        let (result, deposits) = run(bytes, &[Format::Map, Format::Tileset]);
        assert!(result.is_err());
        assert_eq!(deposits.calls, 0, "référence");
    }

    #[test]
    fn pk10_the_historical_path_rejects_a_v2_manifest_and_a_package() {
        let artifacts: [&[u8]; 2] = [&SPRITE_32X16, &SPRITE_96X48];

        let mut deposits = Deposits::default();
        let result = load(&PACKAGE[26..76], &artifacts, &SPRITES, |slot, asset| {
            deposits.record(slot, asset)
        });
        assert_eq!(result, Err(LoadError::Manifest(ManifestError::Version)));
        assert_eq!(deposits.calls, 0);

        let result = load(&PACKAGE, &artifacts, &SPRITES, |slot, asset| {
            deposits.record(slot, asset)
        });
        assert_eq!(result, Err(LoadError::Manifest(ManifestError::Signature)));
        assert_eq!(deposits.calls, 0);
    }

    #[test]
    fn pk11_a_short_artifact_section_reaches_level_3_and_an_empty_one_is_an_offsets_error() {
        // Section non vide, correctement délimitée, trop courte.
        for len in 1..=13 {
            let mut buf = [0u8; 256];
            let bytes = package_of(&mut buf, 5, &TWO, &[&SPRITE_32X16, &SPRITE_96X48[..len]]);
            let (result, deposits) = run(bytes, &SPRITES);
            assert_artifact(result, 1, ReadError::Truncated);
            assert_eq!(deposits.calls, 0, "préfixe de {len} octets");
        }
        // Section de longueur nulle : dernier offset égal à `len`, puis offsets égaux.
        let mut buf = [0u8; 256];
        let bytes = package_of(&mut buf, 5, &TWO, &[&SPRITE_32X16, &[]]);
        assert_rejected(bytes, PackageError::Offsets, "dernier artefact vide");
        let mut buf = [0u8; 256];
        let bytes = package_of(&mut buf, 5, &TWO, &[&[], &SPRITE_96X48]);
        assert_rejected(bytes, PackageError::Offsets, "artefact 0 vide");
    }

    #[test]
    fn pk12_the_envelope_constants_keep_their_literal_values() {
        assert_eq!(SIGNATURE, [0x56, 0x50, 0x4B, 0x47]);
        assert_eq!(VERSION, 1);
        assert_eq!(HEADER_LEN, 10);
        assert_eq!(OFFSET_WIDTH, 8);
        assert_ne!(SIGNATURE, crate::manifest::MANIFEST_SIGNATURE);
        assert_ne!(SIGNATURE, crate::manifest_v2::SIGNATURE);
    }

    fn check_artifacts(artifacts: &[&[u8]]) {
        // Le contenu du manifeste est sans effet sur l'extraction.
        let dummy = [0xAA; 20];
        let mut buf = [0u8; 256];
        let package = assemble(&mut buf, &dummy, artifacts);
        let envelope = Envelope::extract(package).expect("enveloppe conforme");
        assert_eq!(envelope.count(), artifacts.len());
        assert_eq!(envelope.manifest, &dummy[..]);
        for (index, artifact) in artifacts.iter().enumerate() {
            assert_eq!(
                envelope.artifact(index),
                Some(*artifact),
                "artefact {index}"
            );
        }
        assert_eq!(envelope.artifact(artifacts.len()), None);
        assert_eq!(envelope.artifact(usize::MAX), None);
    }

    #[test]
    fn pk13_artifact_access_returns_exact_sections_and_none_out_of_range() {
        check_artifacts(&[]);
        check_artifacts(&[&[1, 2, 3]]);
        check_artifacts(&[&[1, 2, 3], &[4, 5]]);
        check_artifacts(&[&[1, 2, 3], &[4, 5], &[6; 7]]);
    }

    #[test]
    fn pk14_the_twelve_manifest_section_edge_cases_are_classified() {
        let truncated = manifest_error(ManifestError::Truncated);

        // 1 : aucun octet de manifeste (`len ≤ T`).
        for len in [0, 10, 25, 26] {
            assert_rejected(&PACKAGE[..len], PackageError::Truncated, "cas 1");
        }
        // 2, 3, 4 : table incohérente.
        assert_rejected(&with_offsets(26, 90), PackageError::Offsets, "cas 2");
        assert_rejected(&with_offsets(20, 90), PackageError::Offsets, "cas 3");
        assert_rejected(&with_offsets(90, 90), PackageError::Offsets, "cas 4");
        // 5, 6, 8 : section insuffisante pour le lecteur ; 7 : q non nul, `Truncated`.
        for first in 27..=75 {
            assert_rejected(&with_offsets(first, 90), truncated, "cas 5 à 8");
        }
        // 7 : q nul, le lecteur le lit dès 14 octets.
        for first in 40..=43 {
            let mut bytes = with_offsets(first, 90);
            bytes[32..40].fill(0);
            assert_rejected(&bytes, PackageError::ZeroQuantum, "cas 7, q nul");
        }
        for first in 32..=39 {
            let mut bytes = with_offsets(first, 90);
            bytes[32..40].fill(0);
            assert_rejected(&bytes, truncated, "q nul mais incomplet");
        }
        // 9 : section exacte.
        assert_eq!(run(&with_offsets(76, 90), &SPRITES).0, Ok(quantum(10)));
        // 10 : section trop longue, rejetée par le lecteur (option X).
        for first in 77..=89 {
            assert_rejected(
                &with_offsets(first, 90),
                manifest_error(ManifestError::TrailingBytes),
                "cas 10",
            );
        }
        // 11 : `n' = 0`, manifeste valide suivi d'octets.
        let mut trailing = [0u8; 29];
        trailing[..28].copy_from_slice(&EMPTY_PACKAGE);
        assert_rejected(
            &trailing,
            manifest_error(ManifestError::TrailingBytes),
            "cas 11",
        );
        // 12 : `n' = 0`, `len ≤ 10`.
        for len in 0..=10 {
            assert_rejected(&EMPTY_PACKAGE[..len], PackageError::Truncated, "cas 12");
        }
    }
}
