// crates/forge/tests/pipeline.rs

//! Acceptation de la tranche A (jalon intermédiaire : ne valide pas le contrat complet
//! d'ADR-A01, manifeste et identité opaque exclus) : P3.1 et P3.2.
//!
//! Chaîne : `sprite.toml` → Forge → `sprite.bin` → lecteur binaire → `ResourceSlot<Sprite>`.
//! Invariants : I1, I6, I7, I8.
//!
//! Acceptation de la clôture (tranche B) : T6.1 à T6.6, plus bas. Chaîne : sources TOML →
//! compilation en lot → manifeste et artefacts → `load` → deux `ResourceSlot<Sprite>`.
//! Invariants : I12, I13, I6, I7, I8.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use verbum_assets::{
    Asset as RuntimeAsset, BindingError, Format, LoadError, MANIFEST_ENTRY_LEN,
    MANIFEST_HEADER_LEN, MANIFEST_SIGNATURE, MANIFEST_VERSION, Manifest, ManifestEntry,
    ManifestError, Map, ReadError, ReferenceError, SlotIndex, Sprite, Tileset, load,
};
use verbum_ecs::{ResourceSlot, World};
use verbum_forge::{
    AssetKind, AssetSource, CompiledSet, ForgeError, compile_assets, compile_sprite,
};

/// Répertoire temporaire unique, supprimé au `Drop` y compris en cas d'échec du test.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let path = std::env::temp_dir().join(format!(
            "verbum-forge-{label}-{}-{nanos}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("répertoire temporaire unique");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `R` de test : un seul slot de sprite, clé = position statique (I7).
#[derive(Default)]
struct Globals {
    sprite: ResourceSlot<Sprite>,
}

type TestWorld = World<1, (), Globals>;

const SOURCE: &str = "width = 258\nheight = 772\n";

#[test]
fn p3_1_full_chain_on_disk_with_the_source_deleted_before_reading() {
    let dir = TempDir::new("p3-1");
    let source_path = dir.path().join("sprite.toml");
    let artifact_path = dir.path().join("sprite.bin");

    // Forge : lit et parse la source humaine, écrit l'artefact (côté std).
    fs::write(&source_path, SOURCE).expect("écriture de la source");
    let source = fs::read_to_string(&source_path).expect("lecture de la source");
    let artifact = compile_sprite(&source).expect("source valide");
    fs::write(&artifact_path, &artifact).expect("écriture de l'artefact");

    // La source disparaît : la suite ne peut plus dépendre d'elle (I1).
    fs::remove_file(&source_path).expect("suppression de la source");
    assert!(!source_path.exists());

    // Infrastructure de chargement : seule à toucher le disque (I8).
    let bytes = fs::read(&artifact_path).expect("lecture de l'artefact");

    // Runtime : octets → Sprite → Ressource (I6), lue par le champ typé (I7).
    let mut world = TestWorld::new();
    assert_eq!(world.resources().sprite.get(), None);

    let sprite = Sprite::read(&bytes).expect("artefact conforme");
    assert_eq!(world.resources_mut().sprite.insert(sprite), None);

    assert_eq!(
        world.resources().sprite.get(),
        Some(&Sprite {
            width: 258,
            height: 772
        })
    );
}

#[test]
fn p3_2_the_same_source_file_is_accepted_by_the_forge_and_rejected_by_the_reader() {
    let dir = TempDir::new("p3-2");
    let source_path = dir.path().join("sprite.toml");
    fs::write(&source_path, SOURCE).expect("écriture de la source");

    let raw = fs::read(&source_path).expect("lecture des octets du fichier");

    let text = std::str::from_utf8(&raw).expect("source UTF-8");
    assert!(compile_sprite(text).is_ok(), "la Forge accepte la source");

    // Le lecteur runtime ne parse pas de TOML : ce fichier n'est pas un artefact (I1).
    assert!(raw.len() >= verbum_assets::ARTIFACT_LEN);
    assert_eq!(Sprite::read(&raw), Err(ReadError::FormatIdentity));
}

// ---------------------------------------------------------------------------------------
// Clôture de l'étape 2 : T6.1 à T6.6
// ---------------------------------------------------------------------------------------

/// `R` de l'application de test : deux champs distincts de type `ResourceSlot<Sprite>`.
#[derive(Default)]
struct Pair {
    first: ResourceSlot<Sprite>,
    second: ResourceSlot<Sprite>,
}

type PairWorld = World<1, (), Pair>;
type Accessor = fn(&mut Pair) -> &mut ResourceSlot<Sprite>;

fn first_slot(resources: &mut Pair) -> &mut ResourceSlot<Sprite> {
    &mut resources.first
}

fn second_slot(resources: &mut Pair) -> &mut ResourceSlot<Sprite> {
    &mut resources.second
}

/// Liaison statique `slot_index → champ` : slot 0 → `first`, slot 1 → `second`. Elle vit
/// dans l'application (ici le test), non dans `verbum-assets`. Sa longueur est la capacité.
static BINDING: [Accessor; 2] = [first_slot, second_slot];

/// Dépôt de l'application : applique la liaison statique. Aucun retour d'erreur.
fn deposit(world: &mut PairWorld, slot: SlotIndex, asset: RuntimeAsset) {
    let RuntimeAsset::Sprite(sprite) = asset else {
        panic!("variante inattendue : un sprite était attendu");
    };
    let index = usize::try_from(slot.get()).expect("indice de slot");
    let accessor = BINDING.get(index).expect("slot dans la configuration");
    let replaced = accessor(world.resources_mut()).insert(sprite);
    assert_eq!(replaced, None, "slot {index} déjà occupé");
}

/// Contenu des deux slots : `(first, second)`.
fn contents(world: &PairWorld) -> (Option<Sprite>, Option<Sprite>) {
    let resources = world.resources();
    (
        resources.first.get().copied(),
        resources.second.get().copied(),
    )
}

/// Un asset de test : clé logique, source TOML, slot cible.
#[derive(Clone, Copy)]
struct Asset {
    key: &'static str,
    toml: &'static str,
    slot: u32,
}

const A: Asset = Asset {
    key: "a",
    toml: "width = 32\nheight = 16\n",
    slot: 1,
};
const FOOBAR: Asset = Asset {
    key: "foobar",
    toml: "width = 96\nheight = 48\n",
    slot: 0,
};
const THIRD: Asset = Asset {
    key: "third",
    toml: "width = 8\nheight = 8\n",
    slot: 2,
};

const SPRITE_32X16: Sprite = Sprite {
    width: 32,
    height: 16,
};
const SPRITE_96X48: Sprite = Sprite {
    width: 96,
    height: 48,
};

/// Compile en mémoire, la capacité étant passée explicitement.
fn compile(assets: &[Asset], capacity: usize) -> CompiledSet {
    let sources: Vec<AssetSource<'_>> = assets
        .iter()
        .map(|asset| AssetSource {
            kind: AssetKind::Sprite,
            key: asset.key,
            toml: asset.toml,
            slot: SlotIndex::new(asset.slot),
        })
        .collect();
    compile_assets(&sources, capacity).expect("lot valide")
}

/// Sortie d'une chaîne complète sur disque.
struct Loaded {
    manifest: Vec<u8>,
    artifacts: Vec<Vec<u8>>,
    world: PairWorld,
}

/// Chaîne complète sur disque : sources écrites, lues par la Forge, compilation en lot,
/// manifeste et artefacts écrits, sources supprimées, relecture dans l'ordre conservé,
/// `load` avec la liaison statique. Les deux slots sont vides avant le chargement.
fn compile_and_load_on_disk(label: &str, assets: &[Asset]) -> Loaded {
    let dir = TempDir::new(label);

    let source_paths: Vec<PathBuf> = assets
        .iter()
        .enumerate()
        .map(|(index, asset)| {
            let path = dir.path().join(format!("source-{index}.toml"));
            fs::write(&path, asset.toml).expect("écriture de la source");
            path
        })
        .collect();

    let texts: Vec<String> = source_paths
        .iter()
        .map(|path| fs::read_to_string(path).expect("lecture de la source"))
        .collect();
    let sources: Vec<AssetSource<'_>> = assets
        .iter()
        .zip(&texts)
        .map(|(asset, text)| AssetSource {
            kind: AssetKind::Sprite,
            key: asset.key,
            toml: text,
            slot: SlotIndex::new(asset.slot),
        })
        .collect();
    let set = compile_assets(&sources, BINDING.len()).expect("lot valide");

    let manifest_path = dir.path().join("assets.vman");
    fs::write(&manifest_path, set.manifest()).expect("écriture du manifeste");
    // Liste ordonnée des chemins : l'ordre du système de fichiers n'est jamais consulté.
    let artifact_paths: Vec<PathBuf> = set
        .artifacts()
        .iter()
        .enumerate()
        .map(|(index, bytes)| {
            let path = dir.path().join(format!("artifact-{index}.bin"));
            fs::write(&path, bytes).expect("écriture de l'artefact");
            path
        })
        .collect();

    for path in &source_paths {
        fs::remove_file(path).expect("suppression de la source");
        assert!(!path.exists());
    }

    let manifest = fs::read(&manifest_path).expect("lecture du manifeste");
    let artifacts: Vec<Vec<u8>> = artifact_paths
        .iter()
        .map(|path| fs::read(path).expect("lecture de l'artefact"))
        .collect();

    let mut world = PairWorld::new();
    assert_eq!(
        contents(&world),
        (None, None),
        "slots vides avant chargement"
    );
    let slices: Vec<&[u8]> = artifacts.iter().map(Vec::as_slice).collect();
    load(
        &manifest,
        &slices,
        &[Format::Sprite; BINDING.len()],
        |slot, asset| deposit(&mut world, slot, asset),
    )
    .expect("chargement");

    Loaded {
        manifest,
        artifacts,
        world,
    }
}

/// Décode l'artefact désigné par une entrée du manifeste.
fn decode(artifacts: &[Vec<u8>], entry: ManifestEntry) -> Sprite {
    let index = usize::try_from(entry.artifact_index().get()).expect("indice d'artefact");
    Sprite::read(&artifacts[index]).expect("artefact conforme")
}

#[test]
fn t6_1_full_chain_on_disk_through_the_manifest_into_two_slots() {
    let loaded = compile_and_load_on_disk("t6-1", &[A, FOOBAR]);

    // Clé « a » → slot 1 (`second`), clé « foobar » → slot 0 (`first`).
    assert_eq!(
        contents(&loaded.world),
        (Some(SPRITE_96X48), Some(SPRITE_32X16))
    );
}

/// Manifeste littéral indépendant de la Forge, écrit à la main : identifiants du RFC 9923
/// §8.3 (`"a"` puis `"foobar"`), artefacts 0 et 1, slots 1 et 0.
const REFERENCE_MANIFEST: [u8; 42] = [
    0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // en-tête, n = 2
    0x8C, 0xEC, 0x01, 0x86, 0x4C, 0xDC, 0x63, 0xAF, // id de « a »
    0x00, 0x00, 0x00, 0x00, // artifact_index 0
    0x01, 0x00, 0x00, 0x00, // slot_index 1
    0xE8, 0x67, 0x39, 0xF7, 0x71, 0x41, 0x94, 0x85, // id de « foobar »
    0x01, 0x00, 0x00, 0x00, // artifact_index 1
    0x00, 0x00, 0x00, 0x00, // slot_index 0
];

#[test]
fn t6_2_expected_ids_resolve_to_the_right_artifact_and_slot_without_load() {
    let reference = Manifest::parse(&REFERENCE_MANIFEST).expect("manifeste littéral conforme");
    let mut expected = reference.entries();
    let id_of_a = expected.next().expect("entrée « a »").id();
    let id_of_foobar = expected.next().expect("entrée « foobar »").id();

    let set = compile(&[A, FOOBAR], BINDING.len());
    let produced = Manifest::parse(set.manifest()).expect("manifeste produit conforme");

    let entry = produced.find(id_of_a).expect("identifiant de « a »");
    assert_eq!(entry.artifact_index().get(), 0);
    assert_eq!(entry.slot_index().get(), 1);
    assert_eq!(decode(set.artifacts(), entry), SPRITE_32X16);

    let entry = produced
        .find(id_of_foobar)
        .expect("identifiant de « foobar »");
    assert_eq!(entry.artifact_index().get(), 1);
    assert_eq!(entry.slot_index().get(), 0);
    assert_eq!(decode(set.artifacts(), entry), SPRITE_96X48);
}

#[test]
fn t6_3_reversing_the_compilation_order_changes_artifact_indices_but_not_the_outcome() {
    let reference = compile_and_load_on_disk("t6-3-reference", &[A, FOOBAR]);
    let permuted = compile_and_load_on_disk("t6-3-permuted", &[FOOBAR, A]);

    let before = Manifest::parse(&reference.manifest).expect("manifeste conforme");
    let after = Manifest::parse(&permuted.manifest).expect("manifeste conforme");
    assert_eq!(before.len(), after.len());
    for entry in before.entries() {
        let moved = after.find(entry.id()).expect("même identifiant");
        assert_eq!(moved.slot_index(), entry.slot_index());
        assert_ne!(
            moved.artifact_index(),
            entry.artifact_index(),
            "l'indice d'artefact doit avoir changé"
        );
        assert_eq!(
            decode(&permuted.artifacts, moved),
            decode(&reference.artifacts, entry)
        );
    }

    // État final identique à T6.1.
    let expected = (Some(SPRITE_96X48), Some(SPRITE_32X16));
    assert_eq!(contents(&reference.world), expected);
    assert_eq!(contents(&permuted.world), expected);
}

/// Charge dans un monde neuf et exige un rejet : le dépôt n'est jamais appelé et les
/// deux slots restent vides.
fn load_rejected(manifest: &[u8], artifacts: &[&[u8]], capacity: usize) -> LoadError {
    let mut world = PairWorld::new();
    assert_eq!(
        contents(&world),
        (None, None),
        "slots vides avant chargement"
    );
    let mut calls = 0;
    let formats = vec![Format::Sprite; capacity];
    let result = load(manifest, artifacts, &formats, |slot, asset| {
        calls += 1;
        deposit(&mut world, slot, asset);
    });
    let error = result.expect_err("chargement rejeté");
    assert_eq!(calls, 0, "le dépôt ne doit jamais être appelé");
    assert_eq!(contents(&world), (None, None), "slots restés vides");
    error
}

/// Copie de `base` dont les octets à `offset` sont remplacés par `bytes`.
fn patched(base: &[u8], offset: usize, bytes: &[u8]) -> Vec<u8> {
    let mut altered = base.to_vec();
    altered[offset..offset + bytes.len()].copy_from_slice(bytes);
    altered
}

#[test]
fn t6_4_every_rejection_happens_before_the_first_deposit() {
    let set = compile(&[A, FOOBAR], BINDING.len());
    let artifacts: [&[u8]; 2] = [&set.artifacts()[0], &set.artifacts()[1]];
    let base = set.manifest();
    // Entrée 0 à l'offset 10, entrée 1 à l'offset 26 ; id +0, artefact +8, slot +12.

    // Identifiant dupliqué (niveau 1).
    let duplicated_id = patched(base, 26, &base[10..18]);
    assert_eq!(
        load_rejected(&duplicated_id, &artifacts, 2),
        LoadError::Manifest(ManifestError::DuplicateId)
    );

    // `slot_index` dupliqué (niveau 2).
    let duplicated_slot = patched(base, 38, &base[22..26]);
    assert_eq!(
        load_rejected(&duplicated_slot, &artifacts, 2),
        LoadError::Binding(BindingError::DuplicateSlotIndex)
    );

    // `slot_index` hors limites.
    let slot_out_of_range = patched(base, 38, &[0x02, 0x00, 0x00, 0x00]);
    assert_eq!(
        load_rejected(&slot_out_of_range, &artifacts, 2),
        LoadError::Binding(BindingError::SlotIndexOutOfRange)
    );

    // `artifact_index` hors limites.
    let artifact_out_of_range = patched(base, 34, &[0x02, 0x00, 0x00, 0x00]);
    assert_eq!(
        load_rejected(&artifact_out_of_range, &artifacts, 2),
        LoadError::Binding(BindingError::ArtifactIndexOutOfRange)
    );

    // Nombre d'artefacts différent de `n`, en moins puis en plus.
    assert_eq!(
        load_rejected(base, &artifacts[..1], 2),
        LoadError::Binding(BindingError::EntryCountMismatch)
    );
    assert_eq!(
        load_rejected(base, &[artifacts[0], artifacts[1], artifacts[0]], 2),
        LoadError::Binding(BindingError::EntryCountMismatch)
    );

    // Un artefact mal formé parmi des artefacts valides (niveau 3).
    let error = load_rejected(base, &[artifacts[0], &artifacts[1][..13]], 2);
    assert!(
        matches!(error, LoadError::Artifact { index, error: ReadError::Truncated } if index.get() == 1),
        "obtenu : {error:?}"
    );
    let error = load_rejected(base, &[&artifacts[0][..5], artifacts[1]], 2);
    assert!(
        matches!(error, LoadError::Artifact { index, error: ReadError::Truncated } if index.get() == 0),
        "obtenu : {error:?}"
    );

    // Trois assets compilés pour trois slots, chargés avec une capacité de 2.
    let three = compile(&[A, FOOBAR, THIRD], 3);
    let three_artifacts: [&[u8]; 3] = [
        &three.artifacts()[0],
        &three.artifacts()[1],
        &three.artifacts()[2],
    ];
    assert_eq!(
        load_rejected(three.manifest(), &three_artifacts, 2),
        LoadError::Binding(BindingError::CapacityExceeded)
    );
}

#[test]
fn t6_5_the_toml_source_given_to_the_manifest_reader_is_rejected() {
    let source = A.toml.as_bytes();
    assert!(source.len() >= 10);

    assert_eq!(
        Manifest::parse(source).err(),
        Some(ManifestError::Signature)
    );
    assert_eq!(
        load_rejected(source, &[], BINDING.len()),
        LoadError::Manifest(ManifestError::Signature)
    );
}

#[test]
fn t6_6_a_capacity_above_the_asset_count_leaves_the_other_slots_empty() {
    // Un seul asset, capacité 2 : le slot 0 n'est pas sollicité.
    let single = compile_and_load_on_disk("t6-6-single", &[A]);
    assert_eq!(contents(&single.world), (None, Some(SPRITE_32X16)));

    // Manifeste vide et collection vide : aucun dépôt, les deux slots vides.
    let empty = compile_and_load_on_disk("t6-6-empty", &[]);
    assert_eq!(empty.manifest.len(), 10);
    assert!(empty.artifacts.is_empty());
    assert_eq!(contents(&empty.world), (None, None));
}

// ---------------------------------------------------------------------------------------
// U2 : tileset, TS-1 à TS-6
// ---------------------------------------------------------------------------------------

/// `R` de l'application de test : un slot par genre, plus un slot de tileset inutilisé.
#[derive(Default)]
struct Mixed {
    sprite: ResourceSlot<Sprite>,
    tileset: ResourceSlot<Tileset>,
    spare: ResourceSlot<Tileset>,
}

type MixedWorld = World<1, (), Mixed>;

/// Table statique des formats attendus : slot 0 → sprite, slots 1 et 2 → tileset. Sa
/// longueur est la capacité.
static TABLE: [Format; 3] = [Format::Sprite, Format::Tileset, Format::Tileset];

/// Dépôt de l'application : liaison statique `slot → champ typé`. Aucun retour d'erreur ;
/// un couple (slot, variante) inattendu ou un slot occupé fait échouer le test.
fn deposit_mixed(world: &mut MixedWorld, slot: SlotIndex, asset: RuntimeAsset) {
    let resources = world.resources_mut();
    let replaced = match (slot.get(), asset) {
        (0, RuntimeAsset::Sprite(value)) => resources.sprite.insert(value).is_some(),
        (1, RuntimeAsset::Tileset(value)) => resources.tileset.insert(value).is_some(),
        (2, RuntimeAsset::Tileset(value)) => resources.spare.insert(value).is_some(),
        (slot, other) => panic!("dépôt inattendu : slot {slot}, {other:?}"),
    };
    assert!(!replaced, "slot déjà occupé");
}

type MixedContents = (Option<Sprite>, Option<Tileset>, Option<Tileset>);

fn mixed_contents(world: &MixedWorld) -> MixedContents {
    let resources = world.resources();
    (
        resources.sprite.get().copied(),
        resources.tileset.get().copied(),
        resources.spare.get().copied(),
    )
}

const TILES_TOML: &str = "tile_width = 16\ntile_height = 8\ncolumns = 4\ncount = 12\n";

/// Artefact attendu pour `TILES_TOML`, écrit à la main.
const TILESET_16X8: [u8; 18] = [
    0x56, 0x54, 0x4C, 0x53, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x08, 0x00, 0x04, 0x00,
    0x0C, 0x00,
];

const TILESET_VALUE: Tileset = Tileset {
    tile_width: 16,
    tile_height: 8,
    columns: 4,
    count: 12,
};

fn tiles() -> AssetSource<'static> {
    AssetSource {
        kind: AssetKind::Tileset,
        key: "a",
        toml: TILES_TOML,
        slot: SlotIndex::new(1),
    }
}

fn foobar_sprite() -> AssetSource<'static> {
    AssetSource {
        kind: AssetKind::Sprite,
        key: "foobar",
        toml: FOOBAR.toml,
        slot: SlotIndex::new(0),
    }
}

/// Ordre de référence : tileset (« a », slot 1) puis sprite (« foobar », slot 0), de sorte
/// que `artifact_index ≠ slot_index` pour chaque entrée.
fn reference_set(capacity: usize) -> CompiledSet {
    compile_assets(&[tiles(), foobar_sprite()], capacity).expect("lot valide")
}

fn slices(set: &CompiledSet) -> Vec<&[u8]> {
    set.artifacts().iter().map(Vec::as_slice).collect()
}

#[test]
fn ts_1_a_sprite_and_a_tileset_flow_from_sources_into_two_typed_slots() {
    let set = reference_set(2);
    assert_eq!(set.artifacts()[0], TILESET_16X8);
    assert_eq!(set.artifacts()[1].len(), 14);

    let mut world = MixedWorld::new();
    assert_eq!(mixed_contents(&world), (None, None, None));
    let mut calls = 0;
    load(set.manifest(), &slices(&set), &TABLE[..2], |slot, asset| {
        calls += 1;
        deposit_mixed(&mut world, slot, asset);
    })
    .expect("chargement");
    assert_eq!(calls, 2);
    assert_eq!(
        mixed_contents(&world),
        (Some(SPRITE_96X48), Some(TILESET_VALUE), None)
    );
}

#[test]
fn ts_2_expected_ids_resolve_to_the_right_artifact_slot_and_format() {
    let reference = Manifest::parse(&REFERENCE_MANIFEST).expect("manifeste littéral conforme");
    let mut expected = reference.entries();
    let id_of_a = expected.next().expect("entrée « a »").id();
    let id_of_foobar = expected.next().expect("entrée « foobar »").id();

    let set = reference_set(2);
    let produced = Manifest::parse(set.manifest()).expect("manifeste produit conforme");

    let entry = produced.find(id_of_a).expect("identifiant de « a »");
    assert_eq!(entry.artifact_index().get(), 0);
    assert_eq!(entry.slot_index().get(), 1);
    let index = usize::try_from(entry.artifact_index().get()).expect("indice");
    assert_eq!(Tileset::read(&set.artifacts()[index]), Ok(TILESET_VALUE));

    let entry = produced
        .find(id_of_foobar)
        .expect("identifiant de « foobar »");
    assert_eq!(entry.artifact_index().get(), 1);
    assert_eq!(entry.slot_index().get(), 0);
    assert_eq!(decode(set.artifacts(), entry), SPRITE_96X48);
}

#[test]
fn ts_3_reversing_the_compilation_order_changes_artifact_indices_but_not_the_outcome() {
    let reference = reference_set(2);
    let reversed = compile_assets(&[foobar_sprite(), tiles()], 2).expect("lot valide");

    let before = Manifest::parse(reference.manifest()).expect("manifeste conforme");
    let after = Manifest::parse(reversed.manifest()).expect("manifeste conforme");
    for entry in before.entries() {
        let moved = after.find(entry.id()).expect("même identifiant");
        assert_eq!(moved.slot_index(), entry.slot_index());
        assert_ne!(moved.artifact_index(), entry.artifact_index());
    }

    let mut world = MixedWorld::new();
    load(
        reversed.manifest(),
        &slices(&reversed),
        &TABLE[..2],
        |slot, asset| deposit_mixed(&mut world, slot, asset),
    )
    .expect("chargement");
    assert_eq!(
        mixed_contents(&world),
        (Some(SPRITE_96X48), Some(TILESET_VALUE), None)
    );
}

/// Charge et exige un rejet : aucun dépôt, aucun slot occupé.
fn mixed_rejected(manifest: &[u8], artifacts: &[&[u8]], table: &[Format]) -> LoadError {
    let mut world = MixedWorld::new();
    let mut calls = 0;
    let result = load(manifest, artifacts, table, |slot, asset| {
        calls += 1;
        deposit_mixed(&mut world, slot, asset);
    });
    let error = result.expect_err("chargement rejeté");
    assert_eq!(calls, 0, "le dépôt ne doit jamais être appelé");
    assert_eq!(mixed_contents(&world), (None, None, None));
    error
}

#[test]
fn ts_4_every_rejection_happens_before_the_first_deposit() {
    let set = reference_set(2);
    let artifacts = slices(&set);

    // Niveau 1 : manifeste altéré.
    let mut altered = set.manifest().to_vec();
    altered[0] ^= 0xFF;
    assert_eq!(
        mixed_rejected(&altered, &artifacts, &TABLE[..2]),
        LoadError::Manifest(ManifestError::Signature)
    );

    // Niveau 2 : liaisons (table trop courte).
    assert_eq!(
        mixed_rejected(set.manifest(), &artifacts, &TABLE[..1]),
        LoadError::Binding(BindingError::CapacityExceeded)
    );

    // Niveau 2 : format du slot (le tileset de l'artefact 0 est lié au slot 1, qui attend
    // un sprite dans cette table).
    assert_eq!(
        mixed_rejected(
            set.manifest(),
            &artifacts,
            &[Format::Tileset, Format::Sprite]
        ),
        LoadError::SlotFormat {
            slot: SlotIndex::new(1)
        }
    );

    // Niveau 3 : artefact tronqué (le préfixe est complet, la charge utile non).
    let truncated: [&[u8]; 2] = [&TILESET_16X8[..17], artifacts[1]];
    let error = mixed_rejected(set.manifest(), &truncated, &TABLE[..2]);
    let LoadError::Artifact { index, error } = error else {
        panic!("attendu : erreur d'artefact, obtenu {error:?}");
    };
    assert_eq!(index.get(), 0);
    assert_eq!(error, ReadError::Truncated);
}

#[test]
fn ts_5_the_toml_source_given_to_the_manifest_reader_is_rejected() {
    let source = TILES_TOML.as_bytes();
    assert!(source.len() >= 10);

    assert_eq!(
        Manifest::parse(source).err(),
        Some(ManifestError::Signature)
    );
    assert_eq!(
        mixed_rejected(source, &[], &TABLE),
        LoadError::Manifest(ManifestError::Signature)
    );
}

#[test]
fn ts_6_a_capacity_above_the_asset_count_leaves_the_other_slots_empty() {
    let set = reference_set(TABLE.len());
    let mut world = MixedWorld::new();
    let mut calls = 0;
    load(set.manifest(), &slices(&set), &TABLE, |slot, asset| {
        calls += 1;
        deposit_mixed(&mut world, slot, asset);
    })
    .expect("chargement");
    assert_eq!(calls, 2);
    assert_eq!(
        mixed_contents(&world),
        (Some(SPRITE_96X48), Some(TILESET_VALUE), None)
    );
}

// ---------------------------------------------------------------------------------------
// U3 : map, MAP-F1 à MAP-F6, VM-1, MS-1 à MS-7
// ---------------------------------------------------------------------------------------

/// Identifiant FNV-1a 64 de la clé « a » (RFC 9923 §8.3), écrit à la main.
const ID_OF_A: [u8; 8] = [0x8C, 0xEC, 0x01, 0x86, 0x4C, 0xDC, 0x63, 0xAF];

/// Tileset de 3000 tuiles : assez grand pour les indices de la carte de référence.
const BIG_TILES_TOML: &str = "tile_width = 16\ntile_height = 8\ncolumns = 4\ncount = 3000\n";

/// Carte 3 × 2 de référence, référençant la clé « a ».
const MAP_TOML: &str =
    "tileset = \"a\"\nwidth = 3\nheight = 2\ncells = [1, 515, 1029, 1543, 2057, 2571]\n";

/// Artefact attendu pour `MAP_TOML` avec le tileset « a », écrit à la main.
const MAP_BYTES: [u8; 34] = [
    0x56, 0x4D, 0x41, 0x50, // identité
    0x01, 0x00, // version
    0x18, 0x00, 0x00, 0x00, // longueur de charge utile 24
    0x8C, 0xEC, 0x01, 0x86, 0x4C, 0xDC, 0x63, 0xAF, // identifiant du tileset « a »
    0x03, 0x00, 0x02, 0x00, // 3 × 2
    0x01, 0x00, 0x03, 0x02, 0x05, 0x04, 0x07, 0x06, 0x09, 0x08, 0x0B, 0x0A,
];

fn tileset_source(key: &'static str, toml: &'static str, slot: u32) -> AssetSource<'static> {
    AssetSource {
        kind: AssetKind::Tileset,
        key,
        toml,
        slot: SlotIndex::new(slot),
    }
}

fn map_source(key: &'static str, toml: &'static str, slot: u32) -> AssetSource<'static> {
    AssetSource {
        kind: AssetKind::Map,
        key,
        toml,
        slot: SlotIndex::new(slot),
    }
}

fn sprite_source(key: &'static str, slot: u32) -> AssetSource<'static> {
    AssetSource {
        kind: AssetKind::Sprite,
        key,
        toml: FOOBAR.toml,
        slot: SlotIndex::new(slot),
    }
}

fn invalid_entry(error: ForgeError, entry: usize) -> ForgeError {
    ForgeError::InvalidSource {
        entry,
        cause: Box::new(error),
    }
}

#[test]
fn map_f1_a_valid_source_produces_exactly_the_literal_bytes() {
    let set = compile_assets(
        &[
            tileset_source("a", BIG_TILES_TOML, 0),
            map_source("foobar", MAP_TOML, 1),
        ],
        2,
    )
    .expect("lot valide");
    assert_eq!(set.artifacts()[1], MAP_BYTES);
}

#[test]
fn map_f2_the_output_is_read_back_by_the_runtime_with_the_key_identifier() {
    let set = compile_assets(
        &[
            tileset_source("a", BIG_TILES_TOML, 0),
            map_source("foobar", MAP_TOML, 1),
        ],
        2,
    )
    .expect("lot valide");
    // L'identifiant écrit dans la carte est celui de la clé « a », lu dans le manifeste.
    assert_eq!(set.artifacts()[1][10..18], ID_OF_A);
    let manifest = Manifest::parse(set.manifest()).expect("manifeste conforme");
    assert_eq!(set.manifest()[10..18], ID_OF_A);
    assert_eq!(manifest.len(), 2);

    let slices = slices(&set);
    let mut received = None;
    load(
        set.manifest(),
        &slices,
        &[Format::Tileset, Format::Map],
        |slot, asset| {
            if let RuntimeAsset::Map(map) = asset {
                received = Some((slot.get(), map));
            }
        },
    )
    .expect("chargement");
    let (slot, map) = received.expect("carte déposée");
    assert_eq!(slot, 1);
    assert_eq!(map.tileset(), SlotIndex::new(0));
    assert_eq!((map.width(), map.height()), (3, 2));
    assert_eq!(map.tile(0, 0), Some(1));
    assert_eq!(map.tile(2, 1), Some(2571));
}

#[test]
fn map_f4_reference_errors_are_refused_by_the_batch() {
    let tiles = tileset_source("a", TILES_TOML, 0);
    let sprite = sprite_source("hero", 1);
    let tiles_map = |toml| map_source("m", toml, 2);

    // Clé absente du lot.
    let dangling = "tileset = \"absent\"\nwidth = 1\nheight = 1\ncells = [0]\n";
    assert_eq!(
        compile_assets(&[tiles, tiles_map(dangling)], 3).err(),
        Some(invalid_entry(ForgeError::ReferenceNotFound, 1))
    );

    // Référent d'un autre genre : un sprite, une autre carte, la carte elle-même.
    let to_sprite = "tileset = \"hero\"\nwidth = 1\nheight = 1\ncells = [0]\n";
    assert_eq!(
        compile_assets(&[tiles, sprite, tiles_map(to_sprite)], 3).err(),
        Some(invalid_entry(ForgeError::ReferenceKind, 2))
    );
    let to_map = "tileset = \"other\"\nwidth = 1\nheight = 1\ncells = [0]\n";
    let other = map_source(
        "other",
        "tileset = \"a\"\nwidth = 1\nheight = 1\ncells = [0]\n",
        3,
    );
    assert_eq!(
        compile_assets(&[tiles, tiles_map(to_map), other], 4).err(),
        Some(invalid_entry(ForgeError::ReferenceKind, 1))
    );
    let to_self = "tileset = \"m\"\nwidth = 1\nheight = 1\ncells = [0]\n";
    assert_eq!(
        compile_assets(&[tiles, tiles_map(to_self)], 3).err(),
        Some(invalid_entry(ForgeError::ReferenceKind, 1))
    );

    // Indice de case : `count - 1` accepté, `count` refusé (le tileset compte 12 tuiles).
    let last = "tileset = \"a\"\nwidth = 1\nheight = 1\ncells = [11]\n";
    assert!(compile_assets(&[tiles, tiles_map(last)], 3).is_ok());
    let beyond = "tileset = \"a\"\nwidth = 1\nheight = 1\ncells = [12]\n";
    assert_eq!(
        compile_assets(&[tiles, tiles_map(beyond)], 3).err(),
        Some(invalid_entry(ForgeError::CellOutOfRange, 1))
    );
}

#[test]
fn map_f5_the_compilation_is_deterministic_and_independent_of_the_input_order() {
    let tiles = tileset_source("a", BIG_TILES_TOML, 0);
    let map = map_source("foobar", MAP_TOML, 1);
    let hero = sprite_source("hero", 2);

    let first = compile_assets(&[tiles, map, hero], 3).expect("lot valide");
    let again = compile_assets(&[tiles, map, hero], 3).expect("lot valide");
    assert_eq!(first, again);

    let permuted = compile_assets(&[hero, map, tiles], 3).expect("lot valide");
    // Mêmes octets d'artefact, retrouvés par identifiant : seuls les `artifact_index`
    // changent, jamais l'identifiant de référence écrit dans la carte.
    let before = Manifest::parse(first.manifest()).expect("manifeste conforme");
    let after = Manifest::parse(permuted.manifest()).expect("manifeste conforme");
    for entry in before.entries() {
        let moved = after.find(entry.id()).expect("même identifiant");
        assert_eq!(moved.slot_index(), entry.slot_index());
        let old = usize::try_from(entry.artifact_index().get()).expect("indice");
        let new = usize::try_from(moved.artifact_index().get()).expect("indice");
        assert_eq!(first.artifacts()[old], permuted.artifacts()[new]);
    }
    assert_ne!(first.manifest(), permuted.manifest());
    assert_eq!(permuted.artifacts()[1], MAP_BYTES);
}

#[test]
fn map_f6_the_output_carries_no_string() {
    let key = "ground-tiles";
    let set = compile_assets(
        &[
            tileset_source(key, BIG_TILES_TOML, 0),
            map_source(
                "level-one",
                "tileset = \"ground-tiles\"\nwidth = 1\nheight = 1\ncells = [7]\n",
                1,
            ),
        ],
        2,
    )
    .expect("lot valide");
    let map = &set.artifacts()[1];
    assert_eq!(map.len(), 10 + 12 + 2);
    for name in [key, "level-one", "tileset", "cells", "width", "height"] {
        assert!(
            !map.windows(name.len())
                .any(|window| window == name.as_bytes()),
            "« {name} » figure dans l'artefact de la carte"
        );
        assert!(
            !set.manifest()
                .windows(name.len())
                .any(|window| window == name.as_bytes()),
            "« {name} » figure dans le manifeste"
        );
    }
}

#[test]
fn vm_1_the_manifest_layout_is_unchanged_by_the_new_asset_kinds() {
    assert_eq!(MANIFEST_SIGNATURE, *b"VMAN");
    assert_eq!(MANIFEST_VERSION, 1);
    assert_eq!(MANIFEST_HEADER_LEN, 10);
    assert_eq!(MANIFEST_ENTRY_LEN, 16);

    // Lot mixte : longueur `10 + 16·n` et en-tête littéral. Le lot de référence de
    // l'étape 2 (`f2`) reste vérifié octet par octet par son propre test.
    let set = compile_assets(
        &[
            tileset_source("a", BIG_TILES_TOML, 0),
            map_source("foobar", MAP_TOML, 1),
            sprite_source("hero", 2),
        ],
        3,
    )
    .expect("lot valide");
    assert_eq!(set.manifest().len(), 10 + 16 * 3);
    assert_eq!(
        set.manifest()[..10],
        [0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00]
    );
}

/// `R` de l'application de test : un slot par genre (sprite, tileset, carte).
#[derive(Default)]
struct Triple<'a> {
    sprite: ResourceSlot<Sprite>,
    tileset: ResourceSlot<Tileset>,
    map: ResourceSlot<Map<'a>>,
}

type TripleWorld<'a> = World<1, (), Triple<'a>>;

/// Table statique : slot 0 → sprite, slot 1 → tileset, slot 2 → carte.
static TRIPLE_TABLE: [Format; 3] = [Format::Sprite, Format::Tileset, Format::Map];

fn deposit_triple<'a>(world: &mut TripleWorld<'a>, slot: SlotIndex, asset: RuntimeAsset<'a>) {
    let resources = world.resources_mut();
    let replaced = match (slot.get(), asset) {
        (0, RuntimeAsset::Sprite(value)) => resources.sprite.insert(value).is_some(),
        (1, RuntimeAsset::Tileset(value)) => resources.tileset.insert(value).is_some(),
        (2, RuntimeAsset::Map(value)) => resources.map.insert(value).is_some(),
        (slot, other) => panic!("dépôt inattendu : slot {slot}, {other:?}"),
    };
    assert!(!replaced, "slot déjà occupé");
}

type TripleContents<'a> = (Option<Sprite>, Option<Tileset>, Option<Map<'a>>);

fn triple_contents<'a>(world: &TripleWorld<'a>) -> TripleContents<'a> {
    let resources = world.resources();
    (
        resources.sprite.get().copied(),
        resources.tileset.get().copied(),
        resources.map.get().copied(),
    )
}

const CHAIN_MAP_TOML: &str =
    "tileset = \"a\"\nwidth = 3\nheight = 2\ncells = [0, 1, 2, 3, 4, 11]\n";

/// Ordre de référence du lot de chaîne : carte, tileset, sprite (le tileset référencé a
/// `artifact_index` 1 et `slot_index` 1 ; la carte a `artifact_index` 0 et `slot_index` 2).
fn chain_sources() -> [AssetSource<'static>; 3] {
    [
        map_source("foobar", CHAIN_MAP_TOML, 2),
        tileset_source("a", TILES_TOML, 1),
        sprite_source("hero", 0),
    ]
}

fn expected_triple_map(map: Map<'_>) {
    assert_eq!(map.tileset(), SlotIndex::new(1));
    assert_eq!((map.width(), map.height()), (3, 2));
    let rows = [[0, 1, 2], [3, 4, 11]];
    for (row, line) in rows.iter().enumerate() {
        for (column, value) in line.iter().enumerate() {
            assert_eq!(
                map.tile(
                    u16::try_from(column).expect("colonne"),
                    u16::try_from(row).expect("ligne")
                ),
                Some(*value)
            );
        }
    }
}

#[test]
fn ms_1_a_sprite_a_tileset_and_a_map_flow_into_three_typed_slots() {
    let set = compile_assets(&chain_sources(), 3).expect("lot valide");
    let mut world = TripleWorld::new();
    assert_eq!(triple_contents(&world), (None, None, None));
    let mut calls = 0;
    load(
        set.manifest(),
        &slices(&set),
        &TRIPLE_TABLE,
        |slot, asset| {
            calls += 1;
            deposit_triple(&mut world, slot, asset);
        },
    )
    .expect("chargement");
    assert_eq!(calls, 3);
    let (sprite, tileset, map) = triple_contents(&world);
    assert_eq!(sprite, Some(SPRITE_96X48));
    assert_eq!(tileset, Some(TILESET_VALUE));
    expected_triple_map(map.expect("carte déposée"));
}

#[test]
fn ms_2_identifiers_resolve_to_the_right_artifact_slot_and_format() {
    // Identifiants littéraux (RFC 9923 §8.3) de « a » (tileset) et « foobar » (carte).
    let id_of_a = Manifest::parse(&REFERENCE_MANIFEST)
        .expect("manifeste littéral conforme")
        .entries()
        .next()
        .expect("entrée « a »")
        .id();
    let id_of_foobar = Manifest::parse(&REFERENCE_MANIFEST)
        .expect("manifeste littéral conforme")
        .entries()
        .nth(1)
        .expect("entrée « foobar »")
        .id();

    let set = compile_assets(&chain_sources(), 3).expect("lot valide");
    let manifest = Manifest::parse(set.manifest()).expect("manifeste produit conforme");

    // Carte : artefact 0, slot 2 ; tileset : artefact 1, slot 1 ; sprite : artefact 2, slot 0.
    let map = manifest.find(id_of_foobar).expect("carte");
    assert_eq!((map.artifact_index().get(), map.slot_index().get()), (0, 2));
    let tiles = manifest.find(id_of_a).expect("tileset");
    assert_eq!(
        (tiles.artifact_index().get(), tiles.slot_index().get()),
        (1, 1)
    );
    assert_eq!(Tileset::read(&set.artifacts()[1]), Ok(TILESET_VALUE));
    let hero = manifest.entries().nth(2).expect("sprite");
    assert_eq!(
        (hero.artifact_index().get(), hero.slot_index().get()),
        (2, 0)
    );
    assert_eq!(decode(set.artifacts(), hero), SPRITE_96X48);
    // La carte désigne le tileset par l'identifiant de « a ».
    assert_eq!(set.artifacts()[0][10..18], ID_OF_A);
}

#[test]
fn ms_3_reversing_the_compilation_order_changes_artifact_indices_but_not_the_outcome() {
    let reference = compile_assets(&chain_sources(), 3).expect("lot valide");
    let mut reversed_sources = chain_sources();
    reversed_sources.reverse();
    let reversed = compile_assets(&reversed_sources, 3).expect("lot valide");

    let before = Manifest::parse(reference.manifest()).expect("manifeste conforme");
    let after = Manifest::parse(reversed.manifest()).expect("manifeste conforme");
    for entry in before.entries() {
        let moved = after.find(entry.id()).expect("même identifiant");
        assert_eq!(moved.slot_index(), entry.slot_index());
    }
    assert_ne!(reference.manifest(), reversed.manifest());

    let mut world = TripleWorld::new();
    load(
        reversed.manifest(),
        &slices(&reversed),
        &TRIPLE_TABLE,
        |slot, asset| deposit_triple(&mut world, slot, asset),
    )
    .expect("chargement");
    let (sprite, tileset, map) = triple_contents(&world);
    assert_eq!(sprite, Some(SPRITE_96X48));
    assert_eq!(tileset, Some(TILESET_VALUE));
    expected_triple_map(map.expect("carte déposée"));
}

/// Charge un lot et exige un rejet : aucun dépôt, aucun slot occupé.
fn triple_rejected(manifest: &[u8], artifacts: &[&[u8]], table: &[Format]) -> LoadError {
    let mut world = TripleWorld::new();
    let mut calls = 0;
    let result = load(manifest, artifacts, table, |slot, asset| {
        calls += 1;
        deposit_triple(&mut world, slot, asset);
    });
    let error = result.expect_err("chargement rejeté");
    assert_eq!(calls, 0, "le dépôt ne doit jamais être appelé");
    assert_eq!(triple_contents(&world), (None, None, None));
    error
}

#[test]
fn ms_4_and_ms_7_every_rejection_happens_before_the_first_deposit_references_included() {
    let set = compile_assets(&chain_sources(), 3).expect("lot valide");
    let artifacts = slices(&set);

    // Niveau 1 : manifeste altéré.
    let mut altered = set.manifest().to_vec();
    altered[0] ^= 0xFF;
    assert_eq!(
        triple_rejected(&altered, &artifacts, &TRIPLE_TABLE),
        LoadError::Manifest(ManifestError::Signature)
    );

    // Niveau 2 : liaisons (table trop courte), puis format du slot (table permutée).
    assert_eq!(
        triple_rejected(set.manifest(), &artifacts, &TRIPLE_TABLE[..2]),
        LoadError::Binding(BindingError::CapacityExceeded)
    );
    assert_eq!(
        triple_rejected(
            set.manifest(),
            &artifacts,
            &[Format::Sprite, Format::Map, Format::Tileset]
        ),
        LoadError::SlotFormat {
            slot: SlotIndex::new(2)
        }
    );

    // Niveau 3 : artefact tronqué (la carte a 34 octets, ici 33).
    let truncated_map = &set.artifacts()[0][..33];
    let truncated: [&[u8]; 3] = [truncated_map, artifacts[1], artifacts[2]];
    let LoadError::Artifact { index, error } =
        triple_rejected(set.manifest(), &truncated, &TRIPLE_TABLE)
    else {
        panic!("attendu : erreur d'artefact");
    };
    assert_eq!(index.get(), 0);
    assert_eq!(error, ReadError::Truncated);

    // Niveau 3 : référence non fermée (identifiant du tileset altéré dans la carte).
    let mut dangling = set.artifacts()[0].clone();
    dangling[10] ^= 0xFF;
    let dangling_set: [&[u8]; 3] = [&dangling, artifacts[1], artifacts[2]];
    let LoadError::Reference { index, error } =
        triple_rejected(set.manifest(), &dangling_set, &TRIPLE_TABLE)
    else {
        panic!("attendu : erreur de référence");
    };
    assert_eq!(index.get(), 0);
    assert_eq!(error, ReferenceError::Unresolved);
}

#[test]
fn ms_5_the_toml_source_given_to_the_manifest_reader_is_rejected() {
    let source = CHAIN_MAP_TOML.as_bytes();
    assert!(source.len() >= 10);
    assert_eq!(
        Manifest::parse(source).err(),
        Some(ManifestError::Signature)
    );
    assert_eq!(
        triple_rejected(source, &[], &TRIPLE_TABLE),
        LoadError::Manifest(ManifestError::Signature)
    );
}

#[test]
fn ms_6_a_capacity_above_the_asset_count_leaves_the_other_slots_empty() {
    // Quatre slots, trois assets : le slot 3 reste vide.
    let set = compile_assets(&chain_sources(), 4).expect("lot valide");
    let table = [Format::Sprite, Format::Tileset, Format::Map, Format::Sprite];
    let mut world = TripleWorld::new();
    let mut calls = 0;
    load(set.manifest(), &slices(&set), &table, |slot, asset| {
        calls += 1;
        deposit_triple(&mut world, slot, asset);
    })
    .expect("chargement");
    assert_eq!(calls, 3);
    let (sprite, tileset, map) = triple_contents(&world);
    assert_eq!(sprite, Some(SPRITE_96X48));
    assert_eq!(tileset, Some(TILESET_VALUE));
    expected_triple_map(map.expect("carte déposée"));
}
