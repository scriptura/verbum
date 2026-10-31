// crates/assets/src/load.rs

//! Chargement tout-ou-rien, sans allocation (I5, I10, I12).
//!
//! Niveau 1 (manifeste), niveau 2 (liaisons, puis format de chaque slot), niveau 3
//! (premier décodage de chaque artefact et, pour une carte, fermeture de sa référence ;
//! résultat abandonné), niveau 4 (même opération, puis dépôt) : le dépôt n'est jamais
//! appelé si l'un des niveaux 1 à 3 échoue.

use crate::format::{Asset, Decoded, Format, decode};
use crate::manifest::{ArtifactIndex, BindingError, Manifest, ManifestError, SlotIndex};
use crate::map::Map;
use crate::sprite::ReadError;

/// Échec du chargement avant tout dépôt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// Manifeste non conforme (niveau 1).
    Manifest(ManifestError),
    /// Liaisons incohérentes (niveau 2).
    Binding(BindingError),
    /// L'artefact lié au slot porte l'identité d'un format connu, différent du format que
    /// la table attend pour ce slot (niveau 2).
    SlotFormat { slot: SlotIndex },
    /// Artefact non conforme (niveau 3).
    Artifact {
        index: ArtifactIndex,
        error: ReadError,
    },
    /// Référence d'une carte non fermée (niveau 3).
    Reference {
        index: ArtifactIndex,
        error: ReferenceError,
    },
}

/// Cause d'une référence non fermée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceError {
    /// L'identifiant référencé ne figure pas au manifeste.
    Unresolved,
    /// L'entrée référencée existe, mais son slot n'attend pas un tileset (y compris la
    /// référence d'une carte à elle-même).
    WrongFormat,
}

/// Charge les artefacts désignés par le manifeste et les dépose, un appel par entrée.
///
/// `slot_formats` est la table statique de liaison de l'application : l'élément `s` est le
/// format attendu pour le slot `s`, et sa longueur est la capacité. `deposit` ne renvoie
/// aucune erreur ; les slots visés doivent être vides à l'entrée (précondition de
/// l'appelant, que ce crate ne peut pas vérifier).
///
/// Les niveaux 1 à 3 s'achèvent avant le premier dépôt. Le niveau 4 redécode chaque
/// artefact : le décodeur est une fonction pure de ses octets, empruntés de manière
/// immuable pendant l'appel, donc la seconde lecture ne peut pas échouer si la première
/// a réussi. Cette garantie ne couvre pas une panique de `deposit`.
pub fn load<'a>(
    manifest: &[u8],
    artifacts: &[&'a [u8]],
    slot_formats: &[Format],
    mut deposit: impl FnMut(SlotIndex, Asset<'a>),
) -> Result<(), LoadError> {
    // Niveau 1, puis niveau 2 (liaisons, puis format de chaque slot).
    let manifest = Manifest::parse(manifest).map_err(LoadError::Manifest)?;
    manifest
        .check_bindings(artifacts.len(), slot_formats.len())
        .map_err(LoadError::Binding)?;
    for entry in manifest.entries() {
        let expected = expected_format(slot_formats, entry.slot_index());
        let bytes = artifact_bytes(artifacts, entry.artifact_index());
        // Seule une identité connue, différente du format attendu, est rejetée ici ; une
        // identité inconnue ou un préfixe incomplet reste au niveau 3.
        if Format::of_identity(bytes).is_some_and(|found| found != expected) {
            return Err(LoadError::SlotFormat {
                slot: entry.slot_index(),
            });
        }
    }

    // Niveau 3 : premier décodage de chaque artefact, fermeture des références des
    // cartes, résultat abandonné.
    for entry in manifest.entries() {
        let index = entry.artifact_index();
        let expected = expected_format(slot_formats, entry.slot_index());
        materialize(
            &manifest,
            slot_formats,
            expected,
            artifact_bytes(artifacts, index),
        )
        .map_err(|failure| failure.at(index))?;
    }

    // Niveau 4 : redécodage et dépôt, seulement si les niveaux 1 à 3 ont réussi.
    for entry in manifest.entries() {
        let expected = expected_format(slot_formats, entry.slot_index());
        let bytes = artifact_bytes(artifacts, entry.artifact_index());
        match materialize(&manifest, slot_formats, expected, bytes) {
            Ok(asset) => deposit(entry.slot_index(), asset),
            // Violation interne impossible (octets immuables, décodeur déterministe,
            // première lecture réussie) : active aussi en production, hors `LoadError`.
            Err(_) => unreachable!("seconde lecture en échec après une première réussie"),
        }
    }
    Ok(())
}

/// Échec de la matérialisation d'un artefact, avant d'être rattaché à son index.
#[derive(Clone, Copy, Debug)]
enum Failure {
    Read(ReadError),
    Reference(ReferenceError),
}

impl Failure {
    fn at(self, index: ArtifactIndex) -> LoadError {
        match self {
            Self::Read(error) => LoadError::Artifact { index, error },
            Self::Reference(error) => LoadError::Reference { index, error },
        }
    }
}

/// Décodage unique des niveaux 3 et 4, suivi pour une carte de la fermeture de sa
/// référence : l'identifiant du tileset est cherché au manifeste, l'entrée trouvée doit
/// exister et son slot doit attendre un tileset ; la valeur porte le **slot** du référent.
/// Fonction pure des octets et du manifeste : aucune allocation, aucune référence suivie.
fn materialize<'a>(
    manifest: &Manifest<'_>,
    slot_formats: &[Format],
    expected: Format,
    bytes: &'a [u8],
) -> Result<Asset<'a>, Failure> {
    match decode(expected, bytes).map_err(Failure::Read)? {
        Decoded::Ready(asset) => Ok(asset),
        Decoded::Map(parts) => {
            let referent = manifest
                .find(parts.tileset)
                .ok_or(Failure::Reference(ReferenceError::Unresolved))?;
            if expected_format(slot_formats, referent.slot_index()) != Format::Tileset {
                return Err(Failure::Reference(ReferenceError::WrongFormat));
            }
            Ok(Asset::Map(Map::new(referent.slot_index(), parts)))
        }
    }
}

/// Format attendu pour le slot ; l'indice a été validé par `check_bindings`.
fn expected_format(slot_formats: &[Format], slot: SlotIndex) -> Format {
    match usize::try_from(slot.get())
        .ok()
        .and_then(|position| slot_formats.get(position))
    {
        Some(format) => *format,
        None => unreachable!("slot_index validé par check_bindings"),
    }
}

/// Octets de l'artefact désigné ; l'indice a été validé par `check_bindings`.
fn artifact_bytes<'b>(artifacts: &[&'b [u8]], index: ArtifactIndex) -> &'b [u8] {
    match usize::try_from(index.get())
        .ok()
        .and_then(|position| artifacts.get(position))
    {
        Some(bytes) => bytes,
        None => unreachable!("artifact_index validé par check_bindings"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::AssetId;
    use crate::sprite::Sprite;

    /// Manifeste littéral à deux entrées, écrit à la main :
    /// entrée 0 : id 0x1122334455667788, artefact 0, slot 1 ;
    /// entrée 1 : id 0x99AABBCCDDEEFF00, artefact 1, slot 0.
    const MANIFEST: [u8; 42] = [
        0x56, 0x4D, 0x41, 0x4E, // signature
        0x01, 0x00, // version
        0x02, 0x00, 0x00, 0x00, // n = 2
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id 0
        0x00, 0x00, 0x00, 0x00, // artifact_index 0
        0x01, 0x00, 0x00, 0x00, // slot_index 1
        0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id 1
        0x01, 0x00, 0x00, 0x00, // artifact_index 1
        0x00, 0x00, 0x00, 0x00, // slot_index 0
    ];

    /// Artefact 32 × 16 (0x0020 × 0x0010).
    const SPRITE_32X16: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x20, 0x00, 0x10, 0x00,
    ];

    /// Artefact 96 × 48 (0x0060 × 0x0030).
    const SPRITE_96X48: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x60, 0x00, 0x30, 0x00,
    ];

    const ID_0: u64 = 0x1122_3344_5566_7788;
    const ID_1: u64 = 0x99AA_BBCC_DDEE_FF00;

    /// Dépôt de test : un emplacement par slot, un compteur d'appels ; un second dépôt
    /// dans le même slot est un échec du test.
    #[derive(Default)]
    struct Deposits {
        calls: usize,
        slots: [Option<Sprite>; 4],
    }

    impl Deposits {
        fn record(&mut self, slot: SlotIndex, asset: Asset) {
            let Asset::Sprite(sprite) = asset else {
                panic!("variante inattendue : un sprite était attendu");
            };
            self.calls += 1;
            let cell = &mut self.slots[slot.get() as usize];
            assert!(cell.is_none(), "slot {} déposé deux fois", slot.get());
            *cell = Some(sprite);
        }
    }

    fn sprite(width: u16, height: u16) -> Sprite {
        Sprite { width, height }
    }

    #[test]
    fn l1_nominal_load_deposits_each_artifact_in_its_slot_exactly_once() {
        let mut deposits = Deposits::default();
        let result = load(
            &MANIFEST,
            &[&SPRITE_32X16, &SPRITE_96X48],
            &[Format::Sprite; 2],
            |slot, sprite| deposits.record(slot, sprite),
        );
        assert_eq!(result, Ok(()));
        assert_eq!(deposits.calls, 2);
        assert_eq!(deposits.slots[1], Some(sprite(32, 16)));
        assert_eq!(deposits.slots[0], Some(sprite(96, 48)));
        assert_eq!(deposits.slots[2], None);
    }

    #[test]
    fn l2_lookup_by_id_leads_to_the_right_artifact_and_slot() {
        let manifest = Manifest::parse(&MANIFEST).expect("manifeste conforme");
        let artifacts: [&[u8]; 2] = [&SPRITE_32X16, &SPRITE_96X48];

        let first = manifest.find(AssetId::from_raw(ID_0)).expect("id 0");
        assert_eq!(first.artifact_index().get(), 0);
        assert_eq!(first.slot_index().get(), 1);
        let bytes = artifacts[first.artifact_index().get() as usize];
        assert_eq!(Sprite::read(bytes), Ok(sprite(32, 16)));

        let second = manifest.find(AssetId::from_raw(ID_1)).expect("id 1");
        assert_eq!(second.artifact_index().get(), 1);
        assert_eq!(second.slot_index().get(), 0);
        let bytes = artifacts[second.artifact_index().get() as usize];
        assert_eq!(Sprite::read(bytes), Ok(sprite(96, 48)));
    }

    #[test]
    fn l3_a_truncated_artifact_at_index_1_leaves_every_slot_untouched() {
        let mut deposits = Deposits::default();
        let result = load(
            &MANIFEST,
            &[&SPRITE_32X16, &SPRITE_96X48[..13]],
            &[Format::Sprite; 2],
            |slot, sprite| deposits.record(slot, sprite),
        );
        let Err(LoadError::Artifact { index, error }) = result else {
            panic!("attendu : erreur d'artefact, obtenu {result:?}");
        };
        assert_eq!(index.get(), 1);
        assert_eq!(error, ReadError::Truncated);
        assert_eq!(deposits.calls, 0);
    }

    #[test]
    fn l3_a_truncated_artifact_at_index_0_leaves_every_slot_untouched() {
        let mut deposits = Deposits::default();
        let result = load(
            &MANIFEST,
            &[&SPRITE_32X16[..5], &SPRITE_96X48],
            &[Format::Sprite; 2],
            |slot, sprite| deposits.record(slot, sprite),
        );
        let Err(LoadError::Artifact { index, error }) = result else {
            panic!("attendu : erreur d'artefact, obtenu {result:?}");
        };
        assert_eq!(index.get(), 0);
        assert_eq!(error, ReadError::Truncated);
        assert_eq!(deposits.calls, 0);
    }

    #[test]
    fn l3_an_invalid_manifest_at_level_1_leaves_every_slot_untouched() {
        let mut altered = MANIFEST;
        altered[0] ^= 0xFF;
        let mut deposits = Deposits::default();
        let result = load(
            &altered,
            &[&SPRITE_32X16, &SPRITE_96X48],
            &[Format::Sprite; 2],
            |slot, sprite| deposits.record(slot, sprite),
        );
        assert_eq!(result, Err(LoadError::Manifest(ManifestError::Signature)));
        assert_eq!(deposits.calls, 0);
    }

    #[test]
    fn l3_invalid_bindings_at_level_2_leave_every_slot_untouched() {
        let mut deposits = Deposits::default();
        let result = load(
            &MANIFEST,
            &[&SPRITE_32X16, &SPRITE_96X48],
            &[Format::Sprite; 1],
            |slot, sprite| deposits.record(slot, sprite),
        );
        assert_eq!(
            result,
            Err(LoadError::Binding(BindingError::CapacityExceeded))
        );
        assert_eq!(deposits.calls, 0);

        let result = load(
            &MANIFEST,
            &[&SPRITE_32X16],
            &[Format::Sprite; 2],
            |slot, sprite| deposits.record(slot, sprite),
        );
        assert_eq!(
            result,
            Err(LoadError::Binding(BindingError::EntryCountMismatch))
        );
        assert_eq!(deposits.calls, 0);
    }

    #[test]
    fn l4_empty_manifest_and_empty_collection_load_without_deposit() {
        let empty = [
            0x56, 0x4D, 0x41, 0x4E, // signature
            0x01, 0x00, // version
            0x00, 0x00, 0x00, 0x00, // n = 0
        ];
        let mut deposits = Deposits::default();
        let result = load(&empty, &[], &[Format::Sprite; 2], |slot, sprite| {
            deposits.record(slot, sprite)
        });
        assert_eq!(result, Ok(()));
        assert_eq!(deposits.calls, 0);
    }

    #[test]
    fn l4_a_capacity_above_the_asset_count_leaves_the_other_slot_unsolicited() {
        // Une seule entrée : id 0x1122334455667788, artefact 0, slot 1.
        let single = [
            0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, // en-tête, n = 1
            0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id
            0x00, 0x00, 0x00, 0x00, // artifact_index 0
            0x01, 0x00, 0x00, 0x00, // slot_index 1
        ];
        let mut deposits = Deposits::default();
        let result = load(
            &single,
            &[&SPRITE_32X16],
            &[Format::Sprite; 2],
            |slot, sprite| deposits.record(slot, sprite),
        );
        assert_eq!(result, Ok(()));
        assert_eq!(deposits.calls, 1);
        assert_eq!(deposits.slots[1], Some(sprite(32, 16)));
        assert_eq!(deposits.slots[0], None);
    }

    #[test]
    fn l4_an_empty_manifest_with_a_non_empty_collection_is_rejected() {
        let empty = [0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut deposits = Deposits::default();
        let result = load(
            &empty,
            &[&SPRITE_32X16],
            &[Format::Sprite; 2],
            |slot, sprite| deposits.record(slot, sprite),
        );
        assert_eq!(
            result,
            Err(LoadError::Binding(BindingError::EntryCountMismatch))
        );
        assert_eq!(deposits.calls, 0);
    }

    #[test]
    fn l5_the_reader_is_deterministic_and_the_deposits_equal_the_first_pass_values() {
        for bytes in [
            &SPRITE_32X16[..],
            &SPRITE_96X48[..],
            &SPRITE_32X16[..13],
            &[][..],
        ] {
            assert_eq!(Sprite::read(bytes), Sprite::read(bytes));
        }

        // Valeurs de la passe 3, relues directement, puis valeurs reçues en passe 4.
        let first_pass = [
            Sprite::read(&SPRITE_32X16).unwrap(),
            Sprite::read(&SPRITE_96X48).unwrap(),
        ];
        assert_eq!(first_pass, [sprite(32, 16), sprite(96, 48)]);

        let mut deposits = Deposits::default();
        load(
            &MANIFEST,
            &[&SPRITE_32X16, &SPRITE_96X48],
            &[Format::Sprite; 2],
            |slot, sprite| deposits.record(slot, sprite),
        )
        .unwrap();
        assert_eq!(deposits.slots[1], Some(first_pass[0]));
        assert_eq!(deposits.slots[0], Some(first_pass[1]));
    }

    /// Manifeste d'une seule entrée : id 0x1122334455667788, artefact 0, slot 1.
    const SINGLE: [u8; 26] = [
        0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, // en-tête, n = 1
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id
        0x00, 0x00, 0x00, 0x00, // artifact_index 0
        0x01, 0x00, 0x00, 0x00, // slot_index 1
    ];

    #[test]
    fn sf2_an_artifact_of_unknown_identity_bound_to_a_sprite_slot_is_rejected_by_level_3() {
        // Le texte d'un `sprite.toml` n'est pas un artefact (I1) : identité inconnue.
        let source = b"width = 258\nheight = 772\n";
        assert!(source.len() >= crate::sprite::ARTIFACT_LEN);
        let mut deposits = Deposits::default();
        let result = load(&SINGLE, &[source], &[Format::Sprite; 2], |slot, asset| {
            deposits.record(slot, asset)
        });
        let Err(LoadError::Artifact { index, error }) = result else {
            panic!("attendu : erreur d'artefact, obtenu {result:?}");
        };
        assert_eq!(index.get(), 0);
        assert_eq!(error, ReadError::FormatIdentity);
        assert_eq!(deposits.calls, 0);
    }

    #[test]
    fn tb1_the_table_length_is_the_capacity() {
        let mut deposits = Deposits::default();
        let result = load(
            &MANIFEST,
            &[&SPRITE_32X16, &SPRITE_96X48],
            &[Format::Sprite; 1],
            |slot, asset| deposits.record(slot, asset),
        );
        assert_eq!(
            result,
            Err(LoadError::Binding(BindingError::CapacityExceeded))
        );
        assert_eq!(deposits.calls, 0);

        let result = load(
            &MANIFEST,
            &[&SPRITE_32X16, &SPRITE_96X48],
            &[Format::Sprite; 2],
            |slot, asset| deposits.record(slot, asset),
        );
        assert_eq!(result, Ok(()));
        assert_eq!(deposits.calls, 2);
    }

    #[test]
    fn ld1_deposit_receives_a_sprite_asset_independent_of_the_loaded_bytes() {
        let mut first = SPRITE_32X16;
        let mut second = SPRITE_96X48;
        let mut received: [Option<(u32, Sprite)>; 2] = [None, None];
        let mut calls = 0;
        let result = load(
            &MANIFEST,
            &[&first, &second],
            &[Format::Sprite; 2],
            |slot, asset| {
                // `Asset<'a>` emprunte désormais les octets chargés : seule la valeur `Copy`
                // du sprite est conservée, ce qui exprime l'indépendance vis-à-vis des octets.
                let Asset::Sprite(value) = asset else {
                    panic!("variante inattendue : un sprite était attendu");
                };
                received[calls] = Some((slot.get(), value));
                calls += 1;
            },
        );
        assert_eq!(result, Ok(()));
        assert_eq!(
            received,
            [Some((1, sprite(32, 16))), Some((0, sprite(96, 48))),]
        );

        // Les valeurs déposées sont des copies : détruire les octets sources ne les change pas.
        first.fill(0xFF);
        second.fill(0xFF);
        assert_eq!(
            received,
            [Some((1, sprite(32, 16))), Some((0, sprite(96, 48))),]
        );
    }

    /// Artefact tileset littéral : `VTLS`, version 1, charge utile 8, tuiles de 16 × 8,
    /// 4 colonnes, 12 tuiles. Seule son identité et sa conformité importent ici.
    const TILESET_16X8: [u8; 18] = [
        0x56, 0x54, 0x4C, 0x53, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x08, 0x00, 0x04,
        0x00, 0x0C, 0x00,
    ];

    /// Compteur de dépôts : les tests de format de slot n'attendent aucun dépôt.
    #[derive(Default)]
    struct Journal {
        calls: usize,
    }

    impl Journal {
        fn record(&mut self, _slot: SlotIndex, _asset: Asset) {
            self.calls += 1;
        }
    }

    #[test]
    fn sf1_an_artifact_of_the_other_known_format_is_rejected_before_any_deposit() {
        // Un tileset lié à un slot `Sprite` (artefact 0 → slot 1).
        let mut journal = Journal::default();
        let result = load(
            &MANIFEST,
            &[&TILESET_16X8, &SPRITE_96X48],
            &[Format::Tileset, Format::Sprite],
            |slot, asset| journal.record(slot, asset),
        );
        assert_eq!(
            result,
            Err(LoadError::SlotFormat {
                slot: SlotIndex::new(1)
            })
        );
        assert_eq!(journal.calls, 0);

        // Réciproque : un sprite lié à un slot `Tileset` (artefact 0 → slot 1).
        let result = load(
            &MANIFEST,
            &[&SPRITE_32X16, &TILESET_16X8],
            &[Format::Sprite, Format::Tileset],
            |slot, asset| journal.record(slot, asset),
        );
        assert_eq!(
            result,
            Err(LoadError::SlotFormat {
                slot: SlotIndex::new(1)
            })
        );
        assert_eq!(journal.calls, 0);
    }

    #[test]
    fn sf3_a_binding_error_takes_precedence_over_a_slot_format_error() {
        // Table de longueur 1 : le slot 1 dépasse la capacité (niveau 2, liaisons) alors que
        // l'artefact 0 est un tileset lié à un slot de format différent.
        let mut journal = Journal::default();
        let result = load(
            &MANIFEST,
            &[&TILESET_16X8, &SPRITE_96X48],
            &[Format::Sprite],
            |slot, asset| journal.record(slot, asset),
        );
        assert_eq!(
            result,
            Err(LoadError::Binding(BindingError::CapacityExceeded))
        );
        assert_eq!(journal.calls, 0);
    }

    #[test]
    fn sf4_an_unknown_identity_stays_an_artifact_error_with_two_formats() {
        let source = b"width = 258\nheight = 772\n";
        assert!(source.len() >= crate::header::PREFIX_LEN);
        for table in [
            [Format::Tileset, Format::Sprite],
            [Format::Sprite, Format::Tileset],
        ] {
            let mut journal = Journal::default();
            let result = load(&SINGLE, &[source], &table, |slot, asset| {
                journal.record(slot, asset)
            });
            let Err(LoadError::Artifact { index, error }) = result else {
                panic!("attendu : erreur d'artefact, obtenu {result:?}");
            };
            assert_eq!(index.get(), 0);
            assert_eq!(error, ReadError::FormatIdentity);
            assert_eq!(journal.calls, 0);
        }
    }

    #[test]
    fn sf5_an_incomplete_prefix_with_another_known_identity_keeps_the_stage_two_error() {
        // Huit octets : identité `VTLS` complète, mais préfixe de 10 octets incomplet. Le
        // slot attend un sprite : c'est le décodeur du sprite qui répond `Truncated`.
        let truncated = &TILESET_16X8[..8];
        let mut journal = Journal::default();
        let result = load(
            &SINGLE,
            &[truncated],
            &[Format::Sprite; 2],
            |slot, asset| journal.record(slot, asset),
        );
        let Err(LoadError::Artifact { index, error }) = result else {
            panic!("attendu : erreur d'artefact, obtenu {result:?}");
        };
        assert_eq!(index.get(), 0);
        assert_eq!(error, ReadError::Truncated);
        assert_eq!(journal.calls, 0);
    }

    /// Manifeste littéral à deux entrées, `artifact_index = slot_index` :
    /// entrée 0 : artefact 0, slot 0 ; entrée 1 : artefact 1, slot 1.
    const IDENTITY_MANIFEST: [u8; 42] = [
        0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // en-tête, n = 2
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id 0
        0x00, 0x00, 0x00, 0x00, // artifact_index 0
        0x00, 0x00, 0x00, 0x00, // slot_index 0
        0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id 1
        0x01, 0x00, 0x00, 0x00, // artifact_index 1
        0x01, 0x00, 0x00, 0x00, // slot_index 1
    ];

    #[test]
    fn sf6_the_slot_format_check_takes_precedence_over_a_level_three_error() {
        // Slot 0 attend un tileset et reçoit un sprite valide (identité connue `VSPR`) ;
        // slot 1 attend un sprite et reçoit un artefact tronqué à 13 octets (préfixe complet,
        // identité du format attendu). Le niveau 2 s'achève avant le niveau 3 : c'est
        // `SlotFormat` pour le slot 0, non la troncature du second artefact.
        let truncated = &SPRITE_96X48[..13];
        let mut journal = Journal::default();
        let result = load(
            &IDENTITY_MANIFEST,
            &[&SPRITE_32X16, truncated],
            &[Format::Tileset, Format::Sprite],
            |slot, asset| journal.record(slot, asset),
        );
        assert_eq!(
            result,
            Err(LoadError::SlotFormat {
                slot: SlotIndex::new(0)
            })
        );
        assert_eq!(journal.calls, 0);
    }

    // ----- U3 : références des cartes (RF-1 à RF-6) -------------------------------------

    const TILE_ID: u64 = 0x1122_3344_5566_7788;
    const MAP_A_ID: u64 = 0x99AA_BBCC_DDEE_FF00;
    const SPRITE_ID: u64 = 0x0102_0304_0506_0708;
    const MAP_B_ID: u64 = 0x0A0B_0C0D_0E0F_1011;
    const UNKNOWN_ID: u64 = 0x1111_1111_1111_1111;

    /// Carte 3 × 2 littérale (identité `VMAP`, charge utile 24) référençant `TILE_ID` ;
    /// indices 0x0001, 0x0203, 0x0405, 0x0607, 0x0809, 0x0A0B.
    const MAP_BYTES: [u8; 34] = [
        0x56, 0x4D, 0x41, 0x50, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, // préfixe, charge 24
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // identifiant du tileset
        0x03, 0x00, 0x02, 0x00, // 3 × 2
        0x01, 0x00, 0x03, 0x02, 0x05, 0x04, 0x07, 0x06, 0x09, 0x08, 0x0B, 0x0A,
    ];

    /// `MAP_BYTES` référençant `id`.
    fn map_referencing(id: u64) -> [u8; 34] {
        let mut bytes = MAP_BYTES;
        bytes[10..18].copy_from_slice(&id.to_le_bytes());
        bytes
    }

    /// Manifeste littéral à quatre entrées, écrit à la main :
    /// tileset (artefact 0, slot 2) ; carte A (artefact 1, slot 0) ; sprite (artefact 2,
    /// slot 1) ; carte B (artefact 3, slot 3). Le référent des cartes a donc
    /// `artifact_index` 0 et `slot_index` 2.
    const FOUR: [u8; 74] = [
        0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, // en-tête, n = 4
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id du tileset
        0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, // artefact 0, slot 2
        0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id de la carte A
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // artefact 1, slot 0
        0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // id du sprite
        0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // artefact 2, slot 1
        0x11, 0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, // id de la carte B
        0x03, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, // artefact 3, slot 3
    ];

    const FOUR_TABLE: [Format; 4] = [Format::Map, Format::Sprite, Format::Tileset, Format::Map];

    /// Dépôts d'un lot : compteur et cartes reçues par slot.
    #[derive(Default)]
    struct Maps<'a> {
        calls: usize,
        maps: [Option<Map<'a>>; 4],
    }

    impl<'a> Maps<'a> {
        fn record(&mut self, slot: SlotIndex, asset: Asset<'a>) {
            self.calls += 1;
            if let Asset::Map(map) = asset {
                self.maps[slot.get() as usize] = Some(map);
            }
        }
    }

    fn load_four<'a>(
        map_a: &'a [u8],
        map_b: &'a [u8],
        maps: &mut Maps<'a>,
    ) -> Result<(), LoadError> {
        load(
            &FOUR,
            &[&TILESET_16X8, map_a, &SPRITE_32X16, map_b],
            &FOUR_TABLE,
            |slot, asset| maps.record(slot, asset),
        )
    }

    /// Vérifie une erreur de référence : index de l'artefact (comparé par `get`) et cause.
    fn assert_reference(
        result: Result<(), LoadError>,
        expected_index: u32,
        expected: ReferenceError,
    ) {
        let Err(LoadError::Reference { index, error }) = result else {
            panic!("attendu : erreur de référence, obtenu {result:?}");
        };
        assert_eq!(index.get(), expected_index);
        assert_eq!(error, expected);
    }

    #[test]
    fn rf1_an_unresolved_reference_is_rejected_before_any_deposit() {
        let valid = map_referencing(TILE_ID);
        let dangling = map_referencing(UNKNOWN_ID);
        let mut maps = Maps::default();
        assert_reference(
            load_four(&dangling, &valid, &mut maps),
            1,
            ReferenceError::Unresolved,
        );
        assert_eq!(maps.calls, 0);
    }

    #[test]
    fn rf2_a_referent_of_another_format_is_rejected_before_any_deposit() {
        let valid = map_referencing(TILE_ID);
        // Une carte qui référence un sprite, puis une carte qui référence une carte.
        for id in [SPRITE_ID, MAP_B_ID] {
            let wrong = map_referencing(id);
            let mut maps = Maps::default();
            assert_reference(
                load_four(&wrong, &valid, &mut maps),
                1,
                ReferenceError::WrongFormat,
            );
            assert_eq!(maps.calls, 0);
        }
    }

    #[test]
    fn rf3_a_self_reference_is_rejected_as_a_wrong_format() {
        let valid = map_referencing(TILE_ID);
        let selfish = map_referencing(MAP_A_ID);
        let mut maps = Maps::default();
        assert_reference(
            load_four(&selfish, &valid, &mut maps),
            1,
            ReferenceError::WrongFormat,
        );
        assert_eq!(maps.calls, 0);
    }

    /// Lot minimal à `artifact_index = slot_index` : carte (artefact 0, slot 0) puis
    /// tileset (artefact 1, slot 1). Le référent a donc le même index d'artefact et de
    /// slot, et ce slot n'est pas 0.
    const PAIR: [u8; 42] = [
        0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // en-tête, n = 2
        0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id de la carte
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // artefact 0, slot 0
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id du tileset
        0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // artefact 1, slot 1
    ];

    #[test]
    fn rf4_a_valid_reference_yields_a_map_carrying_the_referent_slot() {
        let map_bytes = map_referencing(TILE_ID);
        let mut maps = Maps::default();
        let result = load(
            &PAIR,
            &[&map_bytes, &TILESET_16X8],
            &[Format::Map, Format::Tileset],
            |slot, asset| maps.record(slot, asset),
        );
        assert_eq!(result, Ok(()));
        assert_eq!(maps.calls, 2);
        let map = maps.maps[0].expect("carte déposée dans le slot 0");
        assert_eq!(map.tileset(), SlotIndex::new(1));
        assert_eq!((map.width(), map.height()), (3, 2));
        assert_eq!(map.tile(2, 1), Some(0x0A0B));
    }

    #[test]
    fn rf5_the_resolved_index_is_the_slot_not_the_artifact_of_the_referent() {
        // Référent : artefact 0, slot 2.
        let map_bytes = map_referencing(TILE_ID);
        let mut maps = Maps::default();
        assert_eq!(load_four(&map_bytes, &map_bytes, &mut maps), Ok(()));
        assert_eq!(maps.calls, 4);
        for slot in [0, 3] {
            let map = maps.maps[slot].expect("carte déposée");
            assert_eq!(map.tileset(), SlotIndex::new(2), "slot de la carte {slot}");
            assert_ne!(map.tileset().get(), 0, "ce n'est pas l'index d'artefact");
        }
    }

    #[test]
    fn rf6_an_invalid_reference_after_valid_artifacts_leaves_no_deposit() {
        // Le tileset, la carte A et le sprite sont valides et précèdent la carte B.
        let valid = map_referencing(TILE_ID);
        let dangling = map_referencing(UNKNOWN_ID);
        let mut maps = Maps::default();
        assert_reference(
            load_four(&valid, &dangling, &mut maps),
            3,
            ReferenceError::Unresolved,
        );
        assert_eq!(maps.calls, 0);
    }
}
