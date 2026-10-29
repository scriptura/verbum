// crates/render/tests/pipeline.rs

//! Pipeline sous Scheduler (U3) : RN-1 à RN-6.
//!
//! `World` et `Scheduler` réels. Deux producteurs indépendants (`ProduceA`,
//! `ProduceB`) alimentent l'accumulateur ; un seul système (`Compose`) le lit et
//! produit la `Frame` ; `Witness` copie la `Frame` en lecture seule. `Clear`, en
//! phase 0, remet l'accumulateur à l'état initial (plan C4, écart E-1).
//!
//! Les comparaisons de `Frame` se font en multiensemble, multiplicité conservée
//! (plan A-1 = Y, E10) ; aucune assertion n'affirme l'ordre de la séquence de la
//! `Frame` (plan C8), et RN-3 n'affirme aucun ordre entre `ProduceA` et
//! `ProduceB`.
//!
//! Politique des producteurs (document d'exécution § 3.2, plan C7), valable pour
//! ce scénario seulement : un `Err(Full)` du `push` ou un emplacement absent
//! provoque un `panic!` explicite. Il n'est ni ignoré ni converti en
//! `BufferFull`. Le scénario nominal ne sature jamais l'accumulateur.
//!
//! Ce que ces tests n'établissent pas : la durée de vie logique de l'accumulateur
//! (plan C4, E-5) n'est pas imposée par les types. Le contrôle de composition
//! (RN-2) ne porte que sur les contrats des systèmes assemblés ; il ne peut pas
//! interdire une lecture directe de la ressource depuis le code extérieur au
//! Scheduler.

use core::convert::Infallible;

use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
use verbum_render::{Contribution, Contributions, Frame};
use verbum_schedule::access::{Locate, Produce, Read};
use verbum_schedule::contract::{Contract, StructuralEffects};
use verbum_schedule::data::{DataId, DataSet};
use verbum_schedule::scheduler::Scheduler;
use verbum_schedule::system::{At, Binding, Cons, Nil, System, SystemList};

// ---------------------------------------------------------------------------
// Constantes de scénario (document d'exécution § 3.2)
// ---------------------------------------------------------------------------

/// Capacité de l'accumulateur et de la `Frame`.
const CAP: usize = 4;

/// Contribution carrée partagée par `ProduceA` et `ProduceB` : doublon voulu.
const Q1: Contribution = Contribution {
    x: 0,
    y: 0,
    width: 2,
    height: 2,
    color: [255, 0, 0, 255],
};

/// Seconde contribution carrée, émise par `ProduceA` seulement.
const Q2: Contribution = Contribution {
    x: 4,
    y: 1,
    width: 3,
    height: 3,
    color: [0, 255, 0, 255],
};

/// Sortie exacte de `ProduceA` à chaque tick.
const A_SORTIE: [Contribution; 2] = [Q1, Q2];

/// Sortie exacte de `ProduceB` à chaque tick.
const B_SORTIE: [Contribution; 1] = [Q1];

/// Borne de production par tick : somme des sorties des producteurs.
const MAX_PAR_TICK: usize = A_SORTIE.len() + B_SORTIE.len();

// Ne prouve que l'arithmétique des constantes ; le lien avec le code des
// producteurs est vérifié par revue (document d'exécution § 3.2, point 4).
const _: () = assert!(MAX_PAR_TICK <= CAP);

// ---------------------------------------------------------------------------
// Catégories, monde, assemblage
// ---------------------------------------------------------------------------

type Acc = Contributions<CAP>;
type Fr = Frame<CAP>;

const ACC: u8 = 0;
const FRAME: u8 = 1;
const WITNESS: u8 = 2;

#[derive(Default)]
struct Globals {
    acc: ResourceSlot<Acc>,
    frame: ResourceSlot<Fr>,
    witness: ResourceSlot<Fr>,
}

type TestWorld = World<1, (), Globals>;
type Buffer = CommandBuffer<1, Infallible, Infallible>;

struct Parts<'w> {
    acc: Option<&'w mut ResourceSlot<Acc>>,
    frame: Option<&'w mut ResourceSlot<Fr>>,
    witness: Option<&'w mut ResourceSlot<Fr>>,
}

impl<'w> Locate<'w, ACC> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Acc>;
    fn take(&mut self) -> Option<Self::Store> {
        self.acc.take()
    }
}

impl<'w> Locate<'w, FRAME> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Fr>;
    fn take(&mut self) -> Option<Self::Store> {
        self.frame.take()
    }
}

impl<'w> Locate<'w, WITNESS> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Fr>;
    fn take(&mut self) -> Option<Self::Store> {
        self.witness.take()
    }
}

struct TestBinding;

impl Binding for TestBinding {
    type World = TestWorld;
    type Buffer = Buffer;
    type Parts<'w> = Parts<'w>;

    fn parts<'w>(world: &'w mut TestWorld, _buffer: &'w mut Buffer) -> Parts<'w> {
        let ((), globals) = world.split_mut();
        Parts {
            acc: Some(&mut globals.acc),
            frame: Some(&mut globals.frame),
            witness: Some(&mut globals.witness),
        }
    }
}

type ProduceAcc<'w> = Produce<ACC, <Parts<'w> as Locate<'w, ACC>>::Store>;
type ReadAcc<'w> = Read<ACC, <Parts<'w> as Locate<'w, ACC>>::Store>;
type ProduceFrame<'w> = Produce<FRAME, <Parts<'w> as Locate<'w, FRAME>>::Store>;
type ReadFrame<'w> = Read<FRAME, <Parts<'w> as Locate<'w, FRAME>>::Store>;
type ProduceWitness<'w> = Produce<WITNESS, <Parts<'w> as Locate<'w, WITNESS>>::Store>;

// ---------------------------------------------------------------------------
// Systèmes (types distincts ; aucun ne nomme un autre)
// ---------------------------------------------------------------------------

/// Phase 0 : remet l'accumulateur à l'état initial (propriétaire de durée de vie).
struct Clear;

impl System<TestBinding> for Clear {
    type Access<'w> = (ProduceAcc<'w>,);

    fn run((mut acc,): Self::Access<'_>) -> Result<(), BufferFull> {
        acc.insert(Acc::new());
        Ok(())
    }
}

/// Phase 1 : unique lecteur de l'accumulateur, unique producteur de la `Frame`.
struct Compose;

impl System<TestBinding> for Compose {
    type Access<'w> = (ReadAcc<'w>, ProduceFrame<'w>);

    fn run((acc, mut frame): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(source) = acc.get() else {
            panic!("violation d'assemblage : accumulateur absent");
        };
        frame.insert(Fr::compose(source));
        Ok(())
    }
}

/// Phase 1 : lecture seule de la `Frame`, copie dans une catégorie de test.
struct Witness;

impl System<TestBinding> for Witness {
    type Access<'w> = (ReadFrame<'w>, ProduceWitness<'w>);

    fn run((frame, mut witness): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(frame) = frame.get() else {
            panic!("violation d'assemblage : frame absente");
        };
        witness.insert(frame.clone());
        Ok(())
    }
}

/// Phase 1 : producteur indépendant, émet exactement `A_SORTIE`.
struct ProduceA;

impl System<TestBinding> for ProduceA {
    type Access<'w> = (ProduceAcc<'w>,);

    fn run((mut acc,): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(acc) = acc.get_mut() else {
            panic!("violation d'assemblage : accumulateur absent");
        };
        for contribution in A_SORTIE {
            if acc.push(contribution).is_err() {
                panic!("borne de production dépassée : accumulateur plein (ProduceA)");
            }
        }
        Ok(())
    }
}

/// Phase 1 : producteur indépendant, émet exactement `B_SORTIE`.
struct ProduceB;

impl System<TestBinding> for ProduceB {
    type Access<'w> = (ProduceAcc<'w>,);

    fn run((mut acc,): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(acc) = acc.get_mut() else {
            panic!("violation d'assemblage : accumulateur absent");
        };
        for contribution in B_SORTIE {
            if acc.push(contribution).is_err() {
                panic!("borne de production dépassée : accumulateur plein (ProduceB)");
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Modèle d'exécution
// ---------------------------------------------------------------------------

const SYSTEMS: usize = 5;
const PHASES: usize = 2;

// Indices de déclaration. `Compose` est déclaré AVANT ses producteurs afin que
// l'ordre exécuté ne puisse venir que de la déduction.
const CLEAR: usize = 0;
const COMPOSE: usize = 1;
const WITNESS_SYSTEM: usize = 2;
const PRODUCE_A: usize = 3;
const PRODUCE_B: usize = 4;

type Systems = Cons<
    At<0, Clear>,
    Cons<At<1, Compose>, Cons<At<1, Witness>, Cons<At<1, ProduceA>, Cons<At<1, ProduceB>, Nil>>>>,
>;

type TestScheduler = Scheduler<TestBinding, Systems, SYSTEMS, PHASES>;

fn scheduler() -> TestScheduler {
    TestScheduler::new().expect("un modèle valide")
}

/// Monde assemblé : accumulateur, `Frame` et témoin initialisés à l'état initial.
fn assembled_world() -> TestWorld {
    let mut world = TestWorld::new();
    let globals = world.resources_mut();
    globals.acc.insert(Acc::new());
    globals.frame.insert(Fr::empty());
    globals.witness.insert(Fr::empty());
    world
}

fn tick(scheduler: &TestScheduler, world: &mut TestWorld, buffer: &mut Buffer) {
    scheduler
        .run_tick(world, buffer)
        .expect("le tick se termine");
}

fn frame_of(world: &TestWorld) -> &Fr {
    world.resources().frame.get().expect("frame présente")
}

/// `(phase, contrat)` de chaque système, lus sur la liste assemblée.
fn contracts() -> [(usize, Contract); SYSTEMS] {
    let filler = Contract::new(
        DataSet::EMPTY,
        DataSet::EMPTY,
        DataSet::EMPTY,
        StructuralEffects::NONE,
    );
    let mut out = [(0, filler); SYSTEMS];
    <Systems as SystemList<TestBinding>>::fill(&mut out, 0);
    out
}

fn id(index: u8) -> DataId {
    DataId::new(index).expect("catégorie valide")
}

/// Clé de tri locale au test : `Contribution` n'implémente ni `Ord` ni `Hash`.
fn cle(c: &Contribution) -> (i32, i32, u32, u32, [u8; 4]) {
    (c.x, c.y, c.width, c.height, c.color)
}

/// Multiensemble : tri par clé locale, multiplicité conservée.
fn multiensemble(contributions: &[Contribution]) -> Vec<Contribution> {
    let mut copie = contributions.to_vec();
    copie.sort_by_key(cle);
    copie
}

fn attendu() -> Vec<Contribution> {
    multiensemble(&[Q1, Q1, Q2])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// RN-1 : après un tick, la `Frame` lue dans le `World` contient exactement les
/// contributions de A et de B, doublon `Q1` compris, aucune absente, aucune
/// ajoutée (multiplicité conservée).
#[test]
fn rn1_la_frame_contient_exactement_les_contributions_des_producteurs() {
    let scheduler = scheduler();
    let mut world = assembled_world();
    let mut buffer = Buffer::new();

    tick(&scheduler, &mut world, &mut buffer);

    let frame = frame_of(&world);
    assert_eq!(frame.len(), 3);
    assert_eq!(multiensemble(frame.as_slice()), attendu());
}

/// RN-2 : contrôle de composition sur les contrats assemblés. Non imposé par les
/// types ni par le modèle entre phases (plan C3, C4).
#[test]
fn rn2_controle_de_composition_sur_les_contrats_assembles() {
    let contracts = contracts();
    let (acc, frame, witness) = (id(ACC), id(FRAME), id(WITNESS));
    let indices = || 0..contracts.len();

    // (a) exactement un système, `Compose`, a FRAME dans `writes` ∪ `produces`.
    let composeurs: Vec<usize> = indices()
        .filter(|&i| {
            let c = contracts[i].1;
            c.writes().contains(frame) || c.produces().contains(frame)
        })
        .collect();
    assert_eq!(composeurs, [COMPOSE]);

    // (b) ACC et FRAME n'apparaissent dans aucun `writes`.
    for (_, contract) in &contracts {
        assert!(!contract.writes().contains(acc));
        assert!(!contract.writes().contains(frame));
    }

    // (c) responsabilités de l'accumulateur : `Compose` en est l'unique lecteur ;
    // seuls `Clear`, `ProduceA` et `ProduceB` le produisent.
    let lecteurs: Vec<usize> = indices()
        .filter(|&i| contracts[i].1.reads().contains(acc))
        .collect();
    assert_eq!(lecteurs, [COMPOSE]);
    let producteurs: Vec<usize> = indices()
        .filter(|&i| contracts[i].1.produces().contains(acc))
        .collect();
    assert_eq!(producteurs, [CLEAR, PRODUCE_A, PRODUCE_B]);
    // `Clear` est en phase 0, strictement avant la phase de `Compose`.
    assert_eq!(contracts[CLEAR].0, 0);
    assert!(contracts[CLEAR].0 < contracts[COMPOSE].0);

    // (d) `Compose` est dans la dernière phase, et aucun système n'est dans une
    // phase postérieure.
    assert_eq!(contracts[COMPOSE].0, PHASES - 1);
    assert!(contracts.iter().all(|(phase, _)| *phase < PHASES));

    // (e) aucune catégorie ne désigne le Backend : toutes les catégories des
    // contrats sont dans {ACC, FRAME, WITNESS}.
    for (_, contract) in &contracts {
        let used = contract
            .reads()
            .union(contract.writes())
            .union(contract.produces());
        for index in 0..DataSet::CAPACITY {
            if used.contains(id(index)) {
                assert!([acc, frame, witness].contains(&id(index)));
            }
        }
    }
}

/// RN-3 : dans l'ordre déduit de la phase de `Compose`, `ProduceA` et `ProduceB`
/// précèdent chacun `Compose`, et `Compose` précède `Witness`. L'ordre relatif de
/// `ProduceA` et `ProduceB` n'est pas affirmé.
#[test]
fn rn3_l_ordre_deduit_place_les_producteurs_avant_compose_puis_witness() {
    let scheduler = scheduler();
    let phase = scheduler.model().phase(1).expect("phase 1 connue");
    let position = |system: usize| {
        phase
            .iter()
            .position(|&index| index == system)
            .expect("système présent dans la phase 1")
    };

    assert!(position(PRODUCE_A) < position(COMPOSE));
    assert!(position(PRODUCE_B) < position(COMPOSE));
    assert!(position(COMPOSE) < position(WITNESS_SYSTEM));
}

/// RN-4 : après un tick, la copie du témoin égale la `Frame` lue dans le `World`.
#[test]
fn rn4_la_copie_du_temoin_egale_la_frame() {
    let scheduler = scheduler();
    let mut world = assembled_world();
    let mut buffer = Buffer::new();

    tick(&scheduler, &mut world, &mut buffer);

    let temoin = world.resources().witness.get().expect("témoin présent");
    assert_eq!(temoin, frame_of(&world));
    assert_eq!(temoin.len(), 3);
}

/// RN-5 : avant le premier tick, accumulateur et `Frame` sont vides ; sur trois
/// ticks, la `Frame` de chaque tick compte trois contributions (aucun cumul).
#[test]
fn rn5_aucun_cumul_entre_les_ticks() {
    let scheduler = scheduler();
    let mut world = assembled_world();
    let mut buffer = Buffer::new();

    assert_eq!(world.resources().acc.get(), Some(&Acc::new()));
    assert_eq!(world.resources().frame.get(), Some(&Fr::empty()));

    for _ in 0..3 {
        tick(&scheduler, &mut world, &mut buffer);
        let frame = frame_of(&world);
        assert_eq!(frame.len(), 3);
        assert_eq!(multiensemble(frame.as_slice()), attendu());
    }
}

/// Exécute `ticks` ticks et rend la suite des `Frame` observées après chacun.
fn suite_de_frames(ticks: usize) -> Vec<Fr> {
    let scheduler = scheduler();
    let mut world = assembled_world();
    let mut buffer = Buffer::new();
    let mut suite = Vec::new();
    for _ in 0..ticks {
        tick(&scheduler, &mut world, &mut buffer);
        suite.push(frame_of(&world).clone());
    }
    suite
}

/// RN-6 : deux exécutions indépendantes de trois ticks donnent la même suite de
/// `Frame`, séquence comprise (reproductibilité du moteur). Aucun ordre
/// particulier n'est affirmé et aucune sémantique n'est prêtée à l'ordre de
/// déclaration.
#[test]
fn rn6_deux_executions_independantes_donnent_la_meme_suite_de_frames() {
    let premiere = suite_de_frames(3);
    let seconde = suite_de_frames(3);
    assert_eq!(premiere.len(), 3);
    assert_eq!(premiere, seconde);
}
