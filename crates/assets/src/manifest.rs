// crates/assets/src/manifest.rs

//! Manifeste `VMAN` : identité opaque des assets et liaisons (I4, I5, I10, I12).
//!
//! Layout, little-endian explicite :
//!
//! | Champ | Taille |
//! |---|---:|
//! | signature [`MANIFEST_SIGNATURE`] | 4 |
//! | version, `u16` ([`MANIFEST_VERSION`]) | 2 |
//! | nombre d'entrées `n`, `u32` | 4 |
//! | par entrée : [`AssetId`], `u64` | 8 |
//! | par entrée : `artifact_index`, `u32` | 4 |
//! | par entrée : `slot_index`, `u32` | 4 |
//!
//! Longueur totale attendue exactement `10 + 16 × n`. Aucun nom, aucun chemin.

use core::fmt;

/// Signature du manifeste. Choix local provisoire, non invariant.
pub const MANIFEST_SIGNATURE: [u8; 4] = *b"VMAN";

/// Version du layout du manifeste acceptée par ce lecteur. Choix local provisoire.
pub const MANIFEST_VERSION: u16 = 1;

/// Longueur de l'en-tête : signature (4) + version (2) + nombre d'entrées (4).
pub const MANIFEST_HEADER_LEN: usize = 10;

/// Longueur d'une entrée : identifiant (8) + `artifact_index` (4) + `slot_index` (4).
pub const MANIFEST_ENTRY_LEN: usize = 16;

// Cohérence des constantes de layout, vérifiée à la compilation.
const _: () = {
    assert!(MANIFEST_HEADER_LEN == MANIFEST_SIGNATURE.len() + size_of::<u16>() + size_of::<u32>());
    assert!(MANIFEST_ENTRY_LEN == size_of::<u64>() + 2 * size_of::<u32>());
};

/// Identité logique d'un asset, produite par la Forge (FNV-1a 64 de la clé logique).
///
/// Type opaque : seul le lecteur du manifeste le construit. Ni constructeur public,
/// ni accesseur numérique, ni conversion depuis ou vers un entier, ni `Display`.
///
/// Pas de construction depuis un entier :
///
/// ```compile_fail,E0423
/// use verbum_assets::AssetId;
///
/// let _ = AssetId(5);
/// ```
///
/// ```compile_fail,E0599
/// use verbum_assets::AssetId;
///
/// let _ = AssetId::new(5);
/// ```
///
/// ```compile_fail,E0624
/// use verbum_assets::AssetId;
///
/// let _ = AssetId::from_raw(5);
/// ```
///
/// Pas d'accès à la valeur numérique :
///
/// ```compile_fail,E0616
/// use verbum_assets::AssetId;
///
/// fn raw(id: AssetId) -> u64 {
///     id.0
/// }
/// ```
///
/// ```compile_fail,E0599
/// use verbum_assets::AssetId;
///
/// fn raw(id: AssetId) -> u64 {
///     id.get()
/// }
/// ```
///
/// ```compile_fail,E0599
/// use verbum_assets::AssetId;
///
/// fn raw(id: AssetId) -> u64 {
///     id.value()
/// }
/// ```
///
/// Pas de conversion depuis ou vers un entier :
///
/// ```compile_fail
/// use verbum_assets::AssetId;
///
/// let _ = AssetId::from(5u64);
/// ```
///
/// ```compile_fail,E0277
/// use verbum_assets::AssetId;
///
/// fn raw(id: AssetId) -> u64 {
///     id.into()
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AssetId(u64);

impl AssetId {
    /// Réservé au lecteur du manifeste (et aux valeurs attendues des tests du crate).
    pub(crate) const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

// Manuel : la valeur numérique n'est jamais affichée.
impl fmt::Debug for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AssetId(<opaque>)")
    }
}

/// Position d'un artefact dans la collection ordonnée fournie au chargement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactIndex(u32);

impl ArtifactIndex {
    /// La valeur de l'indice.
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Ordinal d'un slot dans la configuration statique des slots.
///
/// Le constructeur ne vérifie rien : il ne connaît pas la capacité. Les contrôles
/// `slot_index < slot_count` et d'unicité sont assurés par la Forge et par le chargement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotIndex(u32);

impl SlotIndex {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// La valeur de l'indice.
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Entrée du manifeste : valeur `Copy`, sans référence aux octets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManifestEntry {
    id: AssetId,
    artifact_index: ArtifactIndex,
    slot_index: SlotIndex,
}

impl ManifestEntry {
    pub const fn id(self) -> AssetId {
        self.id
    }

    pub const fn artifact_index(self) -> ArtifactIndex {
        self.artifact_index
    }

    pub const fn slot_index(self) -> SlotIndex {
        self.slot_index
    }
}

/// Rejet d'un manifeste au niveau 1 (octets seuls). Énumération `Copy`, sans allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestError {
    /// Moins d'octets que la formule `10 + 16 × n` ne l'exige.
    Truncated,
    /// Signature différente de [`MANIFEST_SIGNATURE`].
    Signature,
    /// Version différente de [`MANIFEST_VERSION`].
    Version,
    /// Octets au-delà de `10 + 16 × n`.
    TrailingBytes,
    /// Deux entrées portent le même [`AssetId`].
    DuplicateId,
}

/// Rejet d'une liaison au niveau 2 (collection d'artefacts et configuration des slots).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingError {
    /// Le nombre d'entrées diffère du nombre d'artefacts fournis.
    EntryCountMismatch,
    /// Plus d'entrées que de slots (`n > slot_count`).
    CapacityExceeded,
    /// Un `artifact_index` désigne un artefact absent de la collection.
    ArtifactIndexOutOfRange,
    /// Un `slot_index` dépasse la configuration des slots.
    SlotIndexOutOfRange,
    /// Deux entrées désignent le même artefact.
    DuplicateArtifactIndex,
    /// Deux entrées désignent le même slot.
    DuplicateSlotIndex,
}

/// Vue sur un manifeste : emprunte les octets analysés, ne les possède pas.
pub struct Manifest<'a> {
    entries: &'a [u8],
}

/// Itération sur les entrées d'un manifeste, dans l'ordre des octets.
pub struct Entries<'a> {
    chunks: core::slice::ChunksExact<'a, u8>,
}

impl<'a> Manifest<'a> {
    /// Niveau 1 : contrôles sur les seuls octets du manifeste.
    ///
    /// Ordre des contrôles : en-tête complet, signature, version, longueur exacte
    /// `10 + 16 × n` (calculée en `u64` avec opérations vérifiées, conversion en `usize`
    /// vérifiée) puis unicité des identifiants. Aucune allocation.
    pub fn parse(bytes: &'a [u8]) -> Result<Manifest<'a>, ManifestError> {
        let [s0, s1, s2, s3, v0, v1, n0, n1, n2, n3, rest @ ..] = bytes else {
            return Err(ManifestError::Truncated);
        };
        if [*s0, *s1, *s2, *s3] != MANIFEST_SIGNATURE {
            return Err(ManifestError::Signature);
        }
        if u16::from_le_bytes([*v0, *v1]) != MANIFEST_VERSION {
            return Err(ManifestError::Version);
        }

        let count = u64::from(u32::from_le_bytes([*n0, *n1, *n2, *n3]));
        let expected = count
            .checked_mul(MANIFEST_ENTRY_LEN as u64)
            .and_then(|len| usize::try_from(len).ok())
            .ok_or(ManifestError::Truncated)?;
        if rest.len() < expected {
            return Err(ManifestError::Truncated);
        }
        if rest.len() > expected {
            return Err(ManifestError::TrailingBytes);
        }

        let manifest = Manifest { entries: rest };
        // Recherche de doublons sans allocation : comparaisons deux à deux, bornées par `n`.
        for (position, entry) in manifest.entries().enumerate() {
            if manifest
                .entries()
                .skip(position + 1)
                .any(|other| other.id == entry.id)
            {
                return Err(ManifestError::DuplicateId);
            }
        }
        Ok(manifest)
    }

    /// Nombre d'entrées.
    pub fn len(&self) -> usize {
        self.entries.len() / MANIFEST_ENTRY_LEN
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entrées dans l'ordre des octets, sans allocation.
    pub fn entries(&self) -> Entries<'a> {
        Entries {
            chunks: self.entries.chunks_exact(MANIFEST_ENTRY_LEN),
        }
    }

    /// Recherche linéaire sans allocation, réservée au chargement.
    pub fn find(&self, id: AssetId) -> Option<ManifestEntry> {
        self.entries().find(|entry| entry.id == id)
    }

    /// Niveau 2 : cohérence des liaisons avec la collection d'artefacts et les slots.
    ///
    /// Ordre des contrôles : nombre d'entrées égal au nombre d'artefacts, capacité,
    /// bornes de chaque `artifact_index` puis `slot_index`, unicité des `artifact_index`,
    /// unicité des `slot_index`. Une capacité supérieure au nombre d'entrées est acceptée.
    pub fn check_bindings(
        &self,
        artifact_count: usize,
        slot_count: usize,
    ) -> Result<(), BindingError> {
        let count = self.len();
        if count != artifact_count {
            return Err(BindingError::EntryCountMismatch);
        }
        if count > slot_count {
            return Err(BindingError::CapacityExceeded);
        }

        let within = |index: u32, limit: usize| usize::try_from(index).is_ok_and(|i| i < limit);
        for entry in self.entries() {
            if !within(entry.artifact_index.get(), artifact_count) {
                return Err(BindingError::ArtifactIndexOutOfRange);
            }
            if !within(entry.slot_index.get(), slot_count) {
                return Err(BindingError::SlotIndexOutOfRange);
            }
        }

        // Unicité sans allocation : comparaisons deux à deux, bornées par `slot_count`.
        for (position, entry) in self.entries().enumerate() {
            if self
                .entries()
                .skip(position + 1)
                .any(|other| other.artifact_index == entry.artifact_index)
            {
                return Err(BindingError::DuplicateArtifactIndex);
            }
        }
        for (position, entry) in self.entries().enumerate() {
            if self
                .entries()
                .skip(position + 1)
                .any(|other| other.slot_index == entry.slot_index)
            {
                return Err(BindingError::DuplicateSlotIndex);
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Manifest<'_> {
    // Les octets contiennent les identifiants : seul le nombre d'entrées est affiché.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manifest")
            .field("entries", &self.len())
            .finish()
    }
}

impl Iterator for Entries<'_> {
    type Item = ManifestEntry;

    fn next(&mut self) -> Option<ManifestEntry> {
        self.chunks.next().map(decode)
    }
}

/// Décode une entrée de 16 octets, champ par champ.
fn decode(chunk: &[u8]) -> ManifestEntry {
    let [
        i0,
        i1,
        i2,
        i3,
        i4,
        i5,
        i6,
        i7,
        a0,
        a1,
        a2,
        a3,
        t0,
        t1,
        t2,
        t3,
    ] = chunk
    else {
        // `chunks_exact(MANIFEST_ENTRY_LEN)` ne produit que des tranches de 16 octets.
        unreachable!("entrée de longueur différente de MANIFEST_ENTRY_LEN");
    };
    ManifestEntry {
        id: AssetId::from_raw(u64::from_le_bytes([*i0, *i1, *i2, *i3, *i4, *i5, *i6, *i7])),
        artifact_index: ArtifactIndex(u32::from_le_bytes([*a0, *a1, *a2, *a3])),
        slot_index: SlotIndex(u32::from_le_bytes([*t0, *t1, *t2, *t3])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    /// Manifeste littéral à deux entrées, écrit à la main, indépendant des constantes :
    /// signature `VMAN`, version 1, n = 2.
    /// Entrée 0 : id 0x1122334455667788, artefact 0, slot 1.
    /// Entrée 1 : id 0x99AABBCCDDEEFF00, artefact 1, slot 0.
    const LITERAL: [u8; 42] = [
        0x56, 0x4D, 0x41, 0x4E, // signature
        0x01, 0x00, // version, little-endian
        0x02, 0x00, 0x00, 0x00, // n = 2, little-endian
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id 0
        0x00, 0x00, 0x00, 0x00, // artifact_index 0
        0x01, 0x00, 0x00, 0x00, // slot_index 1
        0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id 1
        0x01, 0x00, 0x00, 0x00, // artifact_index 1
        0x00, 0x00, 0x00, 0x00, // slot_index 0
    ];

    const ID_0: u64 = 0x1122_3344_5566_7788;
    const ID_1: u64 = 0x99AA_BBCC_DDEE_FF00;

    fn entry(id: u64, artifact: u32, slot: u32) -> ManifestEntry {
        ManifestEntry {
            id: AssetId::from_raw(id),
            artifact_index: ArtifactIndex(artifact),
            slot_index: SlotIndex(slot),
        }
    }

    /// Écrit en tête de `buffer` un manifeste de `n` entrées aux identifiants distincts.
    fn synthetic(n: u32, buffer: &mut [u8; 10 + 16 * 3]) -> &[u8] {
        buffer[..4].copy_from_slice(b"VMAN");
        buffer[4..6].copy_from_slice(&1u16.to_le_bytes());
        buffer[6..10].copy_from_slice(&n.to_le_bytes());
        for i in 0..n {
            let at = 10 + 16 * i as usize;
            buffer[at..at + 8].copy_from_slice(&(0x100 + u64::from(i)).to_le_bytes());
            buffer[at + 8..at + 12].copy_from_slice(&i.to_le_bytes());
            buffer[at + 12..at + 16].copy_from_slice(&(n - 1 - i).to_le_bytes());
        }
        &buffer[..10 + 16 * n as usize]
    }

    #[test]
    fn m1_parses_the_literal_manifest_and_finds_each_entry() {
        let manifest = Manifest::parse(&LITERAL).expect("manifeste conforme");
        assert_eq!(manifest.len(), 2);
        assert!(!manifest.is_empty());

        let mut entries = manifest.entries();
        assert_eq!(entries.next(), Some(entry(ID_0, 0, 1)));
        assert_eq!(entries.next(), Some(entry(ID_1, 1, 0)));
        assert_eq!(entries.next(), None);

        assert_eq!(
            manifest.find(AssetId::from_raw(ID_0)),
            Some(entry(ID_0, 0, 1))
        );
        assert_eq!(
            manifest.find(AssetId::from_raw(ID_1)),
            Some(entry(ID_1, 1, 0))
        );
        assert_eq!(manifest.find(AssetId::from_raw(ID_0 ^ 1)), None);

        let found = manifest.find(AssetId::from_raw(ID_0)).unwrap();
        assert_eq!(found.id(), AssetId::from_raw(ID_0));
        assert_eq!(found.artifact_index().get(), 0);
        assert_eq!(found.slot_index().get(), 1);
    }

    #[test]
    fn m2_length_formula_holds_for_zero_to_three_entries() {
        for n in 0..=3u32 {
            let mut buffer = [0u8; 10 + 16 * 3];
            let bytes = synthetic(n, &mut buffer);
            assert_eq!(bytes.len(), 10 + 16 * n as usize);
            let manifest = Manifest::parse(bytes).expect("longueur conforme");
            assert_eq!(manifest.len(), n as usize);
            assert_eq!(manifest.entries().count(), n as usize);
        }

        let mut buffer = [0u8; 10 + 16 * 3];
        let empty = Manifest::parse(synthetic(0, &mut buffer)).expect("manifeste vide");
        assert!(empty.is_empty());
        assert_eq!(empty.entries().next(), None);
        assert_eq!(empty.find(AssetId::from_raw(0x100)), None);
    }

    #[test]
    fn m3_every_truncation_is_rejected() {
        for len in 0..LITERAL.len() {
            assert_eq!(
                Manifest::parse(&LITERAL[..len]).err(),
                Some(ManifestError::Truncated),
                "préfixe de {len} octets"
            );
        }
    }

    #[test]
    fn m3_a_huge_entry_count_with_few_bytes_is_rejected_without_panic() {
        for count in [u32::MAX, u32::MAX - 1, 0x1000_0000, 3] {
            let mut bytes = LITERAL;
            bytes[6..10].copy_from_slice(&count.to_le_bytes());
            assert_eq!(
                Manifest::parse(&bytes).err(),
                Some(ManifestError::Truncated),
                "n = {count}"
            );
        }
    }

    #[test]
    fn m3_surplus_is_rejected() {
        let mut surplus = [0u8; 43];
        surplus[..42].copy_from_slice(&LITERAL);
        assert_eq!(
            Manifest::parse(&surplus).err(),
            Some(ManifestError::TrailingBytes)
        );

        let mut fewer = LITERAL;
        fewer[6..10].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(
            Manifest::parse(&fewer).err(),
            Some(ManifestError::TrailingBytes)
        );
    }

    #[test]
    fn m4_altered_signature_is_rejected() {
        for index in 0..4 {
            let mut bytes = LITERAL;
            bytes[index] ^= 0xFF;
            assert_eq!(
                Manifest::parse(&bytes).err(),
                Some(ManifestError::Signature),
                "octet de signature {index} altéré"
            );
        }
        // Le texte d'un fichier TOML n'est pas un manifeste (I1).
        let source = b"width = 258\nheight = 772\n";
        assert!(source.len() >= MANIFEST_HEADER_LEN);
        assert_eq!(
            Manifest::parse(source).err(),
            Some(ManifestError::Signature)
        );
    }

    #[test]
    fn m4_unknown_version_is_rejected() {
        for version in [[0x00, 0x00], [0x02, 0x00], [0x00, 0x01], [0xFF, 0xFF]] {
            let mut bytes = LITERAL;
            bytes[4..6].copy_from_slice(&version);
            assert_eq!(
                Manifest::parse(&bytes).err(),
                Some(ManifestError::Version),
                "version {version:?}"
            );
        }
    }

    #[test]
    fn m5_duplicate_asset_ids_are_rejected_by_parse() {
        // Entrées 0 et 1 identiques.
        let mut adjacent = LITERAL;
        let first_id: [u8; 8] = LITERAL[10..18].try_into().unwrap();
        adjacent[26..34].copy_from_slice(&first_id);
        assert_eq!(
            Manifest::parse(&adjacent).err(),
            Some(ManifestError::DuplicateId)
        );

        // Entrées 0 et 2 identiques, entrée 1 distincte.
        let mut buffer = [0u8; 10 + 16 * 3];
        let bytes = synthetic(3, &mut buffer);
        let mut spaced = [0u8; 10 + 16 * 3];
        spaced.copy_from_slice(bytes);
        let first: [u8; 8] = spaced[10..18].try_into().unwrap();
        spaced[42..50].copy_from_slice(&first);
        assert_eq!(
            Manifest::parse(&spaced).err(),
            Some(ManifestError::DuplicateId)
        );
    }

    /// Tampon fixe pour formater sans allocation.
    struct Buffer {
        data: [u8; 64],
        len: usize,
    }

    impl Write for Buffer {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            let end = self.len + text.len();
            self.data
                .get_mut(self.len..end)
                .ok_or(fmt::Error)?
                .copy_from_slice(text.as_bytes());
            self.len = end;
            Ok(())
        }
    }

    #[test]
    fn m6_debug_of_an_asset_id_never_contains_its_numeric_value() {
        let mut buffer = Buffer {
            data: [0; 64],
            len: 0,
        };
        write!(buffer, "{:?}", AssetId::from_raw(0x0123_4567_89AB_CDEF)).unwrap();
        let text = core::str::from_utf8(&buffer.data[..buffer.len]).unwrap();
        assert_eq!(text, "AssetId(<opaque>)");
    }

    /// Écrit en tête de `buffer` un manifeste dont les entrées sont `(id, artefact, slot)`.
    fn manifest_of<'b>(entries: &[(u64, u32, u32)], buffer: &'b mut [u8; 10 + 16 * 3]) -> &'b [u8] {
        buffer[..4].copy_from_slice(b"VMAN");
        buffer[4..6].copy_from_slice(&1u16.to_le_bytes());
        buffer[6..10].copy_from_slice(&(entries.len() as u32).to_le_bytes());
        for (i, (id, artifact, slot)) in entries.iter().enumerate() {
            let at = 10 + 16 * i;
            buffer[at..at + 8].copy_from_slice(&id.to_le_bytes());
            buffer[at + 8..at + 12].copy_from_slice(&artifact.to_le_bytes());
            buffer[at + 12..at + 16].copy_from_slice(&slot.to_le_bytes());
        }
        &buffer[..10 + 16 * entries.len()]
    }

    fn bindings(
        entries: &[(u64, u32, u32)],
        artifact_count: usize,
        slot_count: usize,
    ) -> Result<(), BindingError> {
        let mut buffer = [0u8; 10 + 16 * 3];
        Manifest::parse(manifest_of(entries, &mut buffer))
            .expect("manifeste de niveau 1 conforme")
            .check_bindings(artifact_count, slot_count)
    }

    #[test]
    fn m7_entry_count_must_equal_the_artifact_count() {
        let entries = [(1, 0, 0), (2, 1, 1)];
        assert_eq!(
            bindings(&entries, 3, 2),
            Err(BindingError::EntryCountMismatch)
        );
        assert_eq!(
            bindings(&entries, 1, 2),
            Err(BindingError::EntryCountMismatch)
        );
        assert_eq!(
            bindings(&entries, 0, 2),
            Err(BindingError::EntryCountMismatch)
        );
    }

    #[test]
    fn m7_more_entries_than_slots_is_rejected() {
        let entries = [(1, 0, 0), (2, 1, 1)];
        assert_eq!(
            bindings(&entries, 2, 1),
            Err(BindingError::CapacityExceeded)
        );
        assert_eq!(
            bindings(&entries, 2, 0),
            Err(BindingError::CapacityExceeded)
        );
    }

    #[test]
    fn m7_out_of_range_artifact_index_is_rejected() {
        assert_eq!(
            bindings(&[(1, 0, 0), (2, 2, 1)], 2, 2),
            Err(BindingError::ArtifactIndexOutOfRange)
        );
        assert_eq!(
            bindings(&[(1, u32::MAX, 0), (2, 1, 1)], 2, 2),
            Err(BindingError::ArtifactIndexOutOfRange)
        );
    }

    #[test]
    fn m7_out_of_range_slot_index_is_rejected() {
        assert_eq!(
            bindings(&[(1, 0, 0), (2, 1, 2)], 2, 2),
            Err(BindingError::SlotIndexOutOfRange)
        );
        assert_eq!(
            bindings(&[(1, 0, u32::MAX), (2, 1, 1)], 2, 2),
            Err(BindingError::SlotIndexOutOfRange)
        );
    }

    #[test]
    fn m7_duplicate_artifact_indices_are_rejected() {
        assert_eq!(
            bindings(&[(1, 0, 0), (2, 0, 1)], 2, 2),
            Err(BindingError::DuplicateArtifactIndex)
        );
    }

    #[test]
    fn m7_duplicate_slot_indices_are_rejected() {
        assert_eq!(
            bindings(&[(1, 0, 1), (2, 1, 1)], 2, 2),
            Err(BindingError::DuplicateSlotIndex)
        );
    }

    #[test]
    fn m7_a_slot_count_above_the_entry_count_is_accepted() {
        let entries = [(1, 0, 0), (2, 1, 2)];
        assert_eq!(bindings(&entries, 2, 3), Ok(()));
        assert_eq!(bindings(&entries, 2, 64), Ok(()));
        assert_eq!(bindings(&[(1, 1, 0), (2, 0, 1)], 2, 2), Ok(()));
        assert_eq!(bindings(&[], 0, 0), Ok(()));
        assert_eq!(bindings(&[], 0, 2), Ok(()));
    }

    #[test]
    fn m8_slot_index_new_returns_its_value_without_any_check() {
        for value in [0, 1, 2, 63, u32::MAX - 1, u32::MAX] {
            assert_eq!(SlotIndex::new(value).get(), value);
        }
    }
}
