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
    BindingError, LoadError, Manifest, ManifestEntry, ManifestError, ReadError, SlotIndex, Sprite,
    load,
};
use verbum_ecs::{ResourceSlot, World};
use verbum_forge::{AssetSource, CompiledSet, compile_assets, compile_sprite};

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
fn deposit(world: &mut PairWorld, slot: SlotIndex, sprite: Sprite) {
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
    load(&manifest, &slices, BINDING.len(), |slot, sprite| {
        deposit(&mut world, slot, sprite)
    })
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
    let result = load(manifest, artifacts, capacity, |slot, sprite| {
        calls += 1;
        deposit(&mut world, slot, sprite);
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
