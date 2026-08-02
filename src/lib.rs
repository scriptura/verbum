//! Verbum: A strict ECS and Data-Oriented 2D game engine.
//!
//! Cette version initiale sert de manifeste architectural. 
//! L'implémentation complète des pipelines ECS, DOD et AOT est en cours de développement.
//! Veuillez consulter le dossier `docs/architecture` pour le corpus décisionnel complet (ADRs).

/// L'état pur et déterministe de la simulation.
/// Ne possède aucune connaissance de l'I/O, du réseau ou de l'écran.
pub struct World {
    pub current_tick: u64,
    // Les futurs stockages plats (Composants/Ressources) viendront s'aligner ici.
}

impl World {
    pub fn new() -> Self {
        Self { current_tick: 0 }
    }
}

/// L'interface d'acquisition matérielle et de projection.
/// Isole intégralement le World du système d'exploitation.
pub struct Shell;

impl Shell {
    pub fn new() -> Self {
        Self
    }

    /// Phase 1 : Capte les événements matériels et produit des Intentions.
    pub fn acquire_intentions(&self) {
        // Mock de l'acquisition matérielle
    }

    /// Phase 3 : Projette l'état immuable du World vers les périphériques (écran, audio).
    pub fn project(&self, _world: &World) {
        // Mock du renderer et du backend audio
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

/// Contrat strict pour tout système du moteur :
/// - Exécution déterministe.
/// - Mutation exclusive des données du World.
/// - Aucune indirection, aucun I/O.
pub trait System {
    fn execute(&self, world: &mut World);
}
