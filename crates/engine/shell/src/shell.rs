// crates/engine/shell/src/shell.rs

//! Boucle du Shell : cycle de vie, itérations bornées et coupure des événements.
//!
//! La boucle ne connaît aucun type du noyau : elle pilote trois fermetures
//! (cadencement, demande d'arrêt, infrastructure) et une [`Inbox`], et rend le
//! résultat de l'infrastructure sans le transformer.
//!
//! La séquence d'une itération (observation de l'arrêt, coupure, invocation
//! unique de l'infrastructure) n'est écrite qu'à un seul endroit : [`iterate`].

use crate::Inbox;

/// Issue d'une itération du cœur ([`iterate`]).
pub(crate) enum Iteration<E> {
    /// L'arrêt a été observé : ni coupure ni invocation n'ont eu lieu.
    Stopped,
    /// L'infrastructure a rendu une erreur, transportée **inchangée**.
    Failed(E),
    /// L'itération s'est achevée : une coupure, une invocation, un succès.
    Completed,
}

/// Cœur d'itération : observe l'arrêt, puis coupe l'`Inbox` et invoque
/// l'infrastructure **une seule fois** avec le lot.
///
/// Ordre garanti : l'arrêt est observé **avant** toute coupure. Si `stop`
/// rend `true`, la boîte n'est pas touchée et l'infrastructure n'est pas
/// invoquée. Le verrou de la boîte est libéré avant l'invocation (voir
/// [`Inbox::cut`]). Le comptage et le cadencement appartiennent au pilote.
pub(crate) fn iterate<Ev, E>(
    stop: &mut impl FnMut() -> bool,
    inbox: &mut Inbox<Ev>,
    infrastructure: &mut impl FnMut(&[Ev]) -> Result<(), E>,
) -> Iteration<E>
where
    Ev: Copy,
{
    if stop() {
        return Iteration::Stopped;
    }
    // Coupure : le verrou de la boîte est libéré au retour de `cut`.
    match infrastructure(inbox.cut()) {
        Ok(()) => Iteration::Completed,
        Err(error) => Iteration::Failed(error),
    }
}

/// Exécute la boucle du Shell jusqu'à la demande d'arrêt ou à la première erreur.
///
/// Boucle bloquante à cadence externe, hors politique temporelle S04.
///
/// Une itération se déroule ainsi :
/// 1. `cadence` attend le prochain tick (politique **provisoire** à pas fixe :
///    [`FixedStepPacing`](crate::FixedStepPacing), remplaçable par toute fermeture) ;
/// 2. `stop` est observée ; si elle rend `true`, la boucle s'arrête ;
/// 3. coupure : `inbox` est prélevée en un seul passage (voir [`Inbox::cut`]) ;
///    les événements acceptés avant la coupure forment le lot, ceux acceptés
///    après restent en attente de l'itération suivante ;
/// 4. `infrastructure` est appelée **une seule fois**, avec ce lot (vide si
///    rien n'a été déposé : elle n'est jamais sautée).
///
/// Le verrou de la boîte est libéré avant l'appel de `infrastructure` : un
/// dépôt effectué pendant un tick est accepté et appartient à une itération
/// ultérieure.
///
/// Une exécution de `infrastructure` engagée n'est jamais interrompue : la
/// demande d'arrêt n'est observée qu'entre deux itérations.
///
/// Les fermetures sont reçues par valeur : elles sont libérées (et leurs
/// valeurs capturées avec elles) au retour de la fonction, après la dernière
/// exécution, quelle que soit l'issue. La boîte reste à l'appelant.
///
/// # Retour
///
/// - `Ok(n)` : arrêt demandé, `n` itérations achevées ;
/// - `Err(e)` : première erreur rendue par `infrastructure`, **inchangée** ;
///   aucune réexécution n'a lieu.
pub fn run_loop<Ev, E>(
    mut cadence: impl FnMut(),
    mut stop: impl FnMut() -> bool,
    inbox: &mut Inbox<Ev>,
    mut infrastructure: impl FnMut(&[Ev]) -> Result<(), E>,
) -> Result<u64, E>
where
    Ev: Copy,
{
    let mut achevees: u64 = 0;
    loop {
        cadence();
        match iterate(&mut stop, inbox, &mut infrastructure) {
            Iteration::Completed => achevees += 1,
            Iteration::Stopped => return Ok(achevees),
            Iteration::Failed(error) => return Err(error),
        }
    }
}
