// crates/forge/src/sprite.rs

//! Compilation d'un `sprite.toml` en artefact `sprite.bin` (I1, I2, I4, I9, I11).
//!
//! Source : deux champs entiers, `width` et `height`, chacun dans `1..=65535`.
//! Aucune chaîne, aucune durée, aucun décodage d'image.

use verbum_assets::{ARTIFACT_LEN, FORMAT_ID, FORMAT_VERSION, PAYLOAD_LEN};

/// Rejet d'une source de sprite. Aucun artefact n'est produit en cas d'erreur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForgeError {
    /// La source n'est pas un TOML valide ; le message vient du parseur.
    Syntax(String),
    /// Champ requis absent.
    MissingField(&'static str),
    /// Champ non reconnu.
    UnknownField(String),
    /// Le champ n'est pas un entier.
    WrongType(&'static str),
    /// Le champ vaut zéro.
    ZeroValue(&'static str),
    /// Le champ est négatif ou dépasse `u16::MAX`.
    OutOfRange(&'static str),
    /// Compilation en lot : la source de l'entrée `entry` est rejetée (position base zéro).
    InvalidSource {
        entry: usize,
        cause: Box<ForgeError>,
    },
    /// Compilation en lot : la clé logique de l'entrée `entry` est vide.
    EmptyKey { entry: usize },
    /// Compilation en lot : deux entrées portent la même clé logique.
    DuplicateKey { first: usize, second: usize },
    /// Compilation en lot : deux clés distinctes donnent le même identifiant.
    IdCollision { first: usize, second: usize },
    /// Compilation en lot : le `slot_index` de l'entrée `entry` dépasse `slot_count`.
    SlotOutOfRange { entry: usize },
    /// Compilation en lot : deux entrées visent le même slot.
    DuplicateSlot { first: usize, second: usize },
    /// Compilation en lot : le lot compte plus d'entrées qu'un `u32` ne peut en numéroter.
    TooManyAssets,
}

/// Compile une source `sprite.toml` en octets d'artefact.
///
/// Ordre des contrôles : syntaxe, champ inconnu, puis `width` et `height` dans cet ordre
/// (présence, type, zéro, intervalle).
pub fn compile_sprite(source: &str) -> Result<Vec<u8>, ForgeError> {
    let table: toml::Table = source
        .parse()
        .map_err(|error: toml::de::Error| ForgeError::Syntax(error.to_string()))?;

    if let Some(unknown) = table
        .keys()
        .find(|key| !matches!(key.as_str(), "width" | "height"))
    {
        return Err(ForgeError::UnknownField(unknown.clone()));
    }

    let width = dimension(&table, "width")?;
    let height = dimension(&table, "height")?;

    let mut bytes = Vec::with_capacity(ARTIFACT_LEN);
    bytes.extend_from_slice(&FORMAT_ID);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(PAYLOAD_LEN as u32).to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&height.to_le_bytes());
    debug_assert_eq!(bytes.len(), ARTIFACT_LEN);
    Ok(bytes)
}

/// Extrait un champ entier non nul tenant dans un `u16`.
fn dimension(table: &toml::Table, field: &'static str) -> Result<u16, ForgeError> {
    let value = table.get(field).ok_or(ForgeError::MissingField(field))?;
    let toml::Value::Integer(integer) = value else {
        return Err(ForgeError::WrongType(field));
    };
    match u16::try_from(*integer) {
        Ok(0) => Err(ForgeError::ZeroValue(field)),
        Ok(dimension) => Ok(dimension),
        Err(_) => Err(ForgeError::OutOfRange(field)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use verbum_assets::Sprite;

    /// Octets attendus pour `width = 258`, `height = 772` (0x0102 et 0x0304), écrits à
    /// la main : témoin indépendant de la constante de P1.1 et des constantes du layout.
    const EXPECTED: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, // identité
        0x01, 0x00, // version, little-endian
        0x04, 0x00, 0x00, 0x00, // longueur de charge utile, little-endian
        0x02, 0x01, // largeur 0x0102, little-endian
        0x04, 0x03, // hauteur 0x0304, little-endian
    ];

    #[test]
    fn p2_1_a_valid_source_produces_exactly_the_literal_bytes() {
        let bytes = compile_sprite("width = 258\nheight = 772\n");
        assert_eq!(bytes.as_deref(), Ok(EXPECTED.as_slice()));
    }

    #[test]
    fn p2_2_forge_output_is_read_back_by_the_runtime_reader() {
        for (width, height) in [
            (1u16, 1u16),
            (258, 772),
            (65535, 1),
            (1, 65535),
            (65535, 65535),
        ] {
            let source = format!("width = {width}\nheight = {height}\n");
            let bytes = compile_sprite(&source).expect("source valide");
            assert_eq!(Sprite::read(&bytes), Ok(Sprite { width, height }));
        }
    }

    #[test]
    fn p2_3_missing_field_is_refused() {
        assert_eq!(compile_sprite(""), Err(ForgeError::MissingField("width")));
        assert_eq!(
            compile_sprite("height = 4"),
            Err(ForgeError::MissingField("width"))
        );
        assert_eq!(
            compile_sprite("width = 4"),
            Err(ForgeError::MissingField("height"))
        );
    }

    #[test]
    fn p2_3_zero_value_is_refused() {
        assert_eq!(
            compile_sprite("width = 0\nheight = 4"),
            Err(ForgeError::ZeroValue("width"))
        );
        assert_eq!(
            compile_sprite("width = 4\nheight = 0"),
            Err(ForgeError::ZeroValue("height"))
        );
    }

    #[test]
    fn p2_3_value_outside_u16_is_refused() {
        assert_eq!(
            compile_sprite("width = 65536\nheight = 4"),
            Err(ForgeError::OutOfRange("width"))
        );
        assert_eq!(
            compile_sprite("width = 4\nheight = -1"),
            Err(ForgeError::OutOfRange("height"))
        );
        assert_eq!(
            compile_sprite("width = 9223372036854775807\nheight = 4"),
            Err(ForgeError::OutOfRange("width"))
        );
    }

    #[test]
    fn p2_3_wrong_type_is_refused() {
        for value in ["\"4\"", "4.0", "true", "[4]", "{ a = 4 }", "1979-05-27"] {
            assert_eq!(
                compile_sprite(&format!("width = {value}\nheight = 4")),
                Err(ForgeError::WrongType("width")),
                "width = {value}"
            );
        }
        assert_eq!(
            compile_sprite("width = 4\nheight = \"4\""),
            Err(ForgeError::WrongType("height"))
        );
    }

    #[test]
    fn p2_3_unknown_field_is_refused() {
        assert_eq!(
            compile_sprite("width = 4\nheight = 4\ndepth = 4"),
            Err(ForgeError::UnknownField("depth".to_owned()))
        );
        // La casse est significative.
        assert_eq!(
            compile_sprite("Width = 4\nwidth = 4\nheight = 4"),
            Err(ForgeError::UnknownField("Width".to_owned()))
        );
    }

    #[test]
    fn p2_3_invalid_toml_is_refused() {
        for source in [
            "width = ",
            "width = 4\nwidth = 5\nheight = 4",
            "width 4",
            "[width\nheight = 4",
            "width = 4\nheight = 4\n=",
        ] {
            assert!(
                matches!(compile_sprite(source), Err(ForgeError::Syntax(_))),
                "source : {source:?}"
            );
        }
    }
}
