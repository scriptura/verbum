//! Noyau ECS de Verbum.
//!
//! Crate `no_std` sans `alloc` (ADR-000, §4) : aucune allocation, y compris à la
//! construction. Les capacités sont fixées avant la simulation (ADR-001, point 5).

#![no_std]

// La table de génération n'est consommée par aucun code non-test avant la Phase 3
// (le `World`). Attente unique, portée au module : elle est satisfaite dès qu'un
// élément du module est inutilisé, et le build échoue si ce n'est plus le cas.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consommé par le World à partir de la Phase 3")
)]
mod entity;

// Même attente que ci-dessus : le stockage n'est consommé par aucun code non-test
// avant la Phase 4b (premier `SparseSet` intégré au `World`).
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consommé par le World à partir de la Phase 4b")
)]
mod storage;

pub use entity::EntityId;
