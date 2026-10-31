// crates/forge/src/tileset.rs

//! Compilation d'un `tileset.toml` en artefact `tileset.bin` (I1, I2, I4, I9, I11).
//!
//! Source : quatre champs entiers, `tile_width`, `tile_height`, `columns` et `count`,
//! chacun dans `1..=65535`, avec `columns <= count`. Aucune chaîne, aucun décodage
//! d'image : la source ne désigne aucun fichier.

use verbum_assets::{
    TILESET_ARTIFACT_LEN, TILESET_FORMAT_ID, TILESET_FORMAT_VERSION, TILESET_PAYLOAD_LEN,
};

use crate::ForgeError;
use crate::sprite::dimension;

/// Compile une source `tileset.toml` en octets d'artefact.
///
/// Ordre des contrôles : syntaxe, champ inconnu, puis `tile_width`, `tile_height`,
/// `columns` et `count` dans cet ordre (présence, type, zéro, intervalle), enfin
/// `columns <= count`.
pub fn compile_tileset(source: &str) -> Result<Vec<u8>, ForgeError> {
    let table: toml::Table = source
        .parse()
        .map_err(|error: toml::de::Error| ForgeError::Syntax(error.to_string()))?;

    if let Some(unknown) = table.keys().find(|key| {
        !matches!(
            key.as_str(),
            "tile_width" | "tile_height" | "columns" | "count"
        )
    }) {
        return Err(ForgeError::UnknownField(unknown.clone()));
    }

    let tile_width = dimension(&table, "tile_width")?;
    let tile_height = dimension(&table, "tile_height")?;
    let columns = dimension(&table, "columns")?;
    let count = dimension(&table, "count")?;
    if columns > count {
        return Err(ForgeError::ColumnsExceedCount);
    }

    let mut bytes = Vec::with_capacity(TILESET_ARTIFACT_LEN);
    bytes.extend_from_slice(&TILESET_FORMAT_ID);
    bytes.extend_from_slice(&TILESET_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(TILESET_PAYLOAD_LEN as u32).to_le_bytes());
    bytes.extend_from_slice(&tile_width.to_le_bytes());
    bytes.extend_from_slice(&tile_height.to_le_bytes());
    bytes.extend_from_slice(&columns.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    debug_assert_eq!(bytes.len(), TILESET_ARTIFACT_LEN);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use verbum_assets::Tileset;

    /// Source de référence : valeurs distinctes par champ (0x0102, 0x0304, 0x0506, 0x0708).
    const SOURCE: &str = "tile_width = 258\ntile_height = 772\ncolumns = 1286\ncount = 1800\n";

    /// Octets attendus pour `SOURCE`, écrits à la main : témoin indépendant des constantes
    /// du layout et de la constante de TLS-R1 (même valeurs, autre écriture).
    const EXPECTED: [u8; 18] = [
        0x56, 0x54, 0x4C, 0x53, // identité
        0x01, 0x00, // version, little-endian
        0x08, 0x00, 0x00, 0x00, // longueur de charge utile, little-endian
        0x02, 0x01, // tile_width 0x0102
        0x04, 0x03, // tile_height 0x0304
        0x06, 0x05, // columns 0x0506
        0x08, 0x07, // count 0x0708
    ];

    #[test]
    fn tls_f1_a_valid_source_produces_exactly_the_literal_bytes() {
        let bytes = compile_tileset(SOURCE);
        assert_eq!(bytes.as_deref(), Ok(EXPECTED.as_slice()));
    }

    #[test]
    fn tls_f2_forge_output_is_read_back_by_the_runtime_reader() {
        for (tile_width, tile_height, columns, count) in [
            (1u16, 1u16, 1u16, 1u16),
            (258, 772, 1286, 1800),
            (65535, 1, 1, 65535),
            (1, 65535, 65535, 65535),
            (65535, 65535, 3, 7),
        ] {
            let source = format!(
                "tile_width = {tile_width}\ntile_height = {tile_height}\ncolumns = {columns}\ncount = {count}\n"
            );
            let bytes = compile_tileset(&source).expect("source valide");
            assert_eq!(
                Tileset::read(&bytes),
                Ok(Tileset {
                    tile_width,
                    tile_height,
                    columns,
                    count
                })
            );
        }
    }

    #[test]
    fn tls_f3_every_invalid_source_is_refused() {
        // Champ manquant : l'ordre de contrôle est celui des champs.
        assert_eq!(
            compile_tileset(""),
            Err(ForgeError::MissingField("tile_width"))
        );
        assert_eq!(
            compile_tileset("tile_width = 1\n"),
            Err(ForgeError::MissingField("tile_height"))
        );
        assert_eq!(
            compile_tileset("tile_width = 1\ntile_height = 1\n"),
            Err(ForgeError::MissingField("columns"))
        );
        assert_eq!(
            compile_tileset("tile_width = 1\ntile_height = 1\ncolumns = 1\n"),
            Err(ForgeError::MissingField("count"))
        );

        // Zéro, négatif, hors `u16`, mauvais type : chacun des quatre champs.
        for field in ["tile_width", "tile_height", "columns", "count"] {
            let with = |value: &str| {
                let mut source = String::new();
                for name in ["tile_width", "tile_height", "columns", "count"] {
                    let text = if name == field { value } else { "1" };
                    source.push_str(&format!("{name} = {text}\n"));
                }
                compile_tileset(&source)
            };
            let field: &'static str = match field {
                "tile_width" => "tile_width",
                "tile_height" => "tile_height",
                "columns" => "columns",
                _ => "count",
            };
            assert_eq!(with("0"), Err(ForgeError::ZeroValue(field)), "{field} = 0");
            assert_eq!(
                with("-1"),
                Err(ForgeError::OutOfRange(field)),
                "{field} = -1"
            );
            assert_eq!(
                with("65536"),
                Err(ForgeError::OutOfRange(field)),
                "{field} = 65536"
            );
            assert_eq!(
                with("\"4\""),
                Err(ForgeError::WrongType(field)),
                "{field} chaîne"
            );
            assert_eq!(
                with("1.5"),
                Err(ForgeError::WrongType(field)),
                "{field} flottant"
            );
        }

        // Champ inconnu et TOML invalide.
        assert_eq!(
            compile_tileset("tile_width = 1\ntile_height = 1\ncolumns = 1\ncount = 1\nextra = 1\n"),
            Err(ForgeError::UnknownField("extra".to_string()))
        );
        assert!(matches!(
            compile_tileset("tile_width = = 1"),
            Err(ForgeError::Syntax(_))
        ));

        // `columns > count` : refusé par la Forge, accepté par le lecteur (TLS-R6).
        assert_eq!(
            compile_tileset("tile_width = 1\ntile_height = 1\ncolumns = 3\ncount = 2\n"),
            Err(ForgeError::ColumnsExceedCount)
        );
        // `columns == count` est conforme.
        assert!(
            compile_tileset("tile_width = 1\ntile_height = 1\ncolumns = 2\ncount = 2\n").is_ok()
        );
    }

    #[test]
    fn tls_f4_the_compilation_is_deterministic() {
        let first = compile_tileset(SOURCE).expect("source valide");
        let second = compile_tileset(SOURCE).expect("source valide");
        assert_eq!(first, second);
        // L'ordre des champs dans la source n'influe pas sur les octets.
        let reordered = "count = 1800\ncolumns = 1286\ntile_height = 772\ntile_width = 258\n";
        assert_eq!(compile_tileset(reordered), Ok(first));
    }

    #[test]
    fn tls_f5_the_output_carries_no_field_name_and_has_the_expected_length() {
        let bytes = compile_tileset(SOURCE).expect("source valide");
        assert_eq!(bytes.len(), 18);
        for name in ["tile_width", "tile_height", "columns", "count"] {
            assert!(
                !bytes
                    .windows(name.len())
                    .any(|window| window == name.as_bytes()),
                "le nom de champ {name} figure dans l'artefact"
            );
        }
    }
}
