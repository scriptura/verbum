// crates/assets/src/load.rs

//! Chargement tout-ou-rien, sans allocation (I5, I10, I12).
//!
//! Niveaux 1 et 2 (manifeste, liaisons), niveau 3 (premier décodage de chaque artefact,
//! résultat abandonné), niveau 4 (redécodage et dépôt) : le dépôt n'est jamais appelé si
//! l'un des niveaux 1 à 3 échoue.

use crate::manifest::{ArtifactIndex, BindingError, Manifest, ManifestError, SlotIndex};
use crate::sprite::{ReadError, Sprite};

/// Échec du chargement avant tout dépôt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// Manifeste non conforme (niveau 1).
    Manifest(ManifestError),
    /// Liaisons incohérentes (niveau 2).
    Binding(BindingError),
    /// Artefact non conforme (niveau 3).
    Artifact {
        index: ArtifactIndex,
        error: ReadError,
    },
}

/// Charge les artefacts désignés par le manifeste et les dépose, un appel par entrée.
///
/// `slot_count` est la capacité : la longueur du tableau statique de liaison de
/// l'application. `deposit` ne renvoie aucune erreur ; les slots visés doivent être vides
/// à l'entrée (précondition de l'appelant, que ce crate ne peut pas vérifier).
///
/// Les niveaux 1 à 3 s'achèvent avant le premier dépôt. Le niveau 4 redécode chaque
/// artefact : le lecteur est une fonction pure de ses octets, empruntés de manière
/// immuable pendant l'appel, donc la seconde lecture ne peut pas échouer si la première
/// a réussi. Cette garantie ne couvre pas une panique de `deposit`.
pub fn load(
    manifest: &[u8],
    artifacts: &[&[u8]],
    slot_count: usize,
    mut deposit: impl FnMut(SlotIndex, Sprite),
) -> Result<(), LoadError> {
    // Niveau 1, puis niveau 2.
    let manifest = Manifest::parse(manifest).map_err(LoadError::Manifest)?;
    manifest
        .check_bindings(artifacts.len(), slot_count)
        .map_err(LoadError::Binding)?;

    // Niveau 3 : premier décodage de chaque artefact, résultat abandonné.
    for entry in manifest.entries() {
        let index = entry.artifact_index();
        Sprite::read(artifact_bytes(artifacts, index))
            .map_err(|error| LoadError::Artifact { index, error })?;
    }

    // Niveau 4 : redécodage et dépôt, seulement si les niveaux 1 à 3 ont réussi.
    for entry in manifest.entries() {
        match Sprite::read(artifact_bytes(artifacts, entry.artifact_index())) {
            Ok(sprite) => deposit(entry.slot_index(), sprite),
            // Violation interne impossible (octets immuables, lecteur déterministe,
            // première lecture réussie) : active aussi en production, hors `LoadError`.
            Err(_) => unreachable!("seconde lecture en échec après une première réussie"),
        }
    }
    Ok(())
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
        fn record(&mut self, slot: SlotIndex, sprite: Sprite) {
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
            2,
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
            2,
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
            2,
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
            2,
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
            1,
            |slot, sprite| deposits.record(slot, sprite),
        );
        assert_eq!(
            result,
            Err(LoadError::Binding(BindingError::CapacityExceeded))
        );
        assert_eq!(deposits.calls, 0);

        let result = load(&MANIFEST, &[&SPRITE_32X16], 2, |slot, sprite| {
            deposits.record(slot, sprite)
        });
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
        let result = load(&empty, &[], 2, |slot, sprite| deposits.record(slot, sprite));
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
        let result = load(&single, &[&SPRITE_32X16], 2, |slot, sprite| {
            deposits.record(slot, sprite)
        });
        assert_eq!(result, Ok(()));
        assert_eq!(deposits.calls, 1);
        assert_eq!(deposits.slots[1], Some(sprite(32, 16)));
        assert_eq!(deposits.slots[0], None);
    }

    #[test]
    fn l4_an_empty_manifest_with_a_non_empty_collection_is_rejected() {
        let empty = [0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut deposits = Deposits::default();
        let result = load(&empty, &[&SPRITE_32X16], 2, |slot, sprite| {
            deposits.record(slot, sprite)
        });
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
            2,
            |slot, sprite| deposits.record(slot, sprite),
        )
        .unwrap();
        assert_eq!(deposits.slots[1], Some(first_pass[0]));
        assert_eq!(deposits.slots[0], Some(first_pass[1]));
    }
}
