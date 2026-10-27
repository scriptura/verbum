// crates/shell/src/pacing.rs

//! Cadencement à pas fixe — politique **provisoire**.
//!
//! Ce module isole la seule politique temporelle du Shell. Elle est
//! provisoire : elle ne préjuge pas de la politique réelle (ADR-S04, non
//! rédigée) et reste remplaçable, car la boucle ne consomme qu'une fermeture
//! `FnMut()`.
//!
//! Ni `Instant` ni `Duration` ne sont jamais remis à l'infrastructure, et le
//! `World` ne reçoit jamais de durée ni d'instant.

use std::time::{Duration, Instant};

/// Cadencement à pas fixe, **provisoire**.
///
/// Chaque appel à [`wait`](Self::wait) attend la prochaine échéance, espacée
/// de l'intervalle fixé à la construction. Pas de rattrapage : si une
/// itération dépasse l'échéance, la suivante repart de l'instant courant, sans
/// rafale ni interpolation.
///
/// Usage : `run_loop(|| pacing.wait(), …)`.
#[derive(Debug)]
pub struct FixedStepPacing {
    interval: Duration,
    next: Option<Instant>,
}

impl FixedStepPacing {
    /// Construit un cadencement à pas fixe **provisoire**.
    ///
    /// Un intervalle nul supprime toute attente. `Duration` n'intervient ici
    /// que côté application ; l'infrastructure n'en reçoit jamais.
    #[must_use]
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            next: None,
        }
    }

    /// Attend la prochaine échéance du pas fixe **provisoire**.
    ///
    /// # Panique
    ///
    /// Panique si l'échéance dépasse la plage de `Instant` (intervalle de
    /// l'ordre de plusieurs siècles).
    pub fn wait(&mut self) {
        let now = Instant::now();
        let target = self.next.unwrap_or(now) + self.interval;
        if target > now {
            std::thread::sleep(target - now);
            self.next = Some(target);
        } else {
            // Échéance dépassée (ou intervalle nul) : ni attente ni rattrapage.
            self.next = Some(now);
        }
    }
}
