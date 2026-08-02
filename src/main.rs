use verbum::{Shell, World, System};

/// Un système fictif pour prouver le concept de mutation déterministe.
struct TimeSystem;

impl System for TimeSystem {
    fn execute(&self, world: &mut World) {
        world.current_tick += 1;
    }
}

fn main() {
    println!("Verbum Engine - Initialisation du socle architectural.");
    println!("In principio erat Verbum.\n");

    let mut world = World::new();
    let shell = Shell::new();
    let time_system = TimeSystem;

    // Démonstration de l'invariant majeur (ADR-S01) : 
    // L'acquisition pilote la logique, il n'y a pas de boucle infinie interne au World.
    println!("-- Démarrage de la boucle externe --");
    for _ in 0..3 {
        // 1. Acquisition
        shell.acquire_intentions();
        
        // 2. Traitement Logique (pur)
        time_system.execute(&mut world);
        println!("Tick logique : {}", world.current_tick);
        
        // 3. Projection
        shell.project(&world);
    }
    
    println!("-- Arrêt --");
    println!("Invariants respectés. Le moteur est prêt pour l'intégration de la Forge.");
}
