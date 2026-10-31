// crates/assets/src/format.rs

//! Formats d'artefact connus et valeur décodée (A-3 i, E-10).
//!
//! Le format d'un artefact est celui que désigne son identité lorsque son préfixe de
//! 10 octets est complet et son identité connue ; sinon, le format attendu pour son slot.
//! Un seul décodeur sert aux niveaux 3 et 4 du chargement.

use crate::header::identity_of;
use crate::map::{self, MAP_FORMAT_ID, Map, MapParts};
use crate::sprite::{FORMAT_ID, ReadError, Sprite};
use crate::tileset::{TILESET_FORMAT_ID, Tileset};

/// Format d'artefact attendu pour un slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// Artefact `VSPR`.
    Sprite,
    /// Artefact `VTLS`.
    Tileset,
    /// Artefact `VMAP`.
    Map,
}

/// Valeur décodée d'un artefact, une variante par format. `'a` est la durée de vie des
/// octets empruntés par les vues (`Map`) ; les valeurs `Copy` (`Sprite`, `Tileset`) n'en
/// dépendent pas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asset<'a> {
    /// Descripteur de sprite.
    Sprite(Sprite),
    /// Descripteur de tileset.
    Tileset(Tileset),
    /// Carte : vue empruntée dont le tileset est résolu en slot.
    Map(Map<'a>),
}

impl Format {
    /// Format désigné par l'identité de `bytes` ; `None` si le préfixe de 10 octets est
    /// incomplet ou si l'identité n'est celle d'aucun format connu.
    pub(crate) fn of_identity(bytes: &[u8]) -> Option<Self> {
        match identity_of(bytes)? {
            FORMAT_ID => Some(Self::Sprite),
            TILESET_FORMAT_ID => Some(Self::Tileset),
            MAP_FORMAT_ID => Some(Self::Map),
            _ => None,
        }
    }
}

/// Résultat du décodeur : une valeur prête, ou les champs d'une carte dont la référence
/// reste à résoudre (la résolution exige le manifeste, que le décodeur ne connaît pas).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Decoded<'a> {
    Ready(Asset<'a>),
    Map(MapParts<'a>),
}

/// Décodeur unique des niveaux 3 et 4 : le format désigné par l'identité prévaut, à
/// défaut celui que la table attend pour le slot (`expected`).
pub(crate) fn decode<'a>(expected: Format, bytes: &'a [u8]) -> Result<Decoded<'a>, ReadError> {
    match Format::of_identity(bytes).unwrap_or(expected) {
        Format::Sprite => Sprite::read(bytes).map(|v| Decoded::Ready(Asset::Sprite(v))),
        Format::Tileset => Tileset::read(bytes).map(|v| Decoded::Ready(Asset::Tileset(v))),
        Format::Map => map::read(bytes).map(Decoded::Map),
    }
}
