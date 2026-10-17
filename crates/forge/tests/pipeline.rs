// crates/forge/tests/pipeline.rs

//! Acceptation de la tranche A (jalon intermédiaire : ne valide pas le contrat complet
//! d'ADR-A01, manifeste et identité opaque exclus).
//!
//! Chaîne : `sprite.toml` → Forge → `sprite.bin` → lecteur binaire → `ResourceSlot<Sprite>`.
//! Invariants : I1, I6, I7, I8.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use verbum_assets::{ReadError, Sprite};
use verbum_ecs::{ResourceSlot, World};
use verbum_forge::compile_sprite;

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
