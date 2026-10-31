// crates/forge/src/batch.rs

//! Compilation en lot : manifeste `VMAN` et artefacts, dans l'ordre d'entrée (I2, I4,
//! I9, I12).
//!
//! Chemin nominal : `artifact_index` est la position de l'asset dans l'entrée ; la clé
//! logique n'est jamais écrite dans la sortie, seul son identifiant FNV-1a 64 l'est.

use verbum_assets::{
    MANIFEST_ENTRY_LEN, MANIFEST_HEADER_LEN, MANIFEST_SIGNATURE, MANIFEST_VERSION, SlotIndex,
    Tileset,
};

use crate::id::fnv1a_64;
use crate::map::{MapSource, encode_map, parse_map};
use crate::{ForgeError, compile_font, compile_sprite, compile_tileset};

/// Genre d'un asset à compiler. Ce n'est pas un champ TOML : il est fourni à côté de la
/// source, et la Forge n'en déduit jamais le genre du contenu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    /// Source `sprite.toml`, artefact `VSPR`.
    Sprite,
    /// Source `tileset.toml`, artefact `VTLS`.
    Tileset,
    /// Source `map.toml`, artefact `VMAP` ; référence un tileset du même lot par sa clé.
    Map,
    /// Source `font.toml`, artefact `VFNT` ; ne référence aucun autre asset.
    Font,
}

/// Un asset à compiler : genre, clé logique, source TOML, slot cible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetSource<'a> {
    /// Genre de l'asset : détermine le compilateur de la source.
    pub kind: AssetKind,
    /// Clé logique, fournie séparément de la source TOML ; jamais écrite dans la sortie.
    pub key: &'a str,
    /// Texte de la source TOML du genre `kind`.
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
/// l'application. Aucun paramètre de hash : la signature publique est
///
/// ```
/// use verbum_forge::{AssetSource, CompiledSet, ForgeError, compile_assets};
///
/// let _: fn(&[AssetSource<'_>], usize) -> Result<CompiledSet, ForgeError> = compile_assets;
/// ```
///
/// Le point de test à calculateur injectable n'est pas accessible depuis l'extérieur
/// du crate :
///
/// ```compile_fail,E0432
/// use verbum_forge::compile_assets_with;
/// ```
///
/// ```compile_fail,E0603
/// let _ = verbum_forge::batch::compile_assets_with(&[], 0, |_: &[u8]| 0u64);
/// ```
///
/// ```compile_fail,E0603
/// let _ = verbum_forge::id::fnv1a_64(b"a");
/// ```
///
/// Aucun calculateur ne se passe à `compile_assets` :
///
/// ```compile_fail,E0061
/// let _ = verbum_forge::compile_assets(&[], 0, |_: &[u8]| 0u64);
/// ```
pub fn compile_assets(
    sources: &[AssetSource<'_>],
    slot_count: usize,
) -> Result<CompiledSet, ForgeError> {
    compile_assets_with(sources, slot_count, fnv1a_64)
}

/// Nombre d'entrées en `u32`, la largeur des index du layout.
fn asset_count(len: usize) -> Result<u32, ForgeError> {
    u32::try_from(len).map_err(|_| ForgeError::TooManyAssets)
}

/// Première paire `(first, second)`, `first < second`, d'entrées vérifiant `same`.
fn first_pair(
    sources: &[AssetSource<'_>],
    same: impl Fn(&AssetSource<'_>, &AssetSource<'_>) -> bool,
) -> Option<(usize, usize)> {
    sources.iter().enumerate().find_map(|(second, later)| {
        sources[..second]
            .iter()
            .position(|earlier| same(earlier, later))
            .map(|first| (first, second))
    })
}

/// Artefact prêt, ou carte analysée dont la référence reste à résoudre.
enum Pending {
    Ready(Vec<u8>),
    Map(MapSource),
}

/// Résout la référence d'une carte dans le lot, valide ses indices et l'encode. La clé du
/// tileset désigne une entrée du lot de genre `Tileset` (une clé absente, une entrée d'un
/// autre genre ou la carte elle-même sont refusées) ; chaque indice doit être strictement
/// inférieur au `count` de ce tileset. L'identifiant écrit est celui de la clé, calculé
/// par le même calculateur que les identifiants du manifeste.
fn resolve_map(
    sources: &[AssetSource<'_>],
    counts: &[Option<u16>],
    map: &MapSource,
    id: &impl Fn(&AssetSource<'_>) -> u64,
) -> Result<Vec<u8>, ForgeError> {
    let position = sources
        .iter()
        .position(|source| source.key == map.tileset_key)
        .ok_or(ForgeError::ReferenceNotFound)?;
    let referent = &sources[position];
    let Some(count) = counts[position].filter(|_| referent.kind == AssetKind::Tileset) else {
        return Err(ForgeError::ReferenceKind);
    };
    if map.cells.iter().any(|cell| *cell >= count) {
        return Err(ForgeError::CellOutOfRange);
    }
    Ok(encode_map(id(referent), map))
}

/// Point de test privé : même compilation, calculateur d'identifiant injecté. Visibilité
/// restreinte au crate, jamais réexporté ni accessible depuis l'API de production.
pub(crate) fn compile_assets_with(
    sources: &[AssetSource<'_>],
    slot_count: usize,
    hash: impl Fn(&[u8]) -> u64,
) -> Result<CompiledSet, ForgeError> {
    let count = asset_count(sources.len())?;

    // Rien n'est émis tant que tous les contrôles ne sont pas passés. La clé dupliquée est
    // détectée avant la collision d'identifiants.
    if let Some(entry) = sources.iter().position(|source| source.key.is_empty()) {
        return Err(ForgeError::EmptyKey { entry });
    }
    if let Some((first, second)) = first_pair(sources, |a, b| a.key == b.key) {
        return Err(ForgeError::DuplicateKey { first, second });
    }
    let within = |slot: SlotIndex| usize::try_from(slot.get()).is_ok_and(|s| s < slot_count);
    if let Some(entry) = sources.iter().position(|source| !within(source.slot)) {
        return Err(ForgeError::SlotOutOfRange { entry });
    }
    if let Some((first, second)) = first_pair(sources, |a, b| a.slot == b.slot) {
        return Err(ForgeError::DuplicateSlot { first, second });
    }
    let id = |source: &AssetSource<'_>| hash(source.key.as_bytes());
    if let Some((first, second)) = first_pair(sources, |a, b| id(a) == id(b)) {
        return Err(ForgeError::IdCollision { first, second });
    }

    // Premier temps : analyse de chaque source seule (les cartes ne sont pas encore encodées).
    let wrap = |entry: usize| {
        move |cause: ForgeError| ForgeError::InvalidSource {
            entry,
            cause: Box::new(cause),
        }
    };
    let pending = sources
        .iter()
        .enumerate()
        .map(|(entry, source)| {
            match source.kind {
                AssetKind::Sprite => compile_sprite(source.toml).map(Pending::Ready),
                AssetKind::Tileset => compile_tileset(source.toml).map(Pending::Ready),
                AssetKind::Map => parse_map(source.toml).map(Pending::Map),
                AssetKind::Font => compile_font(source.toml).map(Pending::Ready),
            }
            .map_err(wrap(entry))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Second temps : résolution des références et validation croisée, puis encodage.
    // `count` de chaque tileset, lu sur l'artefact déjà compilé.
    let counts: Vec<Option<u16>> = pending
        .iter()
        .map(|item| match item {
            Pending::Ready(bytes) => Tileset::read(bytes).ok().map(|tileset| tileset.count),
            Pending::Map(_) => None,
        })
        .collect();
    let artifacts = pending
        .into_iter()
        .enumerate()
        .map(|(entry, item)| match item {
            Pending::Ready(bytes) => Ok(bytes),
            Pending::Map(map) => resolve_map(sources, &counts, &map, &id).map_err(wrap(entry)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut manifest = Vec::with_capacity(MANIFEST_HEADER_LEN + MANIFEST_ENTRY_LEN * sources.len());
    manifest.extend_from_slice(&MANIFEST_SIGNATURE);
    manifest.extend_from_slice(&MANIFEST_VERSION.to_le_bytes());
    manifest.extend_from_slice(&count.to_le_bytes());
    for (artifact_index, source) in (0..count).zip(sources) {
        manifest.extend_from_slice(&id(source).to_le_bytes());
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
    use verbum_assets::{Format, Manifest, load};

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
            kind: AssetKind::Sprite,
            key: "a",
            toml: SPRITE_32X16,
            slot: SlotIndex::new(1),
        }
    }

    fn foobar() -> AssetSource<'static> {
        AssetSource {
            kind: AssetKind::Sprite,
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
                kind: AssetKind::Sprite,
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
                    kind: AssetKind::Sprite,
                    key: first,
                    toml: SPRITE_32X16,
                    slot: SlotIndex::new(0),
                },
                AssetSource {
                    kind: AssetKind::Sprite,
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

    fn source<'a>(key: &'a str, toml: &'a str, slot: u32) -> AssetSource<'a> {
        AssetSource {
            kind: AssetKind::Sprite,
            key,
            toml,
            slot: SlotIndex::new(slot),
        }
    }

    #[test]
    fn f4_an_empty_key_is_rejected() {
        let result = compile_assets(
            &[source("a", SPRITE_32X16, 0), source("", SPRITE_96X48, 1)],
            2,
        );
        assert_eq!(result, Err(ForgeError::EmptyKey { entry: 1 }));
    }

    #[test]
    fn f4_a_duplicate_key_is_rejected() {
        let result = compile_assets(
            &[
                source("a", SPRITE_32X16, 0),
                source("foobar", SPRITE_96X48, 1),
                source("a", SPRITE_96X48, 2),
            ],
            3,
        );
        assert_eq!(
            result,
            Err(ForgeError::DuplicateKey {
                first: 0,
                second: 2
            })
        );
    }

    #[test]
    fn f4_a_duplicate_slot_index_is_rejected() {
        let result = compile_assets(
            &[
                source("a", SPRITE_32X16, 1),
                source("foobar", SPRITE_96X48, 1),
            ],
            2,
        );
        assert_eq!(
            result,
            Err(ForgeError::DuplicateSlot {
                first: 0,
                second: 1
            })
        );
    }

    #[test]
    fn f4_a_slot_index_at_or_above_the_capacity_is_rejected() {
        for slot in [2, 3, u32::MAX] {
            let result = compile_assets(
                &[
                    source("a", SPRITE_32X16, 0),
                    source("foobar", SPRITE_96X48, slot),
                ],
                2,
            );
            assert_eq!(
                result,
                Err(ForgeError::SlotOutOfRange { entry: 1 }),
                "slot {slot}"
            );
        }
    }

    #[test]
    fn f4_one_invalid_source_among_valid_ones_yields_no_output() {
        let result = compile_assets(
            &[
                source("a", SPRITE_32X16, 0),
                source("broken", "width = ", 1),
                source("foobar", SPRITE_96X48, 2),
            ],
            3,
        );
        let Err(ForgeError::InvalidSource { entry, cause }) = result else {
            panic!("attendu : source invalide, obtenu {result:?}");
        };
        assert_eq!(entry, 1);
        assert!(matches!(*cause, ForgeError::Syntax(_)));

        let result = compile_assets(
            &[
                source("a", SPRITE_32X16, 0),
                source("zero", "width = 0\nheight = 4", 1),
            ],
            2,
        );
        assert_eq!(
            result,
            Err(ForgeError::InvalidSource {
                entry: 1,
                cause: Box::new(ForgeError::ZeroValue("width")),
            })
        );
    }

    #[test]
    fn f6_two_distinct_keys_with_the_same_id_are_rejected_as_a_collision() {
        let constant = |_: &[u8]| 7u64;
        let result = compile_assets_with(
            &[source("x", SPRITE_32X16, 0), source("y", SPRITE_96X48, 1)],
            2,
            constant,
        );
        assert_eq!(
            result,
            Err(ForgeError::IdCollision {
                first: 0,
                second: 1
            })
        );
    }

    #[test]
    fn f6b_duplicate_keys_take_precedence_over_a_collision() {
        let constant = |_: &[u8]| 7u64;
        let result = compile_assets_with(
            &[source("x", SPRITE_32X16, 0), source("x", SPRITE_96X48, 1)],
            2,
            constant,
        );
        assert_eq!(
            result,
            Err(ForgeError::DuplicateKey {
                first: 0,
                second: 1
            })
        );
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn an_entry_count_above_u32_max_is_a_typed_error_not_a_panic() {
        let largest = usize::try_from(u32::MAX).expect("usize de 64 bits");
        assert_eq!(asset_count(0), Ok(0));
        assert_eq!(asset_count(largest), Ok(u32::MAX));
        assert_eq!(asset_count(largest + 1), Err(ForgeError::TooManyAssets));
        assert_eq!(asset_count(usize::MAX), Err(ForgeError::TooManyAssets));
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
        let result = load(set.manifest(), &[], &[Format::Sprite; 2], |_, _| {
            deposits += 1
        });
        assert_eq!(result, Ok(()));
        assert_eq!(deposits, 0);
    }

    #[test]
    fn f8_the_output_carries_no_string_and_has_the_expected_lengths() {
        for count in 0..=3usize {
            let sources: [AssetSource<'_>; 3] = [
                AssetSource {
                    kind: AssetKind::Sprite,
                    key: "foobar",
                    toml: SPRITE_32X16,
                    slot: SlotIndex::new(0),
                },
                AssetSource {
                    kind: AssetKind::Sprite,
                    key: "another-key",
                    toml: SPRITE_96X48,
                    slot: SlotIndex::new(1),
                },
                AssetSource {
                    kind: AssetKind::Sprite,
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
