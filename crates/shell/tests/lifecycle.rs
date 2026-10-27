// crates/shell/tests/lifecycle.rs

//! Cycle de vie de la boucle (LC-1 à LC-4).
//!
//! Infrastructure factice : elle journalise ses exécutions et sa libération
//! (`Drop`). L'arrêt est demandé par une fermeture scriptée. Aucun fil réel.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use verbum_shell::{Inbox, run_loop};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trace {
    /// Début de l'exécution numéro `n` (à partir de 0).
    Debut(u32),
    /// Témoin de fin de l'exécution numéro `n`.
    Fin(u32),
    /// Libération de l'infrastructure.
    Libere,
}

type Journal = Rc<RefCell<Vec<Trace>>>;

/// Valeur capturée par l'infrastructure : sa libération est journalisée.
struct Garde {
    journal: Journal,
}

impl Drop for Garde {
    fn drop(&mut self) {
        self.journal.borrow_mut().push(Trace::Libere);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ErreurTest(u32);

/// LC-1 : N itérations puis arrêt → exactement N exécutions, dans l'ordre ;
/// une seule libération, après la dernière.
#[test]
fn lc1_n_iterations_puis_arret() {
    const N: u32 = 3;
    let journal: Journal = Rc::default();
    let executions = Rc::new(Cell::new(0u32));

    let garde = Garde {
        journal: journal.clone(),
    };
    let compteur = executions.clone();
    let infrastructure = move |_: &[u32]| -> Result<(), ErreurTest> {
        let n = compteur.get();
        garde.journal.borrow_mut().push(Trace::Debut(n));
        garde.journal.borrow_mut().push(Trace::Fin(n));
        compteur.set(n + 1);
        Ok(())
    };
    let compteur = executions.clone();
    let resultat = run_loop(
        || {},
        move || compteur.get() >= N,
        &mut Inbox::new(0),
        infrastructure,
    );

    assert_eq!(resultat, Ok(u64::from(N)));
    assert_eq!(
        *journal.borrow(),
        [
            Trace::Debut(0),
            Trace::Fin(0),
            Trace::Debut(1),
            Trace::Fin(1),
            Trace::Debut(2),
            Trace::Fin(2),
            Trace::Libere,
        ]
    );
}

/// LC-2 : arrêt demandé avant la première itération → 0 exécution ;
/// libération effectuée.
#[test]
fn lc2_arret_avant_la_premiere_iteration() {
    let journal: Journal = Rc::default();

    let garde = Garde {
        journal: journal.clone(),
    };
    let infrastructure = move |_: &[u32]| -> Result<(), ErreurTest> {
        garde.journal.borrow_mut().push(Trace::Debut(0));
        Ok(())
    };
    let resultat = run_loop(|| {}, || true, &mut Inbox::new(0), infrastructure);

    assert_eq!(resultat, Ok(0));
    assert_eq!(*journal.borrow(), [Trace::Libere]);
}

/// LC-3 : arrêt demandé pendant une exécution → cette exécution s'achève
/// (témoin de fin écrit) ; aucune suivante.
#[test]
fn lc3_arret_pendant_une_execution() {
    let journal: Journal = Rc::default();
    let arret = Rc::new(Cell::new(false));

    let garde = Garde {
        journal: journal.clone(),
    };
    let demande = arret.clone();
    let infrastructure = move |_: &[u32]| -> Result<(), ErreurTest> {
        garde.journal.borrow_mut().push(Trace::Debut(0));
        demande.set(true); // arrêt demandé au milieu de l'exécution
        garde.journal.borrow_mut().push(Trace::Fin(0));
        Ok(())
    };
    let resultat = run_loop(
        || {},
        move || arret.get(),
        &mut Inbox::new(0),
        infrastructure,
    );

    assert_eq!(resultat, Ok(1));
    assert_eq!(
        *journal.borrow(),
        [Trace::Debut(0), Trace::Fin(0), Trace::Libere]
    );
}

/// LC-4 : une exécution échoue → la boucle s'arrête ; l'erreur d'origine est
/// rendue sans transformation ; aucune réexécution ; libération effectuée.
#[test]
fn lc4_erreur_rendue_sans_transformation() {
    let journal: Journal = Rc::default();
    let executions = Rc::new(Cell::new(0u32));

    let garde = Garde {
        journal: journal.clone(),
    };
    let compteur = executions.clone();
    let infrastructure = move |_: &[u32]| -> Result<(), ErreurTest> {
        let n = compteur.get();
        garde.journal.borrow_mut().push(Trace::Debut(n));
        compteur.set(n + 1);
        if n == 2 {
            return Err(ErreurTest(42));
        }
        garde.journal.borrow_mut().push(Trace::Fin(n));
        Ok(())
    };
    // L'arrêt n'est jamais demandé : seule l'erreur peut terminer la boucle.
    let resultat = run_loop(|| {}, || false, &mut Inbox::new(0), infrastructure);

    assert_eq!(resultat, Err(ErreurTest(42)));
    assert_eq!(
        *journal.borrow(),
        [
            Trace::Debut(0),
            Trace::Fin(0),
            Trace::Debut(1),
            Trace::Fin(1),
            Trace::Debut(2),
            Trace::Libere,
        ]
    );
}
