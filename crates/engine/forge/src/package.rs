// crates/engine/forge/src/package.rs

//! Production d'un paquet `VPKG` : enveloppe, manifeste `VMAN` v2 et artefacts (V2-F).
//!
//! Chemin additif (V-F1) : `compile_assets` est appelé sans modification, puis le
//! résultat est assemblé. Le manifeste v2 reprend du manifeste v1 le compteur et les
//! entrées, octet pour octet ; seul le quantum `q_ns` s'y ajoute.
//!
//! Le layout est calculé par une fonction pure ([`package_layout`]) qui précède toute
//! allocation du paquet. Toute taille et tout offset sont vérifiés (`checked_*`,
//! `try_from`) : aucun `as`, aucune saturation. L'échec d'allocation dans les limites
//! représentables est hors contrat.
//!
//! Cibles 32 bits : non testées. Les calculs sont corrects par construction
//! (arithmétique vérifiée, aucune conversion non contrôlée), ce qui est un argument de
//! conception et non un résultat de test.

use verbum_assets::manifest_v2;
use verbum_assets::package as envelope;
use verbum_assets::{MANIFEST_ENTRY_LEN, MANIFEST_HEADER_LEN};

use crate::{AssetSource, ForgeError, compile_assets};

/// Rejet de la production d'un paquet.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompilePackageError {
    /// Erreur de `compile_assets`, ou `q` nul (`ForgeError::ZeroValue("q")`).
    Forge(ForgeError),
    /// Taille ou offset de paquet non représentable, ou capacité supérieure à
    /// `isize::MAX`.
    PackageTooLarge,
    /// Violation d'un invariant structurel de la sortie de `compile_assets`.
    InconsistentBatch,
}

/// Paquet `VPKG` complet : enveloppe, manifeste `VMAN` v2 et artefacts.
///
/// Seul producteur public de paquet. Le champ est privé et aucun constructeur n'est
/// public : un paquet ne s'obtient que par [`compile_package`]. Aucun accesseur ne rend
/// le quantum, le manifeste ou les artefacts séparément.
///
/// Le champ est inaccessible (pf8) :
///
/// ```compile_fail,E0451
/// let _ = verbum_forge::CompiledPackage { bytes: Vec::new() };
/// ```
///
/// Aucun constructeur public (pf8) :
///
/// ```compile_fail,E0599
/// let _ = verbum_forge::CompiledPackage::new(Vec::new());
/// ```
///
/// Aucun accesseur séparé du manifeste (pf8) :
///
/// ```compile_fail,E0599
/// fn f(package: &verbum_forge::CompiledPackage) {
///     let _ = package.manifest();
/// }
/// ```
///
/// Aucun accesseur séparé des artefacts (pf8) :
///
/// ```compile_fail,E0599
/// fn f(package: &verbum_forge::CompiledPackage) {
///     let _ = package.artifacts();
/// }
/// ```
///
/// Aucun accesseur du quantum (pf8) :
///
/// ```compile_fail,E0599
/// fn f(package: &verbum_forge::CompiledPackage) {
///     let _ = package.quantum_ns();
/// }
/// ```
///
/// La fonction de layout est privée :
///
/// ```compile_fail,E0603
/// let _ = verbum_forge::package::package_layout;
/// ```
///
/// Jumeau valide : l'unique accès aux octets est `as_bytes`.
///
/// ```
/// fn f(package: &verbum_forge::CompiledPackage) -> &[u8] {
///     package.as_bytes()
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledPackage {
    bytes: Vec<u8>,
}

impl CompiledPackage {
    /// Octets du paquet complet : enveloppe, manifeste v2, artefacts, et rien d'autre.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Plafond de capacité d'un tampon : `isize::MAX`, exprimé sans `as`.
const CAPACITY_LIMIT: usize = isize::MAX.unsigned_abs();

// L'en-tête de manifeste v1 est destructuré par motif (10 octets) dans `split_manifest`.
const _: () = assert!(MANIFEST_HEADER_LEN == 10);

/// Layout calculé d'un paquet : valeur de taille fixe, sans collection. Les offsets `s_i`
/// n'y figurent pas : ils sont recalculés pendant l'assemblage par [`write_offsets`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Layout {
    /// `n'` : nombre d'artefacts, identique au compteur du manifeste.
    count: u32,
    /// `T = 10 + 8·n'` : début du manifeste v2, fin de la table.
    table_end: usize,
    /// `s_0 = T + 18 + 16·n` : début du premier artefact (fin du paquet si `n' = 0`).
    first_artifact: usize,
    /// Longueur totale du paquet.
    total: usize,
}

/// Layout du paquet : fonction pure, sans aucune allocation.
///
/// `manifest_len` et `declared_count` décrivent le manifeste v1 produit par
/// `compile_assets` ; `artifact_lens` donne la longueur de chaque artefact. Ordre des
/// contrôles : P1 (`n'` en `u32`), P7 (compte, longueur du manifeste, artefact non
/// vide), P2 (`T`), P3 (`s_0`), P6 et P4 (offsets, cumul), P5 (plafond `isize::MAX`).
/// Les longueurs sont parcourues deux fois (P7, puis cumul), jamais collectées.
fn package_layout(
    manifest_len: usize,
    declared_count: u32,
    artifact_lens: impl ExactSizeIterator<Item = usize> + Clone,
) -> Result<Layout, CompilePackageError> {
    layout_within(usize::MAX, manifest_len, declared_count, artifact_lens)
}

/// Cœur de [`package_layout`] ; `ceiling` est la plus grande valeur de `usize` admise
/// par une opération de taille. La production passe `usize::MAX` ; les tests passent une
/// valeur plus petite pour exercer P2, P3 et P4 avec des tailles synthétiques réduites.
fn layout_within(
    ceiling: usize,
    manifest_len: usize,
    declared_count: u32,
    artifact_lens: impl ExactSizeIterator<Item = usize> + Clone,
) -> Result<Layout, CompilePackageError> {
    use CompilePackageError::{InconsistentBatch, PackageTooLarge};

    let n = artifact_lens.len();
    // P1.
    let count = u32::try_from(n).map_err(|_| InconsistentBatch)?;

    // P7.
    if declared_count != count {
        return Err(InconsistentBatch);
    }
    let expected_manifest = n
        .checked_mul(MANIFEST_ENTRY_LEN)
        .and_then(|entries| entries.checked_add(MANIFEST_HEADER_LEN));
    if expected_manifest != Some(manifest_len) {
        return Err(InconsistentBatch);
    }
    if artifact_lens.clone().any(|len| len == 0) {
        return Err(InconsistentBatch);
    }
    let entries_len = manifest_len
        .checked_sub(MANIFEST_HEADER_LEN)
        .ok_or(InconsistentBatch)?;

    let within = |value: usize| (value <= ceiling).then_some(value);

    // P2.
    let table_end = n
        .checked_mul(envelope::OFFSET_WIDTH)
        .and_then(|table| envelope::HEADER_LEN.checked_add(table))
        .and_then(within)
        .ok_or(PackageTooLarge)?;
    // P3.
    let first_artifact = table_end
        .checked_add(manifest_v2::HEADER_LEN)
        .and_then(|end| end.checked_add(entries_len))
        .and_then(within)
        .ok_or(PackageTooLarge)?;

    // P6 (offset écrit en `u64`) et P4 (cumul), par le même pas que l'assemblage.
    let mut cursor = first_artifact;
    for len in artifact_lens {
        cursor = next_offset(ceiling, cursor, len)?.1;
    }

    // P5.
    if cursor > CAPACITY_LIMIT {
        return Err(PackageTooLarge);
    }

    Ok(Layout {
        count,
        table_end,
        first_artifact,
        total: cursor,
    })
}

/// Pas du cumul : à partir du début `cursor` d'un artefact de longueur `len`, rend l'offset
/// à écrire (P6, `u64::try_from`) et le début de l'artefact suivant (P4, somme vérifiée
/// bornée par `ceiling`). Partagé par la validation et l'assemblage.
fn next_offset(
    ceiling: usize,
    cursor: usize,
    len: usize,
) -> Result<(u64, usize), CompilePackageError> {
    let offset = u64::try_from(cursor).map_err(|_| CompilePackageError::PackageTooLarge)?;
    let next = cursor
        .checked_add(len)
        .filter(|next| *next <= ceiling)
        .ok_or(CompilePackageError::PackageTooLarge)?;
    Ok((offset, next))
}

/// Écrit la table d'offsets `s_0 … s_{n'-1}` (`u64` little-endian) à la suite de `out`,
/// directement, sans collection intermédiaire. Exécuté après la validation du layout ;
/// les mêmes opérations vérifiées sont refaites, leur échec reste une erreur typée.
fn write_offsets(
    out: &mut Vec<u8>,
    first_artifact: usize,
    artifact_lens: impl Iterator<Item = usize>,
) -> Result<(), CompilePackageError> {
    let mut cursor = first_artifact;
    for len in artifact_lens {
        let (offset, next) = next_offset(usize::MAX, cursor, len)?;
        out.extend_from_slice(&offset.to_le_bytes());
        cursor = next;
    }
    Ok(())
}

/// Découpe le manifeste v1 : compteur (octets `6..10`) et entrées (octets à partir de
/// `MANIFEST_HEADER_LEN`). Un manifeste plus court que son en-tête est une incohérence.
fn split_manifest(manifest: &[u8]) -> Result<(u32, &[u8]), CompilePackageError> {
    let [_, _, _, _, _, _, c0, c1, c2, c3, entries @ ..] = manifest else {
        return Err(CompilePackageError::InconsistentBatch);
    };
    Ok((u32::from_le_bytes([*c0, *c1, *c2, *c3]), entries))
}

/// Compile un lot d'assets en un paquet `VPKG` de quantum `quantum_ns`.
///
/// Ordre : `quantum_ns == 0` est refusé avant toute compilation
/// (`Forge(ForgeError::ZeroValue("q"))`) ; puis `compile_assets` (mêmes erreurs, même
/// ordre, enveloppées dans `Forge(_)`) ; puis le layout, entièrement validé avant la
/// première allocation du paquet.
///
/// ```
/// use verbum_forge::{AssetSource, CompilePackageError, CompiledPackage, compile_package};
///
/// let _: fn(&[AssetSource<'_>], usize, u64) -> Result<CompiledPackage, CompilePackageError> =
///     compile_package;
/// ```
pub fn compile_package(
    sources: &[AssetSource<'_>],
    slot_count: usize,
    quantum_ns: u64,
) -> Result<CompiledPackage, CompilePackageError> {
    if quantum_ns == 0 {
        return Err(CompilePackageError::Forge(ForgeError::ZeroValue("q")));
    }
    let set = compile_assets(sources, slot_count).map_err(CompilePackageError::Forge)?;

    let (declared_count, entries) = split_manifest(set.manifest())?;
    let artifacts = set.artifacts();
    let lens = artifacts.iter().map(Vec::len);
    let layout = package_layout(set.manifest().len(), declared_count, lens.clone())?;

    // Le layout est entièrement validé, plafond `isize::MAX` compris : première
    // allocation du paquet, et première allocation de `compile_package` après
    // `compile_assets`.
    let mut bytes = Vec::with_capacity(layout.total);

    bytes.extend_from_slice(&envelope::SIGNATURE);
    bytes.extend_from_slice(&envelope::VERSION.to_le_bytes());
    bytes.extend_from_slice(&layout.count.to_le_bytes());
    write_offsets(&mut bytes, layout.first_artifact, lens)?;
    debug_assert_eq!(bytes.len(), layout.table_end);

    bytes.extend_from_slice(&manifest_v2::SIGNATURE);
    bytes.extend_from_slice(&manifest_v2::VERSION.to_le_bytes());
    bytes.extend_from_slice(&quantum_ns.to_le_bytes());
    bytes.extend_from_slice(&declared_count.to_le_bytes());
    bytes.extend_from_slice(entries);
    debug_assert_eq!(bytes.len(), layout.first_artifact);

    for artifact in artifacts {
        bytes.extend_from_slice(artifact);
    }
    debug_assert_eq!(bytes.len(), layout.total);

    Ok(CompiledPackage { bytes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use CompilePackageError::{InconsistentBatch, PackageTooLarge};

    /// Layout via la fonction de production, à partir de longueurs synthétiques.
    fn layout(
        manifest_len: usize,
        count: u32,
        lens: &[usize],
    ) -> Result<Layout, CompilePackageError> {
        package_layout(manifest_len, count, lens.iter().copied())
    }

    /// Table écrite par `write_offsets` pour `first` et `lens`, relue en `u64`.
    fn offsets(first: usize, lens: &[usize]) -> Vec<u64> {
        let mut out = Vec::new();
        write_offsets(&mut out, first, lens.iter().copied()).expect("offsets représentables");
        out.chunks_exact(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().expect("8 octets")))
            .collect()
    }

    #[test]
    fn pl1_exact_values_of_t_s0_and_offsets() {
        // Lot vide : T = 10, s_0 = 28 = fin du paquet.
        let empty = layout(10, 0, &[]);
        assert_eq!(
            empty,
            Ok(Layout {
                count: 0,
                table_end: 10,
                first_artifact: 28,
                total: 28,
            })
        );
        assert_eq!(offsets(28, &[]), Vec::<u64>::new());

        // Lot de référence : deux artefacts de 14 octets, manifeste v1 de 42 octets.
        let reference = layout(42, 2, &[14, 14]);
        assert_eq!(
            reference,
            Ok(Layout {
                count: 2,
                table_end: 26,
                first_artifact: 76,
                total: 104,
            })
        );
        assert_eq!(offsets(76, &[14, 14]), vec![76, 90]);

        // Un artefact : T = 18, s_0 = 18 + 18 + 16 = 52.
        assert_eq!(
            layout(26, 1, &[5]),
            Ok(Layout {
                count: 1,
                table_end: 18,
                first_artifact: 52,
                total: 57,
            })
        );
        assert_eq!(offsets(52, &[5]), vec![52]);

        // Longueurs inégales : T = 34, s_0 = 34 + 18 + 48 = 100, offsets cumulés.
        assert_eq!(
            layout(58, 3, &[1, 20, 300]),
            Ok(Layout {
                count: 3,
                table_end: 34,
                first_artifact: 100,
                total: 421,
            })
        );
        assert_eq!(offsets(100, &[1, 20, 300]), vec![100, 101, 121]);
    }

    #[test]
    fn pl1_write_offsets_appends_after_existing_bytes_and_reports_overflow() {
        // La table s'ajoute à la suite du contenu existant, en little-endian.
        let mut out = vec![0xEE];
        write_offsets(&mut out, 0x0102, [3usize, 4].into_iter()).expect("représentable");
        assert_eq!(
            out,
            [
                0xEE, 0x02, 0x01, 0, 0, 0, 0, 0, 0, // 0x0102
                0x05, 0x01, 0, 0, 0, 0, 0, 0, // 0x0105
            ]
        );
        // Un cumul qui déborde reste une erreur typée, jamais une saturation.
        let mut out = Vec::new();
        assert_eq!(
            write_offsets(&mut out, 1, [usize::MAX].into_iter()),
            Err(PackageTooLarge)
        );
    }

    #[test]
    fn pl2_overflow_of_p2_p3_p4_is_package_too_large() {
        // P4, arithmétique réelle : le cumul déborde `usize`.
        assert_eq!(layout(26, 1, &[usize::MAX]), Err(PackageTooLarge));
        assert_eq!(layout(42, 2, &[usize::MAX - 76, 1]), Err(PackageTooLarge));

        // P2, P3 et P4 à largeur de `usize` réduite : seuil exact de chaque opération.
        let within = |ceiling, manifest_len, count, lens: &[usize]| {
            layout_within(ceiling, manifest_len, count, lens.iter().copied()).map(|l| l.total)
        };
        // P2 : T = 18 pour n' = 1.
        assert_eq!(within(17, 26, 1, &[1]), Err(PackageTooLarge));
        // P3 : s_0 = 28 pour n' = 0.
        assert_eq!(within(27, 10, 0, &[]), Err(PackageTooLarge));
        assert_eq!(within(28, 10, 0, &[]), Ok(28));
        // P4 : total = 52 + len pour n' = 1.
        assert_eq!(within(60, 26, 1, &[8]), Ok(60));
        assert_eq!(within(60, 26, 1, &[9]), Err(PackageTooLarge));
        // P4 sur le second artefact : total = 76 + 14 + 14.
        assert_eq!(within(104, 42, 2, &[14, 14]), Ok(104));
        assert_eq!(within(103, 42, 2, &[14, 14]), Err(PackageTooLarge));
        // P6 (`u64::try_from(usize)`) est inatteignable lorsque `usize` ≤ 64 bits : le
        // chemin existe dans le code mais aucune entrée ne le provoque sur cette cible.
    }

    #[test]
    fn pl3_isize_max_threshold_is_exact_and_precedes_allocation() {
        // n' = 1 : s_0 = 52. Total exactement `isize::MAX` accepté, +1 refusé.
        let accepted = layout(26, 1, &[CAPACITY_LIMIT - 52]).expect("seuil exact accepté");
        assert_eq!(accepted.total, CAPACITY_LIMIT);
        assert_eq!(offsets(52, &[CAPACITY_LIMIT - 52]), vec![52]);
        assert_eq!(
            layout(26, 1, &[CAPACITY_LIMIT - 52 + 1]),
            Err(PackageTooLarge)
        );

        // n' = 2 : s_0 = 76, premier artefact d'un octet.
        let accepted = layout(42, 2, &[1, CAPACITY_LIMIT - 77]).expect("seuil exact accepté");
        assert_eq!(accepted.total, CAPACITY_LIMIT);
        assert_eq!(offsets(76, &[1, CAPACITY_LIMIT - 77]), vec![76, 77]);
        assert_eq!(
            layout(42, 2, &[1, CAPACITY_LIMIT - 76]),
            Err(PackageTooLarge)
        );

        // Le seuil est sans conséquence sur la valeur `isize::MAX` elle-même.
        assert_eq!(CAPACITY_LIMIT, usize::MAX >> 1);

        // `package_layout` n'alloue rien : ces appels rendent une erreur avec des
        // longueurs de l'ordre de `isize::MAX`.
        assert_eq!(layout(26, 1, &[CAPACITY_LIMIT]), Err(PackageTooLarge));
    }

    #[test]
    fn pl4_violated_p1_and_p7_invariants_are_inconsistent_batch() {
        // Compte déclaré différent du nombre d'artefacts.
        assert_eq!(layout(42, 1, &[14, 14]), Err(InconsistentBatch));
        assert_eq!(layout(42, 3, &[14, 14]), Err(InconsistentBatch));
        assert_eq!(layout(10, 1, &[]), Err(InconsistentBatch));
        // Longueur du manifeste différente de `10 + 16·n`.
        assert_eq!(layout(41, 2, &[14, 14]), Err(InconsistentBatch));
        assert_eq!(layout(43, 2, &[14, 14]), Err(InconsistentBatch));
        assert_eq!(layout(0, 0, &[]), Err(InconsistentBatch));
        assert_eq!(layout(9, 0, &[]), Err(InconsistentBatch));
        assert_eq!(layout(11, 0, &[]), Err(InconsistentBatch));
        // Artefact vide.
        assert_eq!(layout(42, 2, &[14, 0]), Err(InconsistentBatch));
        assert_eq!(layout(42, 2, &[0, 14]), Err(InconsistentBatch));
        // Priorité : P7 précède le cumul P4.
        assert_eq!(layout(42, 2, &[usize::MAX, 0]), Err(InconsistentBatch));
    }

    /// P1 : plus d'artefacts qu'un `u32` ne numérote. Itérateur synthétique : rien n'est
    /// alloué ni parcouru, le rejet précède toute itération.
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn pl4_p1_more_artifacts_than_u32_is_inconsistent_batch() {
        let too_many = usize::try_from(u32::MAX).expect("64 bits") + 1;
        let lens = std::iter::repeat_n(1usize, too_many);
        assert_eq!(package_layout(0, 0, lens.clone()), Err(InconsistentBatch));
        assert_eq!(package_layout(0, u32::MAX, lens), Err(InconsistentBatch));
    }

    #[test]
    fn split_manifest_rejects_a_manifest_shorter_than_its_header() {
        assert_eq!(split_manifest(&[]), Err(InconsistentBatch));
        assert_eq!(split_manifest(&[0; 9]), Err(InconsistentBatch));
        let mut header = [0u8; 10];
        header[6..10].copy_from_slice(&[0x02, 0x01, 0x00, 0x00]);
        assert_eq!(split_manifest(&header), Ok((0x0102, [].as_slice())));
    }
}
