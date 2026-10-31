// crates/forge/src/map.rs

//! Analyse d'un `map.toml` et encodage de l'artefact `map.bin` (I1, I2, I4, I9, I11).
//!
//! Source : `tileset` (chaîne : clé logique d'un tileset du même lot, jamais écrite dans
//! la sortie), `width` et `height` dans `1..=65535`, `cells` (tableau de
//! `width × height` entiers dans `0..=65535`). La validation se fait en deux temps :
//! l'analyse d'une source seule (ici), puis la résolution de la référence et la
//! validation croisée (`batch.rs`), qui exigent le lot entier.

use verbum_assets::{MAP_FIXED_PAYLOAD_LEN, MAP_FORMAT_ID, MAP_FORMAT_VERSION};

use crate::ForgeError;
use crate::sprite::dimension;

/// Carte analysée, référence non encore résolue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MapSource {
    /// Clé logique du tileset référencé.
    pub(crate) tileset_key: String,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) cells: Vec<u16>,
}

/// Analyse une source `map.toml`.
///
/// Ordre des contrôles : syntaxe, champ inconnu, `width` et `height` (présence, type,
/// zéro, intervalle), taille de la charge utile (`12 + 2·width·height` doit tenir dans un
/// `u32`), `tileset` (présence, chaîne), `cells` (présence, tableau, éléments entiers dans
/// `0..=65535`), enfin nombre de cases égal à `width × height`.
pub(crate) fn parse_map(source: &str) -> Result<MapSource, ForgeError> {
    let table: toml::Table = source
        .parse()
        .map_err(|error: toml::de::Error| ForgeError::Syntax(error.to_string()))?;

    if let Some(unknown) = table
        .keys()
        .find(|key| !matches!(key.as_str(), "tileset" | "width" | "height" | "cells"))
    {
        return Err(ForgeError::UnknownField(unknown.clone()));
    }

    let width = dimension(&table, "width")?;
    let height = dimension(&table, "height")?;
    let payload = MAP_FIXED_PAYLOAD_LEN as u64 + 2 * u64::from(width) * u64::from(height);
    if u32::try_from(payload).is_err() {
        return Err(ForgeError::PayloadTooLarge);
    }

    let tileset = table
        .get("tileset")
        .ok_or(ForgeError::MissingField("tileset"))?;
    let toml::Value::String(tileset_key) = tileset else {
        return Err(ForgeError::WrongType("tileset"));
    };

    let cells = table
        .get("cells")
        .ok_or(ForgeError::MissingField("cells"))?;
    let toml::Value::Array(items) = cells else {
        return Err(ForgeError::WrongType("cells"));
    };
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        let toml::Value::Integer(integer) = item else {
            return Err(ForgeError::WrongType("cells"));
        };
        values.push(u16::try_from(*integer).map_err(|_| ForgeError::OutOfRange("cells"))?);
    }
    if u64::try_from(values.len()) != Ok(u64::from(width) * u64::from(height)) {
        return Err(ForgeError::CellCount);
    }

    Ok(MapSource {
        tileset_key: tileset_key.clone(),
        width,
        height,
        cells: values,
    })
}

/// Encode l'artefact : `tileset_id` est l'identifiant du tileset résolu par le lot, écrit
/// en little-endian ; aucune chaîne n'est écrite.
pub(crate) fn encode_map(tileset_id: u64, map: &MapSource) -> Vec<u8> {
    let payload = MAP_FIXED_PAYLOAD_LEN + 2 * map.cells.len();
    let mut bytes = Vec::with_capacity(10 + payload);
    bytes.extend_from_slice(&MAP_FORMAT_ID);
    bytes.extend_from_slice(&MAP_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&u32::try_from(payload).unwrap_or(u32::MAX).to_le_bytes());
    bytes.extend_from_slice(&tileset_id.to_le_bytes());
    bytes.extend_from_slice(&map.width.to_le_bytes());
    bytes.extend_from_slice(&map.height.to_le_bytes());
    for cell in &map.cells {
        bytes.extend_from_slice(&cell.to_le_bytes());
    }
    debug_assert_eq!(bytes.len(), 10 + payload);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Source de la carte 3 × 2 de référence.
    const SOURCE: &str =
        "tileset = \"ground\"\nwidth = 3\nheight = 2\ncells = [1, 515, 1029, 1543, 2057, 2571]\n";

    #[test]
    fn map_f3_a_valid_source_is_analyzed_into_its_fields() {
        let map = parse_map(SOURCE).expect("source valide");
        assert_eq!(map.tileset_key, "ground");
        assert_eq!((map.width, map.height), (3, 2));
        assert_eq!(map.cells, [1, 515, 1029, 1543, 2057, 2571]);
    }

    #[test]
    fn map_f3_every_invalid_source_is_refused() {
        let with = |width: &str, height: &str, tileset: &str, cells: &str| {
            parse_map(&format!(
                "width = {width}\nheight = {height}\ntileset = {tileset}\ncells = {cells}\n"
            ))
        };
        let ok = |cells: &str| with("2", "1", "\"t\"", cells);

        // Champ manquant, dans l'ordre de contrôle.
        assert_eq!(parse_map(""), Err(ForgeError::MissingField("width")));
        assert_eq!(
            parse_map("width = 1\n"),
            Err(ForgeError::MissingField("height"))
        );
        assert_eq!(
            parse_map("width = 1\nheight = 1\n"),
            Err(ForgeError::MissingField("tileset"))
        );
        assert_eq!(
            parse_map("width = 1\nheight = 1\ntileset = \"t\"\n"),
            Err(ForgeError::MissingField("cells"))
        );

        // Zéro, hors `u16`, mauvais type sur width et height.
        assert_eq!(
            with("0", "1", "\"t\"", "[0]"),
            Err(ForgeError::ZeroValue("width"))
        );
        assert_eq!(
            with("1", "0", "\"t\"", "[0]"),
            Err(ForgeError::ZeroValue("height"))
        );
        assert_eq!(
            with("65536", "1", "\"t\"", "[0]"),
            Err(ForgeError::OutOfRange("width"))
        );
        assert_eq!(
            with("-1", "1", "\"t\"", "[0]"),
            Err(ForgeError::OutOfRange("width"))
        );
        assert_eq!(
            with("\"1\"", "1", "\"t\"", "[0]"),
            Err(ForgeError::WrongType("width"))
        );

        // `tileset` doit être une chaîne.
        assert_eq!(
            with("1", "1", "5", "[0]"),
            Err(ForgeError::WrongType("tileset"))
        );

        // `cells` : mauvais type, élément non entier, hors `u16`, mauvaise longueur.
        assert_eq!(
            with("1", "1", "\"t\"", "5"),
            Err(ForgeError::WrongType("cells"))
        );
        assert_eq!(ok("[1, \"2\"]"), Err(ForgeError::WrongType("cells")));
        assert_eq!(ok("[1, 1.5]"), Err(ForgeError::WrongType("cells")));
        assert_eq!(ok("[1, -1]"), Err(ForgeError::OutOfRange("cells")));
        assert_eq!(ok("[1, 65536]"), Err(ForgeError::OutOfRange("cells")));
        assert_eq!(ok("[1]"), Err(ForgeError::CellCount));
        assert_eq!(ok("[1, 2, 3]"), Err(ForgeError::CellCount));
        assert!(ok("[0, 65535]").is_ok());

        // Taille de charge utile excédant un `u32` : refusée avant toute allocation.
        assert_eq!(
            with("65535", "65535", "\"t\"", "[]"),
            Err(ForgeError::PayloadTooLarge)
        );

        // Champ inconnu et TOML invalide.
        assert_eq!(
            parse_map("width = 1\nheight = 1\ntileset = \"t\"\ncells = [0]\nextra = 1\n"),
            Err(ForgeError::UnknownField("extra".to_string()))
        );
        assert!(matches!(
            parse_map("width = = 1"),
            Err(ForgeError::Syntax(_))
        ));
    }
}
