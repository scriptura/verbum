// crates/engine/shell/tests/stop_before_cut.rs

//! Conformité ADR-S02 : un arrêt observé avant une itération empêche toute coupure.
//!
//! Test de conformité ajouté par l'unité T4 ; aucune assertion historique n'est modifiée.
//!
//! Scénario : un lot non vide est déposé avant l'appel à `run_loop` ; l'arrêt est demandé
//! dès sa première observation. Observations :
//! - `run_loop` rend `Ok(0)` (aucune itération comptabilisée) ;
//! - l'infrastructure n'est jamais invoquée ;
//! - la boîte n'a subi aucune coupure : une coupure effectuée après le retour rend
//!   l'intégralité du lot, dans l'ordre d'acceptation (`Inbox::cut` vide la boîte, donc
//!   toute coupure pendant l'appel aurait fait perdre ce contenu).

use std::cell::Cell;
use verbum_shell::{Inbox, run_loop};

#[test]
fn arret_avant_la_premiere_iteration_ne_coupe_pas_la_boite() {
    let mut inbox = Inbox::<u32>::new(8);
    let depot = inbox.handle();
    for e in [5, 3, 9, 1] {
        assert_eq!(depot.push(e), Ok(()));
    }

    let invocations = Cell::new(0u32);
    let resultat = run_loop(
        || {},
        || true,
        &mut inbox,
        |_lot: &[u32]| {
            invocations.set(invocations.get() + 1);
            Ok::<(), ()>(())
        },
    );

    assert_eq!(resultat, Ok(0), "aucune itération comptabilisée");
    assert_eq!(invocations.get(), 0, "infrastructure non invoquée");
    assert_eq!(
        inbox.cut(),
        [5, 3, 9, 1],
        "contenu intégralement conservé, dans l'ordre"
    );
}
