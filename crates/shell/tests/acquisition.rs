// crates/shell/tests/acquisition.rs

//! Acquisition des événements : coupure, ordre, saturation (AQ-1 à AQ-7b).
//!
//! PC-2 (U3) est repris par AQ-5. Source scriptée (A3) : les dépôts sont
//! effectués par des fermetures de test, appelées depuis la fermeture de
//! cadencement (dépôt entre deux ticks) ou depuis l'infrastructure factice
//! (dépôt pendant un tick). Aucun fil réel.
//!
//! **Portée.** Ces tests vérifient la logique de visibilité (quel lot voit
//! quel tick). Ils ne démontrent aucune sûreté concurrente : l'atomicité de la
//! coupure est une garantie du code, vérifiée par revue.

use std::cell::{Cell, RefCell};
use std::time::Duration;
use verbum_shell::{FixedStepPacing, Inbox, InboxFull, run_loop};

type Lots = Vec<Vec<u32>>;

/// Rejoue un script de dépôts sur `ticks` itérations et rend les lots observés
/// par l'infrastructure, dans l'ordre.
///
/// - `entre[n]` : événements déposés par la cadence du tick `n`, donc avant la
///   coupure `n` (entre la fin du tick `n − 1` et la coupure `n`) ;
/// - `pendant[n]` : événements déposés par l'infrastructure pendant le tick
///   `n`, donc après la coupure `n`.
///
/// L'arrêt est piloté par le nombre de cadences, non par les exécutions.
fn rejouer(
    capacity: usize,
    entre: &[&[u32]],
    pendant: &[&[u32]],
    ticks: usize,
    mut attente: impl FnMut(),
) -> Lots {
    let mut inbox = Inbox::new(capacity);
    let depot_cadence = inbox.handle();
    let depot_infra = inbox.handle();
    let cadences = Cell::new(0usize);
    let executions = Cell::new(0usize);
    let lots = RefCell::new(Lots::new());

    let resultat = run_loop(
        || {
            attente();
            for &e in entre.get(cadences.get()).copied().unwrap_or(&[]) {
                depot_cadence.push(e).unwrap();
            }
            cadences.set(cadences.get() + 1);
        },
        || cadences.get() > ticks,
        &mut inbox,
        |lot: &[u32]| {
            lots.borrow_mut().push(lot.to_vec());
            for &e in pendant.get(executions.get()).copied().unwrap_or(&[]) {
                depot_infra.push(e).unwrap();
            }
            executions.set(executions.get() + 1);
            Ok::<(), ()>(())
        },
    );

    assert_eq!(resultat, Ok(ticks as u64), "la boucle doit s'achever");
    lots.into_inner()
}

/// Fait tourner `ticks` itérations sur une boîte existante, sans dépôt
/// scripté, et rend les lots observés.
fn iterer(inbox: &mut Inbox<u32>, ticks: usize) -> Lots {
    let cadences = Cell::new(0usize);
    let lots = RefCell::new(Lots::new());
    let resultat = run_loop(
        || cadences.set(cadences.get() + 1),
        || cadences.get() > ticks,
        inbox,
        |lot: &[u32]| {
            lots.borrow_mut().push(lot.to_vec());
            Ok::<(), ()>(())
        },
    );
    assert_eq!(resultat, Ok(ticks as u64));
    lots.into_inner()
}

/// AQ-1 (coupure) : un événement déposé pendant le tick N n'est observé par
/// aucun système du tick N ; le dépôt est accepté et le tick se termine (aucun
/// interblocage) ; l'événement reste en attente ; le lot du tick N+1 le
/// contient, avec les événements acceptés avant la coupure N+1.
#[test]
fn aq1_depot_pendant_un_tick_va_au_tick_suivant() {
    let lots = rejouer(8, &[&[1, 2], &[4]], &[&[3]], 2, || {});
    assert_eq!(lots, [vec![1, 2], vec![3, 4]]);
}

/// AQ-1b : un événement déposé entre la fin du tick N−1 et la coupure du tick N
/// est dans le lot du tick N.
#[test]
fn aq1b_depot_avant_la_coupure_est_dans_le_lot() {
    let lots = rejouer(8, &[&[1], &[9]], &[], 2, || {});
    assert_eq!(lots, [vec![1], vec![9]]);
}

/// AQ-2 : l'ordre d'acceptation par la boîte est conservé dans le lot (valeurs
/// volontairement non triées).
#[test]
fn aq2_ordre_d_acceptation_conserve() {
    let lots = rejouer(8, &[&[5, 3, 9, 1]], &[], 1, || {});
    assert_eq!(lots, [vec![5, 3, 9, 1]]);
}

/// AQ-3 : sur K ticks, aucun événement perdu ni dupliqué. Les valeurs croissent
/// dans l'ordre d'acceptation ; la concaténation des lots est donc exactement
/// la suite 1..=N. Le script contient des ticks sans dépôt.
#[test]
fn aq3_aucun_evenement_perdu_ni_duplique() {
    let entre: &[&[u32]] = &[&[1, 2], &[], &[5], &[7, 8, 9], &[], &[12]];
    let pendant: &[&[u32]] = &[&[3, 4], &[], &[6], &[], &[10, 11], &[]];
    let lots = rejouer(16, entre, pendant, 6, || {});

    assert_eq!(lots.len(), 6);
    let concatenation: Vec<u32> = lots.into_iter().flatten().collect();
    assert_eq!(concatenation, (1..=12).collect::<Vec<u32>>());
}

/// AQ-5 (reprend PC-2 ; ADR-S03 pt 11) : la même séquence scriptée rejouée avec
/// un intervalle nul et un intervalle de l'ordre de la milliseconde donne la
/// même suite de lots, non vides. Aucune assertion sur la durée réelle.
#[test]
fn aq5_meme_sequence_meme_suite_de_lots() {
    let entre: &[&[u32]] = &[&[1, 2], &[3], &[4, 5, 6], &[7]];
    let pendant: &[&[u32]] = &[&[], &[], &[], &[]];

    let mut nul = FixedStepPacing::new(Duration::ZERO);
    let avec_intervalle_nul = rejouer(8, entre, pendant, 4, || nul.wait());
    let mut milli = FixedStepPacing::new(Duration::from_millis(1));
    let avec_intervalle_milli = rejouer(8, entre, pendant, 4, || milli.wait());

    assert_eq!(avec_intervalle_nul, avec_intervalle_milli);
    assert_eq!(avec_intervalle_nul.len(), 4);
    assert!(avec_intervalle_nul.iter().all(|lot| !lot.is_empty()));
}

/// AQ-6 (saturation) : capacité C ; C dépôts réussis ; le dépôt C+1 rend une
/// erreur explicite ; le lot suivant contient exactement les C premiers
/// événements, dans l'ordre ; l'événement refusé n'y figure pas ; la capacité
/// n'a pas changé.
#[test]
fn aq6_saturation_erreur_explicite() {
    const C: usize = 3;
    let mut inbox = Inbox::<u32>::new(C);
    let depot = inbox.handle();

    for e in [1, 2, 3] {
        assert_eq!(depot.push(e), Ok(()));
    }
    assert_eq!(depot.push(4), Err(InboxFull));
    assert_eq!(inbox.capacity(), C);

    let lots = iterer(&mut inbox, 1);
    assert_eq!(lots, [vec![1, 2, 3]]);
    assert_eq!(inbox.capacity(), C);
}

/// AQ-6b : après le prélèvement, la capacité est de nouveau disponible : un
/// nouveau dépôt réussit et se retrouve dans le lot suivant.
#[test]
fn aq6b_capacite_rendue_apres_prelevement() {
    let mut inbox = Inbox::<u32>::new(2);
    let depot = inbox.handle();
    assert_eq!(depot.push(1), Ok(()));
    assert_eq!(depot.push(2), Ok(()));
    assert_eq!(depot.push(3), Err(InboxFull));

    assert_eq!(iterer(&mut inbox, 1), [vec![1, 2]]);

    assert_eq!(depot.push(5), Ok(()));
    assert_eq!(iterer(&mut inbox, 1), [vec![5]]);
}

/// AQ-7 (publication complète) : boîte pleine ; le lot contient les C
/// événements, sans troncature ; la boîte est vide après le prélèvement.
///
/// La capacité du lot, égale à celle de la boîte par construction, est vérifiée
/// par revue.
#[test]
fn aq7_lot_complet_et_boite_videe() {
    const C: usize = 4;
    let mut inbox = Inbox::<u32>::new(C);
    let depot = inbox.handle();
    for e in 1..=4 {
        assert_eq!(depot.push(e), Ok(()));
    }

    let lots = iterer(&mut inbox, 2);
    assert_eq!(lots, [vec![1, 2, 3, 4], vec![]]);
}

/// AQ-7b : une itération sans aucun dépôt reçoit un lot vide ; l'infrastructure
/// est invoquée avec zéro événement, elle n'est ni sautée ni appelée sans lot.
#[test]
fn aq7b_iteration_sans_depot_recoit_un_lot_vide() {
    let mut inbox = Inbox::<u32>::new(4);
    let lots = iterer(&mut inbox, 3);
    assert_eq!(lots, [vec![], vec![], vec![]]);
}
