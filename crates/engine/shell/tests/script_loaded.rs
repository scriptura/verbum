// crates/engine/shell/tests/script_loaded.rs

//! Étape 6, unité U5 : un comportement scripté de bout en bout, headless, sous `run_loop`.
//!
//! Chaîne : source TOML (scénario de référence) → `compile_assets` (un script, slot 4,
//! capacité 6) → `CompiledSet` → `load` (table de formats de 6 slots) → dépôt dans
//! `ScriptTable` → lancement par `CommandBuffer::add_component` + `World::apply` →
//! `run_loop` (cadence vide, arrêt après N itérations, `Inbox::<u8>` vide) → fermeture
//! d'infrastructure : journal `TickStart(k)`, `run_tick`, `TickEnd(k)`, lecture en lecture
//! seule de `TRANSCRIPT`, copie dans l'observateur de la Plateforme (un `Vec` de l'appelant),
//! `Present(k)`.
//!
//! - DM-1 : 12 transcripts égaux au littéral, état final et `Beat` ;
//! - DM-2 : quatre lots corrompus, `load` échoue, aucun tick, aucune présentation ;
//! - DM-3 : journal `[TickStart(k), TickEnd(k), Present(k)]`, présentation en lecture seule,
//!   historique tenu par l'observateur et non par le `World`.
//!
//! Le lot compilé est déclaré avant le `World` (les vues empruntent ses octets). Un seul
//! script, une seule instance : aucun ordre d'émission entre instances n'est affirmé.
//!
//! **Non démontré** : voir le § 10 du plan r1.2 (reproduit au rapport de clôture).

use core::cell::Cell;

use verbum_assets::{Asset, Format, LoadError, ReadError, SlotIndex, load};
use verbum_ecs::EntityId;
use verbum_schedule::scheduler::TickError;
use verbum_script::{ENCODED_LEN, SCRIPT_REGISTERS};
use verbum_shell::{Inbox, run_loop};

#[path = "script_support/mod.rs"]
#[allow(
    dead_code,
    reason = "chaque fichier de test n'emploie qu'une partie de l'assemblage partagé"
)]
mod support;

use support::{
    Acc, Buf, SCRIPT_FORMATS, SLOT_A, Sched, W, beat_of, build_world, capture, launch,
    reference_set, slices, transcript,
};

/// Nombre d'itérations du scénario.
const N: usize = 12;

type Pairs = Vec<(u16, i32)>;

/// Entrée du journal de la fermeture d'infrastructure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    TickStart(usize),
    TickEnd(usize),
    Present(usize),
}

/// État final attendu : ligne « 10 » (26 octets : `04 00 06 00 02 00`, puis 20 octets nuls).
fn final_state() -> [u8; ENCODED_LEN] {
    let mut bytes = [0u8; ENCODED_LEN];
    bytes[..6].copy_from_slice(&[0x04, 0x00, 0x06, 0x00, 0x02, 0x00]);
    bytes
}

/// Transcript attendu au tick `k` (1 à 12), littéral du scénario de référence.
fn expected_transcript(k: usize) -> Pairs {
    match k {
        1 | 4 | 7 => vec![(1, 10)],
        10 => vec![(2, 99)],
        _ => Vec::new(),
    }
}

/// Copie observable d'un accumulateur (lecture seule).
fn pairs(acc: &Acc) -> Pairs {
    acc.as_slice()
        .iter()
        .map(|emission| (emission.channel, emission.value))
        .collect()
}

/// `World` et instance après l'exécution.
struct Execution<'a> {
    world: W<'a>,
    id: EntityId,
}

/// Dépose dans la table, par le vrai `load`, les seuls artefacts de format script, avec la
/// table de formats `table`.
fn load_into<'a>(
    world: &mut W<'a>,
    manifest: &[u8],
    artifacts: &[&'a [u8]],
    table: &[Format],
) -> Result<(), LoadError> {
    load(manifest, artifacts, table, |slot, asset| {
        if let Asset::Script(script) = asset {
            let Some(scripts) = world.resources_mut().scripts.get_mut() else {
                panic!("violation d'assemblage : table absente");
            };
            if scripts.deposit(slot, script).is_err() {
                panic!("dépôt refusé au slot {}", slot.get());
            }
        }
    })
}

/// Charge le lot, lance l'instance, puis exécute `iterations` itérations sous `run_loop`.
/// Le journal et la présentation appartiennent à l'appelant : sur une erreur de chargement
/// ils restent intacts (aucun tick, aucune présentation).
fn execute<'a, P>(
    manifest: &[u8],
    artifacts: &[&'a [u8]],
    table: &[Format],
    iterations: usize,
    journal: &mut Vec<Entry>,
    mut present: P,
) -> Result<Execution<'a>, LoadError>
where
    P: FnMut(&Acc),
{
    let scheduler = Sched::new().expect("modèle valide");
    let mut world: W<'a> = build_world();
    let mut buffer = Buf::new();
    load_into(&mut world, manifest, artifacts, table)?;
    let id = launch(&mut world, &mut buffer, SLOT_A, 0);

    let mut inbox = Inbox::<u8>::new(4);
    let stops = Cell::new(0usize);
    let mut k = 0usize;
    let ran = run_loop(
        || {},
        || {
            stops.set(stops.get() + 1);
            stops.get() > iterations
        },
        &mut inbox,
        |_lot: &[u8]| -> Result<(), TickError> {
            k += 1;
            journal.push(Entry::TickStart(k));
            scheduler.run_tick(&mut world, &mut buffer)?;
            journal.push(Entry::TickEnd(k));
            // Lecture seule de `TRANSCRIPT` ; `EMISSIONS` n'est jamais lue ici.
            let Some(slot) = world.resources().transcript.get() else {
                panic!("violation d'assemblage : transcript absent");
            };
            present(&slot.0);
            journal.push(Entry::Present(k));
            Ok(())
        },
    );
    assert_eq!(ran, Ok(iterations as u64), "boucle");
    Ok(Execution { world, id })
}

/// Résultat d'une exécution nominale, sans emprunt.
struct Run {
    journal: Vec<Entry>,
    observed: Vec<Pairs>,
    final_state: [u8; ENCODED_LEN],
    beat: u32,
    last_transcript: Option<Pairs>,
}

fn nominal() -> Run {
    let set = reference_set();
    let artifacts = slices(&set);
    let mut journal = Vec::new();
    let mut observed: Vec<Pairs> = Vec::new();
    let execution = execute(
        set.manifest(),
        &artifacts,
        &SCRIPT_FORMATS,
        N,
        &mut journal,
        |acc| observed.push(pairs(acc)),
    )
    .expect("chargement");
    Run {
        journal,
        observed,
        final_state: capture(&execution.world, execution.id),
        beat: beat_of(&execution.world, execution.id),
        last_transcript: transcript(&execution.world),
    }
}

/// DM-1 : 12 itérations, 12 transcripts égaux au littéral, état final et `Beat`.
#[test]
fn dm1_twelve_transcripts_equal_the_literal() {
    let run = nominal();
    let expected: Vec<Pairs> = (1..=N).map(expected_transcript).collect();
    assert_eq!(run.observed.len(), N);
    assert_eq!(run.observed, expected);
    // Ticks sans émission observés vides.
    for k in [2, 3, 5, 6, 8, 9, 11, 12] {
        assert!(run.observed[k - 1].is_empty(), "tick {k}");
    }
    assert_eq!(run.final_state, final_state());
    assert_eq!(run.beat, 12);
}

/// Exécute un lot qui doit être refusé ; rend l'erreur, après avoir vérifié qu'aucun tick ni
/// aucune présentation n'a eu lieu.
fn assert_rejected(manifest: &[u8], artifacts: &[&[u8]], table: &[Format]) -> LoadError {
    let mut journal = Vec::new();
    let mut observed: Vec<Pairs> = Vec::new();
    let result = execute(manifest, artifacts, table, N, &mut journal, |acc| {
        observed.push(pairs(acc));
    });
    assert!(journal.is_empty(), "aucun tick");
    assert!(observed.is_empty(), "aucune présentation");
    result.err().expect("chargement rejeté")
}

/// Erreur d'artefact attendue : index d'artefact 0 et cause donnée.
fn assert_artifact_error(error: LoadError, expected: ReadError) {
    let LoadError::Artifact { index, error } = error else {
        panic!("attendu : erreur d'artefact, obtenu {error:?}");
    };
    assert_eq!(index.get(), 0);
    assert_eq!(error, expected);
}

/// DM-2 : quatre lots corrompus ; `load` échoue ; aucun tick, aucune présentation.
#[test]
fn dm2_corrupted_lots_fail_to_load_with_no_tick_and_no_presentation() {
    let set = reference_set();
    let good = set.artifacts()[0].clone();

    // (a) artefact tronqué d'un octet.
    let truncated = &good[..good.len() - 1];
    let error = assert_rejected(set.manifest(), &[truncated], &SCRIPT_FORMATS);
    assert_artifact_error(error, ReadError::Truncated);

    // (b) `Wait` de zéro tick : octet de ticks de l'enregistrement 2 mis à zéro.
    let mut wait_zero = good.clone();
    wait_zero[12 + 8 * 2 + 4] = 0;
    let error = assert_rejected(set.manifest(), &[wait_zero.as_slice()], &SCRIPT_FORMATS);
    assert_artifact_error(error, ReadError::Instruction);

    // (c) registre hors bornes : octet de registre de l'enregistrement 0.
    let mut bad_register = good.clone();
    bad_register[12 + 1] = u8::try_from(SCRIPT_REGISTERS).expect("registres sur un octet");
    let error = assert_rejected(set.manifest(), &[bad_register.as_slice()], &SCRIPT_FORMATS);
    assert_artifact_error(error, ReadError::Instruction);

    // (d) table de formats attendant un sprite au slot 4.
    let mut wrong = SCRIPT_FORMATS;
    wrong[usize::try_from(SLOT_A).expect("slot")] = Format::Sprite;
    let error = assert_rejected(set.manifest(), &[good.as_slice()], &wrong);
    assert_eq!(
        error,
        LoadError::SlotFormat {
            slot: SlotIndex::new(SLOT_A)
        }
    );
}

/// DM-3 : journal, présentation en lecture seule, historique tenu hors du `World`.
#[test]
fn dm3_the_journal_orders_tick_then_presentation_and_the_history_is_external() {
    let run = nominal();
    let expected: Vec<Entry> = (1..=N)
        .flat_map(|k| [Entry::TickStart(k), Entry::TickEnd(k), Entry::Present(k)])
        .collect();
    assert_eq!(run.journal, expected);
    // L'historique complet est celui de l'observateur ; le `World` ne garde que le dernier
    // transcript, ici celui du tick 12, vide.
    assert_eq!(run.observed.len(), N);
    assert_eq!(run.last_transcript, Some(Vec::new()));
    assert!(run.observed.iter().any(|pairs| !pairs.is_empty()));

    // La présentation reçoit `&Acc` : signature en lecture seule.
    fn accept<P: FnMut(&Acc)>(_: P) {}
    accept(|acc| {
        let _ = acc.len();
    });
}
