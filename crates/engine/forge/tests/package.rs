// crates/engine/forge/tests/package.rs

//! Production de paquet `VPKG` par la Forge, en mémoire (V2-F : pf1 à pf7).
//!
//! pf8 (`compile_fail`) est porté par les doctests de `CompiledPackage`, un test
//! d'intégration ne pouvant exprimer un échec de compilation.
//!
//! Les octets attendus sont écrits à la main, indépendamment des constantes du layout ;
//! l'analyse des octets produits est faite ici par des indices littéraux, sans passer par
//! le lecteur d'Assets lorsqu'il s'agit de contrôler le layout lui-même.

use std::num::NonZeroU64;

use verbum_assets::{Asset, Format, PackageError, SlotIndex, Sprite, load, load_package};
use verbum_forge::{
    AssetKind, AssetSource, CompilePackageError, ForgeError, compile_assets, compile_package,
};

/// Quantum asymétrique : tout permutation d'octets de `q` change le paquet.
const Q: u64 = 0x0102_0304_0506_0708;
const Q_OTHER: u64 = 0x0807_0605_0403_0201;

fn a() -> AssetSource<'static> {
    AssetSource {
        kind: AssetKind::Sprite,
        key: "a",
        toml: "width = 32\nheight = 16\n",
        slot: SlotIndex::new(1),
    }
}

fn foobar() -> AssetSource<'static> {
    AssetSource {
        kind: AssetKind::Sprite,
        key: "foobar",
        toml: "width = 96\nheight = 48\n",
        slot: SlotIndex::new(0),
    }
}

fn package(sources: &[AssetSource<'_>], capacity: usize, q: u64) -> Vec<u8> {
    compile_package(sources, capacity, q)
        .expect("lot valide")
        .as_bytes()
        .to_vec()
}

/// Paquet du lot de référence, q = 10, écrit à la main (§ 3.4).
const REFERENCE_PACKAGE: [u8; 104] = [
    0x56, 0x50, 0x4B, 0x47, // VPKG
    0x01, 0x00, // version d'enveloppe 1
    0x02, 0x00, 0x00, 0x00, // n' = 2
    0x4C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // s_0 = 76
    0x5A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // s_1 = 90
    0x56, 0x4D, 0x41, 0x4E, // VMAN
    0x02, 0x00, // version du manifeste 2
    0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // q_ns = 10
    0x02, 0x00, 0x00, 0x00, // n = 2
    0x8C, 0xEC, 0x01, 0x86, 0x4C, 0xDC, 0x63, 0xAF, // id de « a »
    0x00, 0x00, 0x00, 0x00, // artifact_index 0
    0x01, 0x00, 0x00, 0x00, // slot_index 1
    0xE8, 0x67, 0x39, 0xF7, 0x71, 0x41, 0x94, 0x85, // id de « foobar »
    0x01, 0x00, 0x00, 0x00, // artifact_index 1
    0x00, 0x00, 0x00, 0x00, // slot_index 0
    0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x20, 0x00, 0x10,
    0x00, // 32×16
    0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x60, 0x00, 0x30,
    0x00, // 96×48
];

/// Paquet vide, q = `Q` : `n' = n = 0`, 28 octets.
const EMPTY_PACKAGE: [u8; 28] = [
    0x56, 0x50, 0x4B, 0x47, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // enveloppe, n' = 0
    0x56, 0x4D, 0x41, 0x4E, 0x02, 0x00, // VMAN, version 2
    0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // q_ns
    0x00, 0x00, 0x00, 0x00, // n = 0
];

/// Découpe un paquet en (offsets, manifeste, artefacts) par lecture littérale de l'en-tête.
fn split(bytes: &[u8]) -> (Vec<usize>, &[u8], Vec<&[u8]>) {
    let count = usize::try_from(u32::from_le_bytes(
        bytes[6..10].try_into().expect("4 octets"),
    ))
    .expect("n' tient dans usize");
    let table_end = 10 + 8 * count;
    let offsets: Vec<usize> = (0..count)
        .map(|i| {
            let start = 10 + 8 * i;
            usize::try_from(u64::from_le_bytes(
                bytes[start..start + 8].try_into().expect("8 octets"),
            ))
            .expect("offset représentable")
        })
        .collect();
    let first = offsets.first().copied().unwrap_or(bytes.len());
    let artifacts = offsets
        .iter()
        .enumerate()
        .map(|(i, start)| &bytes[*start..offsets.get(i + 1).copied().unwrap_or(bytes.len())])
        .collect();
    (offsets, &bytes[table_end..first], artifacts)
}

/// Dépôts d'un chargement de paquet, triés par slot.
fn deposits(
    bytes: &[u8],
    formats: &[Format],
) -> (Result<NonZeroU64, PackageError>, Vec<(u32, Sprite)>) {
    let mut out = Vec::new();
    let result = load_package(bytes, formats, |slot, asset| match asset {
        Asset::Sprite(sprite) => out.push((slot.get(), sprite)),
        other => panic!("asset inattendu : {other:?}"),
    });
    out.sort_by_key(|(slot, _)| *slot);
    (result, out)
}

#[test]
fn pf1_reference_set_yields_the_literal_package() {
    let bytes = package(&[a(), foobar()], 2, 10);
    assert_eq!(bytes, REFERENCE_PACKAGE.as_slice());
}

#[test]
fn pf2_round_trip_through_load_package_matches_v1_slots_and_q() {
    let sources = [a(), foobar()];
    let bytes = package(&sources, 2, Q);
    let formats = [Format::Sprite, Format::Sprite];

    let (result, packaged) = deposits(&bytes, &formats);
    assert_eq!(result.map(NonZeroU64::get), Ok(Q));
    assert_eq!(
        packaged,
        vec![
            (
                0,
                Sprite {
                    width: 96,
                    height: 48
                }
            ),
            (
                1,
                Sprite {
                    width: 32,
                    height: 16
                }
            ),
        ]
    );

    // Voie v1 sur le même lot : mêmes dépôts.
    let set = compile_assets(&sources, 2).expect("lot valide");
    let artifacts: Vec<&[u8]> = set.artifacts().iter().map(Vec::as_slice).collect();
    let mut legacy = Vec::new();
    load(
        set.manifest(),
        &artifacts,
        &formats,
        |slot, asset| match asset {
            Asset::Sprite(sprite) => legacy.push((slot.get(), sprite)),
            other => panic!("asset inattendu : {other:?}"),
        },
    )
    .expect("chargement v1");
    legacy.sort_by_key(|(slot, _)| *slot);
    assert_eq!(packaged, legacy);
}

#[test]
fn pf3_entries_and_artifacts_equal_those_of_compile_assets() {
    for sources in [vec![a(), foobar()], vec![foobar(), a()], vec![a()], vec![]] {
        let set = compile_assets(&sources, 2).expect("lot valide");
        let bytes = package(&sources, 2, Q);
        let (offsets, manifest, artifacts) = split(&bytes);

        // Manifeste v2 : en-tête de 18 octets (compteur identique), entrées de la v1.
        assert_eq!(manifest.len(), 18 + 16 * sources.len());
        assert_eq!(&manifest[14..18], &set.manifest()[6..10]);
        assert_eq!(&manifest[18..], &set.manifest()[10..]);
        // Artefacts : mêmes octets, même ordre.
        assert_eq!(artifacts.len(), set.artifacts().len());
        for (packaged, compiled) in artifacts.iter().zip(set.artifacts()) {
            assert_eq!(*packaged, compiled.as_slice());
        }
        // Offsets strictement croissants, contigus, jusqu'à la fin du paquet.
        assert_eq!(offsets.len(), sources.len());
        let total: usize = set.artifacts().iter().map(Vec::len).sum();
        assert_eq!(bytes.len(), 10 + 8 * sources.len() + manifest.len() + total);
    }
}

#[test]
fn pf4_permutation_permutes_indices_and_q_changes_exactly_eight_bytes() {
    let reference = package(&[a(), foobar()], 2, Q);
    let permuted = package(&[foobar(), a()], 2, Q);

    let (_, ref_manifest, ref_artifacts) = split(&reference);
    let (_, perm_manifest, perm_artifacts) = split(&permuted);
    // Entrées : (id, artifact_index, slot_index).
    let entry = |manifest: &[u8], i: usize| -> (Vec<u8>, u32, u32) {
        let start = 18 + 16 * i;
        (
            manifest[start..start + 8].to_vec(),
            u32::from_le_bytes(
                manifest[start + 8..start + 12]
                    .try_into()
                    .expect("4 octets"),
            ),
            u32::from_le_bytes(
                manifest[start + 12..start + 16]
                    .try_into()
                    .expect("4 octets"),
            ),
        )
    };
    let (id_a, index_a, slot_a) = entry(ref_manifest, 0);
    let (id_f, index_f, slot_f) = entry(ref_manifest, 1);
    assert_eq!((index_a, index_f), (0, 1));
    assert_eq!(entry(perm_manifest, 0), (id_f, 0, slot_f));
    assert_eq!(entry(perm_manifest, 1), (id_a, 1, slot_a));
    assert_eq!(perm_artifacts[0], ref_artifacts[1]);
    assert_eq!(perm_artifacts[1], ref_artifacts[0]);

    // q différent : même longueur, différence exactement aux 8 octets de q (32..40).
    let other = package(&[a(), foobar()], 2, Q_OTHER);
    assert_eq!(other.len(), reference.len());
    let differing: Vec<usize> = (0..reference.len())
        .filter(|i| reference[*i] != other[*i])
        .collect();
    assert_eq!(differing, (32..40).collect::<Vec<_>>());
    // Positions de q : littéral, `10 + 8·2 = 26`, puis `VMAN` (4) et version (2).
    assert_eq!(&reference[32..40], &Q.to_le_bytes());
    assert_eq!(&other[32..40], &Q_OTHER.to_le_bytes());
}

#[test]
fn pf5_zero_quantum_is_refused_before_any_compilation() {
    let zero = Err(CompilePackageError::Forge(ForgeError::ZeroValue("q")));
    assert_eq!(compile_package(&[a(), foobar()], 2, 0).map(|_| ()), zero);
    assert_eq!(compile_package(&[], 0, 0).map(|_| ()), zero);

    // Priorité sur une source invalide : q nul masque l'erreur de source.
    let invalid = AssetSource {
        kind: AssetKind::Sprite,
        key: "bad",
        toml: "width = 0\nheight = 4\n",
        slot: SlotIndex::new(0),
    };
    assert_eq!(compile_package(&[invalid], 1, 0).map(|_| ()), zero);
    // Même source avec q valide : l'erreur de source apparaît.
    assert_eq!(
        compile_package(&[invalid], 1, 1).map(|_| ()),
        Err(CompilePackageError::Forge(ForgeError::InvalidSource {
            entry: 0,
            cause: Box::new(ForgeError::ZeroValue("width")),
        }))
    );
    // q minimal valide.
    assert!(compile_package(&[a()], 2, 1).is_ok());
}

#[test]
fn pf6_invalid_batches_give_the_compile_assets_errors_wrapped() {
    let sprite = |key: &'static str, toml: &'static str, slot: u32| AssetSource {
        kind: AssetKind::Sprite,
        key,
        toml,
        slot: SlotIndex::new(slot),
    };
    let ok = "width = 4\nheight = 4\n";
    let batches: Vec<(Vec<AssetSource<'static>>, usize)> = vec![
        (vec![sprite("", ok, 0)], 1),
        (vec![sprite("x", ok, 0), sprite("x", ok, 1)], 2),
        (vec![sprite("x", ok, 5)], 2),
        (vec![sprite("x", ok, 0), sprite("y", ok, 0)], 2),
        (vec![sprite("x", "width = ", 0)], 1),
        (vec![sprite("x", "width = 4\nheight = 4\ndepth = 4", 0)], 1),
        (
            vec![sprite("x", ok, 0), sprite("y", "width = 0\nheight = 4", 1)],
            2,
        ),
    ];
    for (sources, capacity) in batches {
        let expected = compile_assets(&sources, capacity).map(|_| ());
        let error = expected.clone().expect_err("lot invalide");
        assert_eq!(
            compile_package(&sources, capacity, Q).map(|_| ()),
            Err(CompilePackageError::Forge(error)),
            "lot {sources:?}"
        );
    }
}

#[test]
fn pf7_empty_batch_is_28_bytes_and_loads_without_deposit() {
    let bytes = package(&[], 0, Q);
    assert_eq!(bytes, EMPTY_PACKAGE.as_slice());

    let (result, deposited) = deposits(&bytes, &[]);
    assert_eq!(result.map(NonZeroU64::get), Ok(Q));
    assert!(deposited.is_empty());
}
