// crates/engine/shell/tests/script_suspend.rs

//! Étape 6, unité U4 : intégration au Scheduler, interface fixe, suspension et reprise.
//!
//! - CT-1 à CT-3 : contrats typés des systèmes, propriété de `STATE`, rôles des catégories ;
//! - SY-0 : montage emprunté (octets construits à l'exécution, table chargée par le vrai `load`) ;
//! - SR-1 à SR-7 : transcript, reconstruction, attente persistante, remplacement de
//!   l'ordonnanceur, deux instances, lancement différé, budget accepté par la Forge.
//!
//! Les scripts sont compilés par la Forge et déposés par le vrai `load`. Aucun test n'affirme
//! l'ordre des émissions entre instances ni l'ordre dense d'un stockage : les comparaisons
//! entre instances se font en multiensemble (tri, multiplicité conservée).
//!
//! **Non démontré** : démonstrateur de bout en bout, cadence réelle, backend graphique (U5).

#[path = "script_support/mod.rs"]
#[allow(
    dead_code,
    reason = "les variantes de `Remove` décrivent le stockage ; U4 n'en construit aucune"
)]
mod support;

use verbum_assets::{Asset, SlotIndex, load};
use verbum_forge::{ForgeError, compile_script};
use verbum_schedule::contract::{Contract, StructuralEffects};
use verbum_schedule::data::DataSet;
use verbum_schedule::model::ModelError;
use verbum_schedule::scheduler::Scheduler;
use verbum_schedule::system::{SystemList, contract_of};
use verbum_script::{ENCODED_LEN, Fault, ScriptState, ScriptTable, Status, step};

use support::{
    Acc, B, BEAT, Buf, Clear, Collect, Drift, EMISSIONS, Interpreter, MAX_STEPS, PHASES,
    RIVAL_SYSTEMS, RivalSystems, SCRIPT_FORMATS, SCRIPT_SLOTS, SCRIPTS, SLOT_A, SLOT_B, STATE,
    SYSTEMS, Sched, Systems, TRANSCRIPT, W, WideInterpreter, beat_of, build_world, capture,
    compile_scripts, launch, launch_observed, load_scripts, multiset, pair_set, reconstruct,
    reference_set, slices, tick, transcript,
};

// ---------------------------------------------------------------------------
// Littéraux de référence (écrits à la main)
// ---------------------------------------------------------------------------

type Pairs = Vec<(u16, i32)>;

/// Encodage canonique de 26 octets : les 14 premiers sont donnés, les 12 derniers (`r1` à `r3`)
/// sont nuls.
fn encoding(head: [u8; 14]) -> [u8; ENCODED_LEN] {
    let mut bytes = [0u8; ENCODED_LEN];
    bytes[..14].copy_from_slice(&head);
    bytes
}

fn row_0() -> [u8; ENCODED_LEN] {
    encoding([0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
}
fn row_1() -> [u8; ENCODED_LEN] {
    encoding([0x04, 0, 0x03, 0, 0x01, 0, 0x03, 0, 0, 0, 0x03, 0, 0, 0])
}
fn row_2() -> [u8; ENCODED_LEN] {
    encoding([0x04, 0, 0x03, 0, 0x01, 0, 0x02, 0, 0, 0, 0x03, 0, 0, 0])
}
fn row_3() -> [u8; ENCODED_LEN] {
    encoding([0x04, 0, 0x03, 0, 0x01, 0, 0x01, 0, 0, 0, 0x03, 0, 0, 0])
}
fn row_4() -> [u8; ENCODED_LEN] {
    encoding([0x04, 0, 0x03, 0, 0x01, 0, 0x03, 0, 0, 0, 0x02, 0, 0, 0])
}
fn row_10() -> [u8; ENCODED_LEN] {
    encoding([0x04, 0, 0x06, 0, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0])
}

/// Transcript attendu du script A seul, au tick `k` (1 à 12).
fn expected_transcript(k: usize) -> Pairs {
    match k {
        1 | 4 | 7 => vec![(1, 10)],
        10 => vec![(2, 99)],
        _ => Vec::new(),
    }
}

/// Trajectoire de la référence : encodages après `t` ticks (`t` de 0 à 12) et transcripts des
/// ticks 1 à 12 (indice `k - 1`). Valeurs possédées : aucun emprunt ne survit à l'appel.
struct Reference {
    encodings: Vec<[u8; ENCODED_LEN]>,
    transcripts: Vec<Pairs>,
}

fn reference_run() -> Reference {
    let set = reference_set();
    let artifacts = slices(&set);
    let mut world = build_world();
    load_scripts(&mut world, set.manifest(), &artifacts).expect("chargement");
    let mut buffer = Buf::new();
    let id = launch(&mut world, &mut buffer, SLOT_A, 0);
    let scheduler = Sched::new().expect("modèle valide");
    let mut encodings = vec![capture(&world, id)];
    let mut transcripts = Vec::new();
    for _ in 1..=12 {
        tick(&scheduler, &mut world, &mut buffer);
        encodings.push(capture(&world, id));
        transcripts.push(transcript(&world).expect("transcript présent"));
    }
    Reference {
        encodings,
        transcripts,
    }
}

// ---------------------------------------------------------------------------
// CT-1 à CT-3 : interface fixe
// ---------------------------------------------------------------------------

fn contract(reads: DataSet, writes: DataSet, produces: DataSet) -> Contract {
    Contract::new(reads, writes, produces, StructuralEffects::NONE)
}

/// CT-1 : le contrat de `Interpreter` est le littéral attendu ; `WideInterpreter` (un accès de
/// plus) ne l'est pas. `B<'static>` ne vaut que pour ce contrôle : le contrat est indépendant de
/// `'a`, et les montages exécutés emploient une durée de vie locale.
#[test]
fn ct1_the_interpreter_contract_is_the_fixed_interface() {
    let interpreter = contract_of::<B<'static>, Interpreter>();
    assert_eq!(
        interpreter,
        contract(
            DataSet::of(SCRIPTS),
            DataSet::of(STATE),
            DataSet::of(EMISSIONS)
        )
    );
    // Les trois autres systèmes ont leurs littéraux propres.
    assert_eq!(
        contract_of::<B<'static>, Clear>(),
        contract(DataSet::EMPTY, DataSet::EMPTY, DataSet::of(EMISSIONS))
    );
    assert_eq!(
        contract_of::<B<'static>, Collect>(),
        contract(
            DataSet::of(EMISSIONS),
            DataSet::EMPTY,
            DataSet::of(TRANSCRIPT)
        )
    );
    assert_eq!(
        contract_of::<B<'static>, Drift>(),
        contract(DataSet::EMPTY, DataSet::of(BEAT), DataSet::EMPTY)
    );
    // Un accès supplémentaire change le contrat : le contrôle échoue.
    assert_ne!(contract_of::<B<'static>, WideInterpreter>(), interpreter);
}

fn filler() -> Contract {
    contract(DataSet::EMPTY, DataSet::EMPTY, DataSet::EMPTY)
}

/// Contrats de la liste de l'assemblage, dans l'ordre de déclaration : `Clear` (0), `Collect` (1),
/// `Interpreter` (2), `Drift` (3).
fn assembled_contracts() -> [(usize, Contract); SYSTEMS] {
    let mut out = [(0usize, filler()); SYSTEMS];
    <Systems as SystemList<B<'static>>>::fill(&mut out, 0);
    out
}

const CLEAR: usize = 0;
const COLLECT: usize = 1;
const INTERPRETER: usize = 2;

/// CT-2 : seul `Interpreter` écrit ou produit `STATE` ; un second écrivain dans la phase 1 est
/// refusé à la construction du `Scheduler`.
#[test]
fn ct2_state_has_a_single_writer() {
    let contracts = assembled_contracts();
    let state = DataSet::of(STATE);
    for (index, (_, contract)) in contracts.iter().enumerate() {
        let touches = contract
            .writes()
            .union(contract.produces())
            .intersects(state);
        assert_eq!(touches, index == INTERPRETER, "système {index}");
    }
    assert!(Sched::new().is_ok());

    let rival = Scheduler::<B<'_>, RivalSystems, RIVAL_SYSTEMS, PHASES>::new();
    assert_eq!(rival.err(), Some(ModelError::MultipleWriters));
}

/// CT-3 : rôles de `EMISSIONS` et `TRANSCRIPT`, et phases.
#[test]
fn ct3_emissions_roles_and_phases() {
    let contracts = assembled_contracts();
    let emissions = DataSet::of(EMISSIONS);
    let transcript_set = DataSet::of(TRANSCRIPT);
    for (index, (_, contract)) in contracts.iter().enumerate() {
        assert_eq!(
            contract.reads().intersects(emissions),
            index == COLLECT,
            "lecteur de EMISSIONS : système {index}"
        );
        assert_eq!(
            contract.produces().intersects(emissions),
            index == CLEAR || index == INTERPRETER,
            "producteur de EMISSIONS : système {index}"
        );
        assert_eq!(
            contract.produces().intersects(transcript_set),
            index == COLLECT,
            "producteur de TRANSCRIPT : système {index}"
        );
    }
    assert!(contracts[CLEAR].0 < contracts[COLLECT].0);
    assert!(contracts.iter().all(|&(phase, _)| phase <= 1));
}

// ---------------------------------------------------------------------------
// SY-0 : montage emprunté
// ---------------------------------------------------------------------------

/// Script de même forme pour toute valeur : `emit 1,v ; wait 1 ; emit 1,v+1 ; halt`.
fn shaped_source(value: i32) -> String {
    format!(
        "[[step]]\nop = \"emit\"\nchannel = 1\nvalue = {value}\n\n\
         [[step]]\nop = \"wait\"\nticks = 1\n\n\
         [[step]]\nop = \"emit\"\nchannel = 1\nvalue = {}\n\n\
         [[step]]\nop = \"halt\"\n",
        value + 1
    )
}

/// Exécute 4 ticks sur un lot emprunté ; le `World` est déclaré après le lot (G-LT).
fn transcripts_over<'a>(manifest: &[u8], artifacts: &[&'a [u8]]) -> Vec<Pairs> {
    let mut world: W<'a> = build_world();
    load_scripts(&mut world, manifest, artifacts).expect("chargement");
    let mut buffer = Buf::new();
    launch(&mut world, &mut buffer, SLOT_A, 0);
    let scheduler = Sched::new().expect("modèle valide");
    let mut seen = Vec::new();
    for _ in 0..4 {
        tick(&scheduler, &mut world, &mut buffer);
        seen.push(transcript(&world).expect("transcript présent"));
    }
    seen
}

/// SY-0 : deux tampons construits à l'exécution, déclarés avant les `World` ; les transcripts
/// dépendent des octets empruntés.
#[test]
fn sy0_the_borrowed_mounting_reads_the_runtime_bytes() {
    let first_source = shaped_source(10);
    let second_source = shaped_source(20);
    let first = compile_scripts(&[("s", first_source.as_str(), SLOT_A)]);
    let second = compile_scripts(&[("s", second_source.as_str(), SLOT_A)]);
    let first_bytes = slices(&first);
    let second_bytes = slices(&second);

    let on_first = transcripts_over(first.manifest(), &first_bytes);
    let on_second = transcripts_over(second.manifest(), &second_bytes);

    assert_eq!(
        on_first,
        vec![vec![(1, 10)], vec![(1, 11)], Vec::new(), Vec::new()]
    );
    assert_eq!(
        on_second,
        vec![vec![(1, 20)], vec![(1, 21)], Vec::new(), Vec::new()]
    );
    assert_ne!(on_first, on_second);
}

// ---------------------------------------------------------------------------
// SR-1 : transcript du tick et Beat
// ---------------------------------------------------------------------------

/// SR-1 : scénario de référence sur 12 ticks.
#[test]
fn sr1_the_transcript_is_the_emissions_of_the_current_tick() {
    let set = reference_set();
    let artifacts = slices(&set);
    let mut world = build_world();
    load_scripts(&mut world, set.manifest(), &artifacts).expect("chargement");
    let mut buffer = Buf::new();
    let id = launch(&mut world, &mut buffer, SLOT_A, 0);
    let scheduler = Sched::new().expect("modèle valide");

    assert_eq!(
        transcript(&world),
        Some(Vec::new()),
        "présent et vide avant le premier tick"
    );

    // Observateur extérieur : un `Vec` de l'appelant.
    let mut observed: Vec<Pairs> = Vec::new();
    for k in 1..=12usize {
        tick(&scheduler, &mut world, &mut buffer);
        let current = transcript(&world).expect("transcript présent");
        assert_eq!(current, expected_transcript(k), "tick {k}");
        assert_eq!(
            u64::from(beat_of(&world, id)),
            k as u64,
            "Beat après le tick {k}"
        );
        observed.push(current);
    }
    let expected: Vec<Pairs> = (1..=12).map(expected_transcript).collect();
    assert_eq!(observed, expected);
    assert_eq!(beat_of(&world, id), 12);
}

// ---------------------------------------------------------------------------
// SR-2 : reconstruction depuis les seuls octets capturés
// ---------------------------------------------------------------------------

/// SR-2 : pour chaque `t` de 0 à 12, reconstruction depuis la capture de 26 octets.
#[test]
fn sr2_a_reconstruction_resumes_from_the_captured_bytes_only() {
    let reference = reference_run();
    let set = reference_set();
    let artifacts = slices(&set);
    for t in 0..=12usize {
        let captured = reference.encodings[t];
        // (a) aller-retour de l'encodage.
        let decoded = ScriptState::from_bytes(&captured).expect("capture valide");
        assert_eq!(decoded.to_bytes(), captured, "t = {t}");

        // `Beat` initial 100 : il n'influence aucune assertion ci-dessous.
        let (mut world, mut buffer, id) = reconstruct(set.manifest(), &artifacts, &captured, 100);
        let scheduler = Sched::new().expect("modèle valide");
        for k in (t + 1)..=12 {
            tick(&scheduler, &mut world, &mut buffer);
            // (b) encodage égal à celui de la référence.
            assert_eq!(
                capture(&world, id),
                reference.encodings[k],
                "t = {t}, tick {k}"
            );
            // (c) transcript du suffixe égal au littéral.
            assert_eq!(
                transcript(&world).expect("transcript présent"),
                expected_transcript(k),
                "t = {t}, tick {k}"
            );
        }
        // (d) état final égal à la ligne « 10 ».
        assert_eq!(capture(&world, id), row_10(), "t = {t}");
    }
}

// ---------------------------------------------------------------------------
// SR-3 : attente persistante
// ---------------------------------------------------------------------------

/// SR-3 : `wait` persistant dans l'état ; reprise au tick 4.
#[test]
fn sr3_the_wait_is_persisted_in_the_state() {
    let reference = reference_run();
    let state = |t: usize| ScriptState::from_bytes(&reference.encodings[t]).expect("état valide");

    // Décompte persistant 3, 2, 1 en fin de ticks 1, 2, 3.
    assert_eq!(
        [state(1).wait(), state(2).wait(), state(3).wait()],
        [3, 2, 1]
    );

    // Aux ticks 2 et 3, tous les octets autres que `wait` (octets 6 à 9) sont constants.
    let without_wait = |bytes: [u8; ENCODED_LEN]| {
        let mut copy = bytes;
        copy[6..10].fill(0);
        copy
    };
    assert_eq!(
        without_wait(reference.encodings[2]),
        without_wait(reference.encodings[1])
    );
    assert_eq!(
        without_wait(reference.encodings[3]),
        without_wait(reference.encodings[1])
    );

    // Aucune valeur 0 de `wait` en fin de tick, ticks 1 à 9.
    for t in 1..=9 {
        assert_ne!(state(t).wait(), 0, "fin du tick {t}");
    }

    // Reprise au tick 4 : r0 3 → 2, pc revenu à 3, wait réarmé à 3, émission (1,10).
    assert_eq!(state(3).registers()[0], 3);
    let resumed = state(4);
    assert_eq!(resumed.registers()[0], 2);
    assert_eq!(resumed.pc(), 3);
    assert_eq!(resumed.wait(), 3);
    assert_eq!(reference.transcripts[3], vec![(1, 10)]);

    // Octets de fin de ticks 1, 2, 3, 4 et 10 égaux aux littéraux.
    assert_eq!(reference.encodings[0], row_0());
    assert_eq!(reference.encodings[1], row_1());
    assert_eq!(reference.encodings[2], row_2());
    assert_eq!(reference.encodings[3], row_3());
    assert_eq!(reference.encodings[4], row_4());
    assert_eq!(reference.encodings[10], row_10());
}

// ---------------------------------------------------------------------------
// SR-4 : remplacement de l'ordonnanceur
// ---------------------------------------------------------------------------

/// SR-4 : un `Scheduler::new()` neuf entre les ticks 4 et 5, sur le même monde.
#[test]
fn sr4_a_fresh_scheduler_continues_the_same_world() {
    let reference = reference_run();
    let set = reference_set();
    let artifacts = slices(&set);
    let mut world = build_world();
    load_scripts(&mut world, set.manifest(), &artifacts).expect("chargement");
    let mut buffer = Buf::new();
    let id = launch(&mut world, &mut buffer, SLOT_A, 0);

    {
        let first = Sched::new().expect("modèle valide");
        for _ in 1..=4 {
            tick(&first, &mut world, &mut buffer);
        }
    }
    let second = Sched::new().expect("modèle valide");
    for k in 5..=12usize {
        tick(&second, &mut world, &mut buffer);
        assert_eq!(capture(&world, id), reference.encodings[k], "tick {k}");
        assert_eq!(
            transcript(&world).expect("transcript présent"),
            reference.transcripts[k - 1],
            "tick {k}"
        );
    }
}

// ---------------------------------------------------------------------------
// SR-5 : deux instances
// ---------------------------------------------------------------------------

/// Exécute `ticks` ticks et rend, par tick, le transcript en forme normale de multiensemble.
fn run_multiset<'a>(
    scheduler: &Sched<'a>,
    world: &mut W<'a>,
    buffer: &mut Buf<'a>,
    ticks: std::ops::RangeInclusive<usize>,
    out: &mut Vec<Pairs>,
) {
    for _ in ticks {
        tick(scheduler, world, buffer);
        out.push(multiset(transcript(world).expect("transcript présent")));
    }
}

/// SR-5, variante (i) : B lancé par `apply` après le tick 2.
#[test]
fn sr5_variant_i_a_second_instance_launched_after_tick_two() {
    let set = pair_set();
    let artifacts = slices(&set);
    let mut world = build_world();
    load_scripts(&mut world, set.manifest(), &artifacts).expect("chargement");
    let mut buffer = Buf::new();
    let a = launch(&mut world, &mut buffer, SLOT_A, 0);
    let scheduler = Sched::new().expect("modèle valide");

    let mut seen: Vec<Pairs> = Vec::new();
    run_multiset(&scheduler, &mut world, &mut buffer, 1..=2, &mut seen);
    let b = launch(&mut world, &mut buffer, SLOT_B, 0);
    run_multiset(&scheduler, &mut world, &mut buffer, 3..=12, &mut seen);

    let expected: Vec<Pairs> = (1..=12usize)
        .map(|k| match k {
            1 | 7 => vec![(1, 10)],
            3 => vec![(3, 7)],
            4 => multiset(vec![(1, 10), (3, 8)]),
            10 => vec![(2, 99)],
            _ => Vec::new(),
        })
        .collect();
    assert_eq!(seen, expected);

    // Chaque instance exécute le script de son slot.
    assert_eq!(capture(&world, a), row_10());
    let b_state = ScriptState::from_bytes(&capture(&world, b)).expect("état valide");
    assert_eq!(
        (b_state.script(), b_state.status(), b_state.pc()),
        (1, Status::Halted, 3)
    );
}

/// SR-5, variante (ii) : A et B lancés avant le tick 1, dans les deux ordres.
#[test]
fn sr5_variant_ii_two_instances_launched_before_the_first_tick() {
    let set = pair_set();
    let artifacts = slices(&set);
    let expected: Vec<Pairs> = (1..=12usize)
        .map(|k| match k {
            1 => multiset(vec![(1, 10), (3, 7)]),
            2 => vec![(3, 8)],
            4 | 7 => vec![(1, 10)],
            10 => vec![(2, 99)],
            _ => Vec::new(),
        })
        .collect();

    let mut finals = Vec::new();
    for order in [[SLOT_A, SLOT_B], [SLOT_B, SLOT_A]] {
        let mut world = build_world();
        load_scripts(&mut world, set.manifest(), &artifacts).expect("chargement");
        let mut buffer = Buf::new();
        let ids = order.map(|slot| (slot, launch(&mut world, &mut buffer, slot, 0)));
        let scheduler = Sched::new().expect("modèle valide");
        let mut seen: Vec<Pairs> = Vec::new();
        run_multiset(&scheduler, &mut world, &mut buffer, 1..=12, &mut seen);
        assert_eq!(seen, expected, "ordre {order:?}");

        // États finaux par instance, désignées par leur slot.
        let state_of = |wanted: u32| {
            let Some(&(_, id)) = ids.iter().find(|&&(slot, _)| slot == wanted) else {
                panic!("instance absente");
            };
            capture(&world, id)
        };
        finals.push((state_of(SLOT_A), state_of(SLOT_B)));
    }
    assert_eq!(finals[0], finals[1]);
    assert_eq!(finals[0].0, row_10());
}

// ---------------------------------------------------------------------------
// SR-6 : lancement par commande différée
// ---------------------------------------------------------------------------

/// SR-6 : la commande est ajoutée au tampon, le composant reste absent, puis `apply` le rend
/// présent.
#[test]
fn sr6_a_deferred_launch_is_visible_only_after_apply() {
    let set = reference_set();
    let artifacts = slices(&set);
    let mut world = build_world();
    load_scripts(&mut world, set.manifest(), &artifacts).expect("chargement");
    let mut buffer = Buf::new();

    let id = launch_observed(&mut world, &mut buffer, SLOT_A, 0, |world, buffer, id| {
        // (1) les commandes sont dans le tampon (état et `Beat`).
        assert_eq!(buffer.len(), 2, "commandes ajoutées au tampon");
        // (2) le composant reste absent du `World`.
        assert!(
            world.components().states.get(id).is_none(),
            "état présent avant apply"
        );
        assert!(!world.components().states.contains(id));
    });

    // (3) `apply` a été exécuté ; (4) le composant est présent.
    assert!(buffer.is_empty());
    let Some(state) = world.components().states.get(id) else {
        panic!("état absent après apply");
    };
    assert_eq!(state.status(), Status::Running);
    assert_eq!(state.pc(), 0);
    assert_eq!(state.registers(), [0; verbum_script::SCRIPT_REGISTERS]);
    assert_eq!(state.script(), 4);
}

// ---------------------------------------------------------------------------
// SR-7 : un script accepté par la Forge n'atteint jamais `Faulted(Budget)`
// ---------------------------------------------------------------------------

/// Terminaison d'un segment rectiligne.
#[derive(Clone, Copy, Debug)]
enum End {
    Wait,
    Halt,
    Program,
}

const ADD: &str = "[[step]]\nop = \"add\"\nregister = 1\nvalue = 1\n\n";
const WAIT: &str = "[[step]]\nop = \"wait\"\nticks = 1\n\n";
const HALT: &str = "[[step]]\nop = \"halt\"\n\n";

/// Segment rectiligne de `len` instructions : `Halt` et `Wait` comptent pour une instruction, la
/// fin du programme pour aucune. Longueur du plus long chemin : `len`.
fn straight(len: usize, end: End) -> String {
    let plain = match end {
        End::Program => len,
        End::Wait | End::Halt => len - 1,
    };
    let mut source = ADD.repeat(plain);
    match end {
        End::Wait => source.push_str(WAIT),
        End::Halt => source.push_str(HALT),
        End::Program => {}
    }
    source
}

/// Branchement dont un bras est de longueur `m + 1` et l'autre de longueur 1 ; plus long chemin
/// `m + 3` (`set`, `jnz`, bras long). `long_is_taken` place le bras long côté saut ; `register`
/// vaut la valeur mise dans `r0` avant le saut (zéro : saut non pris, non nul : saut pris).
fn branch(m: usize, long_is_taken: bool, register: i32) -> String {
    let mut source = format!("[[step]]\nop = \"set\"\nregister = 0\nvalue = {register}\n\n");
    let target = if long_is_taken { "long" } else { "short" };
    source.push_str(&format!(
        "[[step]]\nop = \"jnz\"\nregister = 0\ntarget = \"{target}\"\n\n"
    ));
    let long_arm = |label: Option<&str>| {
        let mut arm = String::new();
        let labelled = |body: &str, first: bool| match (label, first) {
            (Some(name), true) => {
                body.replacen("[[step]]\n", &format!("[[step]]\nlabel = \"{name}\"\n"), 1)
            }
            _ => body.to_string(),
        };
        for index in 0..m {
            arm.push_str(&labelled(ADD, index == 0));
        }
        arm.push_str(&labelled(HALT, m == 0));
        arm
    };
    let short_arm = |label: Option<&str>| match label {
        Some(name) => HALT.replacen("[[step]]\n", &format!("[[step]]\nlabel = \"{name}\"\n"), 1),
        None => HALT.to_string(),
    };
    if long_is_taken {
        source.push_str(&short_arm(None));
        source.push_str(&long_arm(Some("long")));
    } else {
        source.push_str(&long_arm(None));
        source.push_str(&short_arm(Some("short")));
    }
    source
}

/// Plus long chemin d'un branchement de paramètre `m`.
const fn branch_longest(m: usize) -> usize {
    m + 3
}

/// Compile par la Forge, dépose par le vrai `load`, exécute par `step` sur plusieurs ticks.
/// `step` ne rend aucun compteur d'instructions : aucune mesure directe n'est affirmée.
fn run_accepted(source: &str) {
    let set = compile_scripts(&[("s", source, 0)]);
    let artifacts = slices(&set);
    let mut table = ScriptTable::<SCRIPT_SLOTS>::new();
    let loaded = load(
        set.manifest(),
        &artifacts,
        &SCRIPT_FORMATS,
        |slot, asset| {
            if let Asset::Script(script) = asset {
                assert!(table.deposit(slot, script).is_ok(), "dépôt refusé");
            }
        },
    );
    assert!(loaded.is_ok(), "chargement refusé");

    let Ok(mut state) = ScriptState::start(SlotIndex::new(0)) else {
        panic!("slot hors de la borne de ScriptState");
    };
    let mut sink = Acc::new();
    for tick in 0..(MAX_STEPS + 4) {
        sink.clear();
        assert!(
            step(table.get(state.script()), &mut state, &mut sink).is_ok(),
            "saturation des émissions"
        );
        assert_ne!(
            state.status(),
            Status::Faulted(Fault::Budget),
            "tick {tick}\n{source}"
        );
    }
    assert!(state.pc() > 0, "le script n'a pas progressé\n{source}");
}

/// Vérifie une source dont le plus long chemin vaut `longest`.
fn check_source(source: &str, longest: usize) {
    let compiled = compile_script(source);
    if longest <= MAX_STEPS {
        assert!(
            compiled.is_ok(),
            "refusé à tort (longueur {longest})\n{source}"
        );
        run_accepted(source);
    } else {
        assert!(
            matches!(compiled, Err(ForgeError::BudgetExceeded { .. })),
            "accepté à tort (longueur {longest})\n{source}"
        );
    }
}

/// SR-7 : famille générée à partir de `SCRIPT_MAX_STEPS_PER_TICK`. Sous Miri, paire limite
/// seulement (E-7).
#[test]
fn sr7_a_script_accepted_by_the_forge_never_faults_on_budget() {
    let ends = [End::Wait, End::Halt, End::Program];
    if cfg!(miri) {
        // Un segment de `MAX` accepté et exécuté, un segment de `MAX + 1` refusé.
        check_source(&straight(MAX_STEPS, End::Halt), MAX_STEPS);
        check_source(&straight(MAX_STEPS + 1, End::Halt), MAX_STEPS + 1);
        // Un branchement à bras long accepté et exécuté, avec ses deux issues.
        let m = MAX_STEPS - 3;
        for register in [0, 1] {
            check_source(&branch(m, true, register), branch_longest(m));
        }
        return;
    }
    for len in [MAX_STEPS - 1, MAX_STEPS, MAX_STEPS + 1, MAX_STEPS + 2] {
        for end in ends {
            check_source(&straight(len, end), len);
        }
        // Branchements : l'un ou l'autre bras est le plus long, deux issues du saut.
        let m = len - 3;
        for long_is_taken in [false, true] {
            for register in [0, 1] {
                check_source(&branch(m, long_is_taken, register), branch_longest(m));
            }
        }
    }
}
