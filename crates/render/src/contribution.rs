// crates/render/src/contribution.rs

//! Contribution de rendu : donnée plate émise par un système producteur.
//!
//! Une contribution est le contenu interne d'une Ressource transitoire
//! (ADR-R01, point 1 ; ADR-R02, bilan) : elle n'est ni persistante, ni
//! adressable, ni requêtable. Elle ne porte aucune clé, aucune référence,
//! aucun flottant, aucun `EntityId` et aucun type de la Plateforme
//! (ADR-R03, point 1 ; ADR-000, point 3).
//!
//! Aucune relation d'ordre n'est définie sur ce type (`Ord`, `PartialOrd` et
//! `Hash` sont volontairement absents) : le corpus ne fournit aucune règle
//! d'ordre de la frame (plan C8).

/// Rectangle plein décrivant une intention visuelle minimale (plan C12).
///
/// Les champs sont des entiers : position et taille en unités de la cible,
/// couleur RGBA sur 8 bits par canal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contribution {
    /// Abscisse du coin supérieur gauche.
    pub x: i32,
    /// Ordonnée du coin supérieur gauche.
    pub y: i32,
    /// Largeur.
    pub width: u32,
    /// Hauteur.
    pub height: u32,
    /// Couleur RGBA, un octet par canal.
    pub color: [u8; 4],
}

impl Contribution {
    /// Valeur de remplissage des emplacements inutilisés des stockages à
    /// capacité fixe. Privée au crate : jamais observable par l'API publique,
    /// qui n'expose que la tranche des éléments acceptés.
    pub(crate) const FILL: Self = Self {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        color: [0; 4],
    };
}
