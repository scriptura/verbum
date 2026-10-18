// crates/forge/src/batch.rs

//! Compilation en lot : manifeste `VMAN` et artefacts, dans l'ordre d'entrée (I2, I4,
//! I9, I12).
//!
//! Chemin nominal : `artifact_index` est la position de l'asset dans l'entrée ; la clé
//! logique n'est jamais écrite dans la sortie, seul son identifiant FNV-1a 64 l'est.

use verbum_assets::{
    MANIFEST_ENTRY_LEN, MANIFEST_HEADER_LEN, MANIFEST_SIGNATURE, MANIFEST_VERSION, SlotIndex,
};

use crate::id::fnv1a_64;
use crate::{ForgeError, compile_sprite};

/// Un asset à compiler : clé logique, source TOML, slot cible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetSource<'a> {
    /// Clé logique, fournie séparément de la source TOML ; jamais écrite dans la sortie.
    pub key: &'a str,
    /// Texte du `sprite.toml`.
    pub toml: &'a str,
    /// Ordinal du slot dans la configuration statique de l'application.
    pub slot: SlotIndex,
}

/// Résultat d'une compilation en lot : manifeste et artefacts, dans l'ordre d'entrée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledSet {
    manifest: Vec<u8>,
    artifacts: Vec<Vec<u8>>,
}

impl CompiledSet {
    /// Octets du manifeste `VMAN`.
    pub fn manifest(&self) -> &[u8] {
        &self.manifest
    }

    /// Octets des artefacts ; la position `i` est l'`artifact_index` de l'entrée `i`.
    pub fn artifacts(&self) -> &[Vec<u8>] {
        &self.artifacts
    }
}

/// Compile un lot d'assets avec l'identifiant FNV-1a 64 bits de la clé logique.
///
/// `slot_count` est la capacité : la longueur du tableau statique de liaison de
/// l'application.
pub fn compile_assets(
    sources: &[AssetSource<'_>],
    slot_count: usize,
) -> Result<CompiledSet, ForgeError> {
    compile_assets_with(sources, slot_count, fnv1a_64)
}

/// Point de test privé : même compilation, calculateur d'identifiant injecté. Visibilité
/// restreinte au crate, jamais réexporté ni accessible depuis l'API de production.
pub(crate) fn compile_assets_with(
    sources: &[AssetSource<'_>],
    _slot_count: usize,
    hash: impl Fn(&[u8]) -> u64,
) -> Result<CompiledSet, ForgeError> {
    let artifacts = sources
        .iter()
        .map(|source| compile_sprite(source.toml))
        .collect::<Result<Vec<_>, _>>()?;

    // Les index sont des `u32` dans le layout ; un lot de 2^32 assets ou plus est hors de
    // portée d'une compilation en mémoire.
    let count = u32::try_from(sources.len()).expect("moins de 2^32 assets par lot");

    let mut manifest = Vec::with_capacity(MANIFEST_HEADER_LEN + MANIFEST_ENTRY_LEN * sources.len());
    manifest.extend_from_slice(&MANIFEST_SIGNATURE);
    manifest.extend_from_slice(&MANIFEST_VERSION.to_le_bytes());
    manifest.extend_from_slice(&count.to_le_bytes());
    for (artifact_index, source) in (0..count).zip(sources) {
        manifest.extend_from_slice(&hash(source.key.as_bytes()).to_le_bytes());
        manifest.extend_from_slice(&artifact_index.to_le_bytes());
        manifest.extend_from_slice(&source.slot.get().to_le_bytes());
    }
    debug_assert_eq!(
        manifest.len(),
        MANIFEST_HEADER_LEN + MANIFEST_ENTRY_LEN * sources.len()
    );

    Ok(CompiledSet {
        manifest,
        artifacts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use verbum_assets::{Manifest, load};

    const SPRITE_32X16: &str = "width = 32\nheight = 16\n";
    const SPRITE_96X48: &str = "width = 96\nheight = 48\n";

    /// Artefact 32 × 16, littéral écrit à la main.
    const BIN_32X16: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x20, 0x00, 0x10, 0x00,
    ];

    /// Artefact 96 × 48, littéral écrit à la main.
    const BIN_96X48: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x60, 0x00, 0x30, 0x00,
    ];

    fn a() -> AssetSource<'static> {
        AssetSource {
            key: "a",
            toml: SPRITE_32X16,
            slot: SlotIndex::new(1),
        }
    }

    fn foobar() -> AssetSource<'static> {
        AssetSource {
            key: "foobar",
            toml: SPRITE_96X48,
            slot: SlotIndex::new(0),
        }
    }

    #[test]
    fn f2_the_reference_set_yields_the_literal_manifest_and_artifacts() {
        let set = compile_assets(&[a(), foobar()], 2).expect("lot valide");

        // Identifiants : constantes du RFC 9923 §8.3 (« a », « foobar »), non calculées
        // par la Forge.
        let expected_manifest: [u8; 42] = [
            0x56, 0x4D, 0x41, 0x4E, // signature
            0x01, 0x00, // version
            0x02, 0x00, 0x00, 0x00, // n = 2
            0x8C, 0xEC, 0x01, 0x86, 0x4C, 0xDC, 0x63, 0xAF, // id de « a »
            0x00, 0x00, 0x00, 0x00, // artifact_index 0
            0x01, 0x00, 0x00, 0x00, // slot_index 1
            0xE8, 0x67, 0x39, 0xF7, 0x71, 0x41, 0x94, 0x85, // id de « foobar »
            0x01, 0x00, 0x00, 0x00, // artifact_index 1
            0x00, 0x00, 0x00, 0x00, // slot_index 0
        ];
        assert_eq!(set.manifest(), expected_manifest.as_slice());
        assert_eq!(set.artifacts().len(), 2);
        assert_eq!(set.artifacts()[0], BIN_32X16);
        assert_eq!(set.artifacts()[1], BIN_96X48);
    }

    #[test]
    fn f3_permuting_the_input_order_changes_artifact_indices_only() {
        let reference = compile_assets(&[a(), foobar()], 2).expect("lot valide");
        let permuted = compile_assets(&[foobar(), a()], 2).expect("lot valide");

        let expected_manifest: [u8; 42] = [
            0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // en-tête, n = 2
            0xE8, 0x67, 0x39, 0xF7, 0x71, 0x41, 0x94, 0x85, // id de « foobar »
            0x00, 0x00, 0x00, 0x00, // artifact_index 0
            0x00, 0x00, 0x00, 0x00, // slot_index 0
            0x8C, 0xEC, 0x01, 0x86, 0x4C, 0xDC, 0x63, 0xAF, // id de « a »
            0x01, 0x00, 0x00, 0x00, // artifact_index 1
            0x01, 0x00, 0x00, 0x00, // slot_index 1
        ];
        assert_eq!(permuted.manifest(), expected_manifest.as_slice());
        assert_eq!(permuted.artifacts()[0], BIN_96X48);
        assert_eq!(permuted.artifacts()[1], BIN_32X16);

        // Mêmes identifiants et mêmes slots, indices d'artefacts permutés.
        let before = Manifest::parse(reference.manifest()).expect("manifeste conforme");
        let after = Manifest::parse(permuted.manifest()).expect("manifeste conforme");
        for entry in before.entries() {
            let moved = after.find(entry.id()).expect("même identifiant");
            assert_eq!(moved.slot_index(), entry.slot_index());
            assert_ne!(moved.artifact_index(), entry.artifact_index());
        }
    }

    #[test]
    fn f5_the_same_key_with_different_content_gives_the_same_id() {
        let small = compile_assets(&[a()], 2).expect("lot valide");
        let large = compile_assets(
            &[AssetSource {
                key: "a",
                toml: SPRITE_96X48,
                slot: SlotIndex::new(1),
            }],
            2,
        )
        .expect("lot valide");

        let id = |set: &CompiledSet| {
            Manifest::parse(set.manifest())
                .expect("manifeste conforme")
                .entries()
                .next()
                .expect("une entrée")
                .id()
        };
        assert_eq!(id(&small), id(&large));
        assert_ne!(small.artifacts(), large.artifacts());
    }

    fn two_ids(first: &'static str, second: &'static str) -> [verbum_assets::AssetId; 2] {
        let set = compile_assets(
            &[
                AssetSource {
                    key: first,
                    toml: SPRITE_32X16,
                    slot: SlotIndex::new(0),
                },
                AssetSource {
                    key: second,
                    toml: SPRITE_32X16,
                    slot: SlotIndex::new(1),
                },
            ],
            2,
        )
        .expect("lot accepté");
        let manifest = Manifest::parse(set.manifest()).expect("manifeste conforme");
        let mut entries = manifest.entries();
        [
            entries.next().expect("entrée 0").id(),
            entries.next().expect("entrée 1").id(),
        ]
    }

    #[test]
    fn f5_keys_differing_only_by_case_give_different_ids() {
        let [lower, upper] = two_ids("a", "A");
        assert_ne!(lower, upper);
    }

    #[test]
    fn f5_canonically_equivalent_unicode_keys_are_not_normalized() {
        // Séquences d'échappement, jamais les caractères : aucun outil ne peut
        // normaliser le fichier source.
        let precomposed = "\u{00E9}";
        let decomposed = "e\u{0301}";
        assert_eq!(precomposed.len(), 2);
        assert_eq!(decomposed.len(), 3);

        let [first, second] = two_ids(precomposed, decomposed);
        assert_ne!(first, second);
    }

    #[test]
    fn f7_an_empty_batch_yields_a_ten_byte_manifest_accepted_by_parse_and_load() {
        let set = compile_assets(&[], 2).expect("lot vide");
        let expected: [u8; 10] = [0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(set.manifest(), expected.as_slice());
        assert!(set.artifacts().is_empty());

        let manifest = Manifest::parse(set.manifest()).expect("manifeste vide conforme");
        assert!(manifest.is_empty());

        let mut deposits = 0;
        let result = load(set.manifest(), &[], 2, |_, _| deposits += 1);
        assert_eq!(result, Ok(()));
        assert_eq!(deposits, 0);
    }

    #[test]
    fn f8_the_output_carries_no_string_and_has_the_expected_lengths() {
        for count in 0..=3usize {
            let sources: [AssetSource<'_>; 3] = [
                AssetSource {
                    key: "foobar",
                    toml: SPRITE_32X16,
                    slot: SlotIndex::new(0),
                },
                AssetSource {
                    key: "another-key",
                    toml: SPRITE_96X48,
                    slot: SlotIndex::new(1),
                },
                AssetSource {
                    key: "third/key.png",
                    toml: SPRITE_32X16,
                    slot: SlotIndex::new(2),
                },
            ];
            let set = compile_assets(&sources[..count], 3).expect("lot valide");

            assert_eq!(set.manifest().len(), 10 + 16 * count);
            assert_eq!(set.artifacts().len(), count);
            for artifact in set.artifacts() {
                assert_eq!(artifact.len(), 14);
            }

            for source in &sources[..count] {
                let key = source.key.as_bytes();
                assert!(!set.manifest().windows(key.len()).any(|w| w == key));
                for artifact in set.artifacts() {
                    assert!(!artifact.windows(key.len()).any(|w| w == key));
                }
            }
        }
    }
}
