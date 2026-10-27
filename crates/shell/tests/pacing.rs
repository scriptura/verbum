// crates/shell/tests/pacing.rs

//! Cadencement provisoire (PC-1 et PC-3).
//!
//! PC-2 est déplacé en U4 (couvert par AQ-5) : le numéro n'est pas réattribué.

use std::cell::Cell;
use std::time::{Duration, Instant};
use verbum_shell::{FixedStepPacing, Inbox, run_loop};

/// Infrastructure factice : compte ses exécutions, ne peut pas échouer.
fn compter<'a>(executions: &'a Cell<u32>) -> impl FnMut(&[u32]) -> Result<(), ()> + 'a {
    move |_| {
        executions.set(executions.get() + 1);
        Ok(())
    }
}

/// PC-1 : intervalle nul → N itérations sans attente.
///
/// « Sans attente » est borné largement (1 s pour 1000 itérations) : la borne
/// détecte une attente par itération, pas un écart de performance.
///
/// Sous Miri, l'interpréteur ralentit l'exécution d'un facteur sans rapport avec l'horloge
/// réelle : la borne temporelle n'y est pas évaluée. Les assertions sur le résultat et sur
/// le nombre d'exécutions restent évaluées.
#[test]
fn pc1_intervalle_nul_sans_attente() {
    const N: u32 = 1000;
    let executions = Cell::new(0u32);
    let mut pacing = FixedStepPacing::new(Duration::ZERO);

    let debut = Instant::now();
    let resultat = run_loop(
        || pacing.wait(),
        || executions.get() >= N,
        &mut Inbox::new(0),
        compter(&executions),
    );
    let ecoule = debut.elapsed();

    assert_eq!(resultat, Ok(u64::from(N)));
    assert_eq!(executions.get(), N);
    if !cfg!(miri) {
        assert!(
            ecoule < Duration::from_secs(1),
            "attente détectée : {ecoule:?}"
        );
    }
}

/// Politique scriptée : déclenchement à la demande, sans horloge. Chaque
/// attente consomme un « top » fourni par le script.
struct Scripte<'a> {
    tops: &'a Cell<u32>,
}

impl Scripte<'_> {
    fn wait(&mut self) {
        let restant = self.tops.get();
        assert!(restant > 0, "attente sans top fourni par le script");
        self.tops.set(restant - 1);
    }
}

/// PC-3 : la boucle accepte une seconde politique de cadencement (scriptée,
/// déclenchement à la demande) sans modification du reste. Le même appel à
/// `run_loop`, la même infrastructure, produisent le même résultat avec la
/// politique à pas fixe et avec la politique scriptée.
#[test]
fn pc3_seconde_politique_sans_modifier_le_reste() {
    const N: u32 = 5;

    // Politique à pas fixe provisoire.
    let executions = Cell::new(0u32);
    let mut pas_fixe = FixedStepPacing::new(Duration::ZERO);
    let avec_pas_fixe = run_loop(
        || pas_fixe.wait(),
        || executions.get() >= N,
        &mut Inbox::new(0),
        compter(&executions),
    );
    assert_eq!(avec_pas_fixe, Ok(u64::from(N)));
    assert_eq!(executions.get(), N);

    // Politique scriptée : N + 1 tops (une attente précède l'observation de l'arrêt).
    let executions = Cell::new(0u32);
    let tops = Cell::new(N + 1);
    let mut scripte = Scripte { tops: &tops };
    let avec_script = run_loop(
        || scripte.wait(),
        || executions.get() >= N,
        &mut Inbox::new(0),
        compter(&executions),
    );
    assert_eq!(avec_script, avec_pas_fixe);
    assert_eq!(executions.get(), N);
    assert_eq!(tops.get(), 0, "chaque attente a consommé exactement un top");
}
