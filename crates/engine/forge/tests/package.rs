// crates/engine/forge/tests/package.rs

//! Production de paquet `VPKG` par la Forge, en mémoire (V2-F : pf1 à pf7, V2-Z : pf9).
//!
//! pf8 (`compile_fail`) est porté par les doctests de `CompiledPackage`, un test
//! d'intégration ne pouvant exprimer un échec de compilation.
//!
//! Les octets attendus sont écrits à la main, indépendamment des constantes du layout ;
//! l'analyse des octets produits est faite ici par des indices littéraux, sans passer par
//! le lecteur d'Assets lorsqu'il s'agit de contrôler le layout lui-même.

use std::num::NonZeroU64;

use verbum_assets::{
    Asset, Font, Format, Glyph, Map, PackageError, Script, SlotIndex, Sprite, Tileset, load,
    load_package, manifest_v2, package as envelope,
};
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

// ---------------------------------------------------------------------------------------
// pf9 : lot hétérogène des cinq formats, `compile_package` → `load_package` (V2-Z)
// ---------------------------------------------------------------------------------------

const TILES_TOML: &str = "tile_width = 16\ntile_height = 8\ncolumns = 4\ncount = 12\n";

/// Carte 3 × 2 référençant la clé « a » (le tileset).
const MAP_TOML: &str = "tileset = \"a\"\nwidth = 3\nheight = 2\ncells = [0, 1, 2, 3, 4, 11]\n";

/// Script de référence de l'étape 6 : sept pas, 68 octets.
const SCRIPT_A_TOML: &str = "[[step]]\nop = \"set\"\nregister = 0\nvalue = 3\n\
    [[step]]\nlabel = \"loop\"\nop = \"emit\"\nchannel = 1\nvalue = 10\n\
    [[step]]\nop = \"wait\"\nticks = 3\n\
    [[step]]\nop = \"add\"\nregister = 0\nvalue = -1\n\
    [[step]]\nop = \"jnz\"\nregister = 0\ntarget = \"loop\"\n\
    [[step]]\nop = \"emit\"\nchannel = 2\nvalue = 99\n\
    [[step]]\nop = \"halt\"\n";

/// Second script : quatre pas, 44 octets.
const SCRIPT_B_TOML: &str = "[[step]]\nop = \"emit\"\nchannel = 3\nvalue = 7\n\
    [[step]]\nop = \"wait\"\nticks = 1\n\
    [[step]]\nop = \"emit\"\nchannel = 3\nvalue = 8\n\
    [[step]]\nop = \"halt\"\n";

/// Police de trois glyphes, 38 octets.
const FONT_TOML: &str = "line_height = 2571\n\
    [[glyph]]\ncodepoint = 65\nwidth = 5\nadvance = 6\n\
    [[glyph]]\ncodepoint = 233\nwidth = 7\nadvance = 8\n\
    [[glyph]]\ncodepoint = 128512\nwidth = 0\nadvance = 9\n";

const FONT_GLYPHS: [Glyph; 3] = [
    Glyph {
        codepoint: 0x41,
        width: 5,
        advance: 6,
    },
    Glyph {
        codepoint: 0xE9,
        width: 7,
        advance: 8,
    },
    Glyph {
        codepoint: 0x1_F600,
        width: 0,
        advance: 9,
    },
];

fn source(
    kind: AssetKind,
    key: &'static str,
    toml: &'static str,
    slot: u32,
) -> AssetSource<'static> {
    AssetSource {
        kind,
        key,
        toml,
        slot: SlotIndex::new(slot),
    }
}

/// Lot hétérogène. Ordre des artefacts : tileset (18 octets, slot 2), carte (34, slot 5),
/// script « intro » (68, slot 4), sprite (14, slot 0), script « outro » (44, slot 1),
/// police (38, slot 3). Les six longueurs sont distinctes et `artifact_index ≠ slot_index`
/// pour chaque entrée.
fn hetero_sources() -> [AssetSource<'static>; 6] {
    [
        source(AssetKind::Tileset, "a", TILES_TOML, 2),
        source(AssetKind::Map, "ground", MAP_TOML, 5),
        source(AssetKind::Script, "intro", SCRIPT_A_TOML, 4),
        source(AssetKind::Sprite, "hero", "width = 96\nheight = 48\n", 0),
        source(AssetKind::Script, "outro", SCRIPT_B_TOML, 1),
        source(AssetKind::Font, "ui", FONT_TOML, 3),
    ]
}

/// Table des formats de slot (capacité 6) : 0 sprite, 1 script, 2 tileset, 3 police,
/// 4 script, 5 carte.
const HETERO_TABLE: [Format; 6] = [
    Format::Sprite,
    Format::Script,
    Format::Tileset,
    Format::Font,
    Format::Script,
    Format::Map,
];

/// Slot de chaque entrée, dans l'ordre des artefacts.
const HETERO_SLOTS: [u32; 6] = [2, 5, 4, 0, 1, 3];

/// Longueurs attendues des artefacts, écrites à la main, dans l'ordre des artefacts.
const HETERO_LENGTHS: [usize; 6] = [18, 34, 68, 14, 44, 38];

/// Offsets absolus attendus, écrits à la main : `T = 10 + 8·6 = 58`, manifeste
/// `18 + 16·6 = 114`, `s_0 = 172`, puis cumul des longueurs ; fin du paquet à 388.
const HETERO_OFFSETS: [u64; 6] = [172, 190, 224, 292, 306, 350];
const HETERO_TOTAL: usize = 388;

/// Dépôts typés : conserve chacune des cinq variantes avec le slot reçu. Un second dépôt
/// d'une variante à emplacement unique est un échec du test ; les scripts (deux) sont
/// conservés dans l'ordre de dépôt.
#[derive(Debug, Default, PartialEq)]
struct Typed<'a> {
    calls: usize,
    sprite: Option<(u32, Sprite)>,
    tileset: Option<(u32, Tileset)>,
    map: Option<(u32, Map<'a>)>,
    font: Option<(u32, Font<'a>)>,
    scripts: Vec<(u32, Script<'a>)>,
}

impl<'a> Typed<'a> {
    fn record(&mut self, slot: SlotIndex, asset: Asset<'a>) {
        self.calls += 1;
        let slot = slot.get();
        let replaced = match asset {
            Asset::Sprite(value) => self.sprite.replace((slot, value)).is_some(),
            Asset::Tileset(value) => self.tileset.replace((slot, value)).is_some(),
            Asset::Map(value) => self.map.replace((slot, value)).is_some(),
            Asset::Font(value) => self.font.replace((slot, value)).is_some(),
            Asset::Script(value) => {
                self.scripts.push((slot, value));
                false
            }
        };
        assert!(!replaced, "variante déposée deux fois (slot {slot})");
    }
}

fn le32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("4 octets"))
}

fn le64(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().expect("8 octets"))
}

#[test]
fn pf9_heterogeneous_batch_of_the_five_formats_round_trips_through_load_package() {
    let sources = hetero_sources();
    let capacity = HETERO_TABLE.len();

    // Production : le paquet et, séparément, le jeu historique du même lot.
    let compiled = compile_package(&sources, capacity, Q).expect("lot valide");
    let bytes = compiled.as_bytes();
    let set = compile_assets(&sources, capacity).expect("lot valide");

    // Les longueurs des artefacts sont distinctes et conformes aux valeurs littérales.
    let lens: Vec<usize> = set.artifacts().iter().map(Vec::len).collect();
    assert_eq!(lens, HETERO_LENGTHS);
    for (i, len) in lens.iter().enumerate() {
        assert!(!lens[..i].contains(len), "longueur {len} dupliquée");
    }

    // --- Offsets : analyse littérale du paquet, indépendante du chargement. -------------
    assert_eq!(&bytes[..4], envelope::SIGNATURE.as_slice());
    assert_eq!(&bytes[4..6], envelope::VERSION.to_le_bytes().as_slice());
    let count = usize::try_from(le32(bytes, 6)).expect("n'");
    assert_eq!(count, HETERO_LENGTHS.len());

    let table_end = envelope::HEADER_LEN + envelope::OFFSET_WIDTH * count;
    let manifest_len = manifest_v2::HEADER_LEN + manifest_v2::ENTRY_LEN * count;
    assert_eq!((table_end, manifest_len), (58, 114));

    let offsets: Vec<u64> = (0..count)
        .map(|i| le64(bytes, envelope::HEADER_LEN + envelope::OFFSET_WIDTH * i))
        .collect();
    // Valeurs écrites à la main, puis recalculées depuis les longueurs réelles.
    assert_eq!(offsets, HETERO_OFFSETS);
    let mut cursor = table_end + manifest_len;
    for (i, len) in lens.iter().enumerate() {
        assert_eq!(offsets[i], u64::try_from(cursor).expect("offset"), "s_{i}");
        cursor += len;
    }
    assert_eq!(cursor, HETERO_TOTAL);
    assert_eq!(bytes.len(), HETERO_TOTAL);
    // Strictement croissants, contigus, le dernier artefact finissant avec le paquet.
    for (i, artifact) in set.artifacts().iter().enumerate() {
        let start = usize::try_from(offsets[i]).expect("offset");
        let end = offsets
            .get(i + 1)
            .map_or(bytes.len(), |next| usize::try_from(*next).expect("offset"));
        assert!(start < end, "section {i} vide");
        assert_eq!(&bytes[start..end], artifact.as_slice(), "artefact {i}");
    }

    // --- Manifeste v2 : q, compteur, entrées ; ordre des artefacts ≠ ordre des slots. ---
    let manifest = &bytes[table_end..table_end + manifest_len];
    assert_eq!(&manifest[..4], manifest_v2::SIGNATURE.as_slice());
    assert_eq!(
        &manifest[4..6],
        manifest_v2::VERSION.to_le_bytes().as_slice()
    );
    assert_eq!(le64(manifest, 6), Q);
    assert_eq!(le32(manifest, 14), le32(bytes, 6), "n = n'");
    assert_eq!(&manifest[18..], &set.manifest()[10..], "entrées de la v1");
    for (k, &expected_slot) in HETERO_SLOTS.iter().enumerate().take(count) {
        let at = manifest_v2::HEADER_LEN + manifest_v2::ENTRY_LEN * k;
        let artifact_index = le32(manifest, at + 8);
        let slot_index = le32(manifest, at + 12);
        assert_eq!(artifact_index, u32::try_from(k).expect("indice"));
        assert_eq!(slot_index, expected_slot, "slot de l'entrée {k}");
        assert_ne!(artifact_index, slot_index, "entrée {k}");
    }
    // La carte (artefact 1) désigne le tileset par l'identifiant de l'entrée 0.
    assert_eq!(
        &set.artifacts()[1][10..18],
        &manifest[18..26],
        "identifiant du tileset dans la carte"
    );

    // --- Chargement du paquet. ----------------------------------------------------------
    let mut packaged = Typed::default();
    let result = load_package(bytes, &HETERO_TABLE, |slot, asset| {
        packaged.record(slot, asset)
    });
    assert_eq!(result.map(NonZeroU64::get), Ok(Q));
    assert_eq!(packaged.calls, 6);

    // --- Chemin historique sur le même ensemble de sources. -----------------------------
    let slices: Vec<&[u8]> = set.artifacts().iter().map(Vec::as_slice).collect();
    let mut legacy = Typed::default();
    load(set.manifest(), &slices, &HETERO_TABLE, |slot, asset| {
        legacy.record(slot, asset)
    })
    .expect("chargement v1");
    assert_eq!(legacy.calls, 6);

    // Mêmes ressources, mêmes slots, pour les cinq variantes.
    assert_eq!(packaged, legacy);

    // --- Contenu déposé, valeurs écrites à la main. -------------------------------------
    assert_eq!(
        packaged.sprite,
        Some((
            0,
            Sprite {
                width: 96,
                height: 48
            }
        ))
    );
    assert_eq!(
        packaged.tileset,
        Some((
            2,
            Tileset {
                tile_width: 16,
                tile_height: 8,
                columns: 4,
                count: 12
            }
        ))
    );

    let (map_slot, map) = packaged.map.expect("carte déposée");
    assert_eq!(map_slot, 5);
    assert_eq!((map.width(), map.height()), (3, 2));
    let rows = [[0u16, 1, 2], [3, 4, 11]];
    for (row, line) in rows.iter().enumerate() {
        for (column, value) in line.iter().enumerate() {
            assert_eq!(
                map.tile(
                    u16::try_from(column).expect("colonne"),
                    u16::try_from(row).expect("ligne")
                ),
                Some(*value),
                "case ({column}, {row})"
            );
        }
    }
    assert_eq!(map.tile(3, 0), None);

    // Référence Map → Tileset : résolue vers le slot du tileset (2), ni l'indice
    // d'artefact du tileset (0), ni le slot de la carte (5).
    let (tileset_slot, _) = packaged.tileset.expect("tileset déposé");
    assert_eq!(map.tileset(), SlotIndex::new(tileset_slot));
    assert_eq!(map.tileset(), SlotIndex::new(2));
    assert_ne!(map.tileset().get(), 0);
    assert_ne!(map.tileset().get(), map_slot);

    let (font_slot, font) = packaged.font.expect("police déposée");
    assert_eq!(font_slot, 3);
    assert_eq!((font.len(), font.line_height()), (3, 2571));
    for (index, glyph) in (0u16..).zip(FONT_GLYPHS) {
        assert_eq!(font.glyph(index), Some(glyph));
    }
    assert_eq!(font.glyph(3), None);

    // Scripts : reçus dans l'ordre du manifeste (« intro » slot 4, « outro » slot 1),
    // égaux à la relecture directe des artefacts correspondants.
    let intro = Script::read(&set.artifacts()[2]).expect("script conforme");
    let outro = Script::read(&set.artifacts()[4]).expect("script conforme");
    assert_eq!(packaged.scripts, vec![(4, intro), (1, outro)]);
    assert_eq!((intro.len(), outro.len()), (7, 4));
}
