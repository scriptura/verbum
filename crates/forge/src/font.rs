// crates/forge/src/font.rs

//! Compilation d'un `font.toml` en artefact `font.bin` (I1, I2, I4, I9, I11).
//!
//! Source : `line_height` dans `1..=65535` et un tableau non vide de tables `glyph`
//! (1 à 65535 éléments) portant chacune `codepoint` (valeur scalaire Unicode : hors
//! substituts, au plus `0x10FFFF`), `width` dans `0..=65535` (zéro admis : glyphe sans
//! encre) et `advance` dans `1..=65535`. Les points de code doivent être strictement
//! croissants. Ces règles sémantiques sont celles de la Forge seule : le lecteur runtime
//! ne contrôle que la structure binaire (I2). Aucune chaîne, aucun fichier de police.

use verbum_assets::{FONT_FIXED_PAYLOAD_LEN, FONT_FORMAT_ID, FONT_FORMAT_VERSION, FONT_RECORD_LEN};

use crate::ForgeError;
use crate::sprite::dimension;

/// Glyphe validé, avant encodage.
struct GlyphSource {
    codepoint: u32,
    width: u16,
    advance: u16,
}

/// Compile une source `font.toml` en octets d'artefact.
///
/// Ordre des contrôles : syntaxe, champ inconnu de premier niveau, `line_height`
/// (présence, type, zéro, intervalle), `glyph` (présence, tableau de tables), tableau
/// non vide, au plus 65535 glyphes ; puis, glyphe par glyphe dans l'ordre de la source :
/// champ inconnu, `codepoint` (présence, type, intervalle, valeur scalaire), `width`
/// (présence, type, intervalle), `advance` (présence, type, zéro, intervalle), enfin
/// point de code strictement supérieur à celui du glyphe précédent.
pub fn compile_font(source: &str) -> Result<Vec<u8>, ForgeError> {
    let table: toml::Table = source
        .parse()
        .map_err(|error: toml::de::Error| ForgeError::Syntax(error.to_string()))?;

    if let Some(unknown) = table
        .keys()
        .find(|key| !matches!(key.as_str(), "line_height" | "glyph"))
    {
        return Err(ForgeError::UnknownField(unknown.clone()));
    }

    let line_height = dimension(&table, "line_height")?;

    let glyphs = table
        .get("glyph")
        .ok_or(ForgeError::MissingField("glyph"))?;
    let toml::Value::Array(items) = glyphs else {
        return Err(ForgeError::WrongType("glyph"));
    };
    if items.is_empty() {
        return Err(ForgeError::EmptyGlyphs);
    }
    let Ok(glyph_count) = u16::try_from(items.len()) else {
        return Err(ForgeError::TooManyGlyphs);
    };

    let mut parsed: Vec<GlyphSource> = Vec::with_capacity(items.len());
    for item in items {
        let toml::Value::Table(glyph) = item else {
            return Err(ForgeError::WrongType("glyph"));
        };
        let glyph = parse_glyph(glyph)?;
        if parsed
            .last()
            .is_some_and(|previous| glyph.codepoint <= previous.codepoint)
        {
            return Err(ForgeError::CodepointOrder);
        }
        parsed.push(glyph);
    }

    let payload = FONT_FIXED_PAYLOAD_LEN + FONT_RECORD_LEN * parsed.len();
    let payload_len = u32::try_from(payload).map_err(|_| ForgeError::PayloadTooLarge)?;
    let mut bytes = Vec::with_capacity(10 + payload);
    bytes.extend_from_slice(&FONT_FORMAT_ID);
    bytes.extend_from_slice(&FONT_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&payload_len.to_le_bytes());
    bytes.extend_from_slice(&glyph_count.to_le_bytes());
    bytes.extend_from_slice(&line_height.to_le_bytes());
    for glyph in &parsed {
        bytes.extend_from_slice(&glyph.codepoint.to_le_bytes());
        bytes.extend_from_slice(&glyph.width.to_le_bytes());
        bytes.extend_from_slice(&glyph.advance.to_le_bytes());
    }
    debug_assert_eq!(bytes.len(), 10 + payload);
    Ok(bytes)
}

/// Analyse une table `glyph`.
fn parse_glyph(glyph: &toml::Table) -> Result<GlyphSource, ForgeError> {
    if let Some(unknown) = glyph
        .keys()
        .find(|key| !matches!(key.as_str(), "codepoint" | "width" | "advance"))
    {
        return Err(ForgeError::UnknownField(unknown.clone()));
    }

    let codepoint = integer(glyph, "codepoint")?;
    let codepoint = u32::try_from(codepoint).map_err(|_| ForgeError::OutOfRange("codepoint"))?;
    if char::from_u32(codepoint).is_none() {
        return Err(ForgeError::InvalidCodepoint);
    }
    let width = integer(glyph, "width")?;
    let width = u16::try_from(width).map_err(|_| ForgeError::OutOfRange("width"))?;
    let advance = dimension(glyph, "advance")?;
    Ok(GlyphSource {
        codepoint,
        width,
        advance,
    })
}

/// Extrait un champ entier (présence, type).
fn integer(table: &toml::Table, field: &'static str) -> Result<i64, ForgeError> {
    match table.get(field).ok_or(ForgeError::MissingField(field))? {
        toml::Value::Integer(value) => Ok(*value),
        _ => Err(ForgeError::WrongType(field)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use verbum_assets::{Font, Glyph};

    /// Source de référence : 3 glyphes, hauteur de ligne 0x0A0B = 2571 ; glyphes
    /// (0x41, 5, 6), (0xE9, 7, 8), (0x1F600, 0, 9).
    const SOURCE: &str = "line_height = 2571\n\
        [[glyph]]\ncodepoint = 65\nwidth = 5\nadvance = 6\n\
        [[glyph]]\ncodepoint = 233\nwidth = 7\nadvance = 8\n\
        [[glyph]]\ncodepoint = 128512\nwidth = 0\nadvance = 9\n";

    /// Octets attendus pour `SOURCE`, écrits à la main (mêmes valeurs que FNT-R1).
    const EXPECTED: [u8; 38] = [
        0x56, 0x46, 0x4E, 0x54, // identité
        0x01, 0x00, // version
        0x1C, 0x00, 0x00, 0x00, // longueur de charge utile 28
        0x03, 0x00, // 3 glyphes
        0x0B, 0x0A, // hauteur de ligne
        0x41, 0x00, 0x00, 0x00, 0x05, 0x00, 0x06, 0x00, // glyphe 0
        0xE9, 0x00, 0x00, 0x00, 0x07, 0x00, 0x08, 0x00, // glyphe 1
        0x00, 0xF6, 0x01, 0x00, 0x00, 0x00, 0x09, 0x00, // glyphe 2
    ];

    /// Source à un glyphe, avec remplacement d'un champ.
    fn one(line_height: &str, codepoint: &str, width: &str, advance: &str) -> String {
        format!(
            "line_height = {line_height}\n[[glyph]]\ncodepoint = {codepoint}\nwidth = {width}\nadvance = {advance}\n"
        )
    }

    #[test]
    fn fnt_f1_a_valid_source_produces_exactly_the_literal_bytes() {
        assert_eq!(compile_font(SOURCE).as_deref(), Ok(EXPECTED.as_slice()));
    }

    #[test]
    fn fnt_f2_forge_output_is_read_back_by_the_runtime_reader() {
        let bytes = compile_font(SOURCE).expect("source valide");
        let font = Font::read(&bytes).expect("artefact conforme");
        assert_eq!((font.len(), font.line_height()), (3, 2571));
        let expected = [
            Glyph {
                codepoint: 0x41,
                width: 5,
                advance: 6,
            },
            Glyph {
                codepoint: 0xE9,
                width: 7,
                advance: 8,
            },
            Glyph {
                codepoint: 0x1_F600,
                width: 0,
                advance: 9,
            },
        ];
        for (index, glyph) in (0u16..).zip(expected) {
            assert_eq!(font.glyph(index), Some(glyph));
        }
        assert_eq!(font.glyph(3), None);

        // Bornes : 1 glyphe extrême, puis 65535 glyphes (maximum), charge `4 + 8·65535`.
        let extreme = one("65535", "1114111", "65535", "65535");
        let bytes = compile_font(&extreme).expect("source valide");
        let font = Font::read(&bytes).expect("artefact conforme");
        assert_eq!(
            font.glyph(0),
            Some(Glyph {
                codepoint: 0x10_FFFF,
                width: 65535,
                advance: 65535
            })
        );
        let mut many = String::from("line_height = 1\n");
        for index in 0u32..65535 {
            // Saute les substituts : 65535 valeurs scalaires strictement croissantes.
            let codepoint = if index < 0xD800 { index } else { index + 0x800 };
            many.push_str(&format!(
                "[[glyph]]\ncodepoint = {codepoint}\nwidth = 1\nadvance = 1\n"
            ));
        }
        let bytes = compile_font(&many).expect("source valide");
        assert_eq!(bytes.len(), 10 + 4 + 8 * 65535);
        let font = Font::read(&bytes).expect("artefact conforme");
        assert_eq!(font.len(), u16::MAX);
        assert_eq!(
            font.glyph(u16::MAX - 1).map(|glyph| glyph.codepoint),
            Some(65534 + 0x800)
        );
    }

    #[test]
    fn fnt_f3_every_invalid_source_is_refused() {
        // Syntaxe, champ inconnu (premier niveau et glyphe).
        assert!(matches!(
            compile_font("line_height = = 1"),
            Err(ForgeError::Syntax(_))
        ));
        assert_eq!(
            compile_font(&format!("{SOURCE}extra = 1\n")),
            Err(ForgeError::UnknownField("extra".to_string()))
        );
        assert_eq!(
            compile_font(
                "line_height = 1\n[[glyph]]\ncodepoint = 1\nwidth = 1\nadvance = 1\nextra = 1\n"
            ),
            Err(ForgeError::UnknownField("extra".to_string()))
        );

        // Champs manquants.
        assert_eq!(
            compile_font(""),
            Err(ForgeError::MissingField("line_height"))
        );
        assert_eq!(
            compile_font("line_height = 1\n"),
            Err(ForgeError::MissingField("glyph"))
        );
        for (source, field) in [
            ("[[glyph]]\nwidth = 1\nadvance = 1\n", "codepoint"),
            ("[[glyph]]\ncodepoint = 1\nadvance = 1\n", "width"),
            ("[[glyph]]\ncodepoint = 1\nwidth = 1\n", "advance"),
        ] {
            assert_eq!(
                compile_font(&format!("line_height = 1\n{source}")),
                Err(ForgeError::MissingField(field))
            );
        }

        // `line_height` : zéro, négatif, hors `u16`, mauvais type.
        assert_eq!(
            compile_font(&one("0", "1", "1", "1")),
            Err(ForgeError::ZeroValue("line_height"))
        );
        for bad in ["-1", "65536"] {
            assert_eq!(
                compile_font(&one(bad, "1", "1", "1")),
                Err(ForgeError::OutOfRange("line_height"))
            );
        }
        assert_eq!(
            compile_font(&one("\"4\"", "1", "1", "1")),
            Err(ForgeError::WrongType("line_height"))
        );

        // Champs de glyphe : intervalles et types.
        assert_eq!(
            compile_font(&one("1", "-1", "1", "1")),
            Err(ForgeError::OutOfRange("codepoint"))
        );
        assert_eq!(
            compile_font(&one("1", "4294967296", "1", "1")),
            Err(ForgeError::OutOfRange("codepoint"))
        );
        assert_eq!(
            compile_font(&one("1", "1.5", "1", "1")),
            Err(ForgeError::WrongType("codepoint"))
        );
        for bad in ["-1", "65536"] {
            assert_eq!(
                compile_font(&one("1", "1", bad, "1")),
                Err(ForgeError::OutOfRange("width"))
            );
            assert_eq!(
                compile_font(&one("1", "1", "1", bad)),
                Err(ForgeError::OutOfRange("advance"))
            );
        }
        assert_eq!(
            compile_font(&one("1", "1", "\"4\"", "1")),
            Err(ForgeError::WrongType("width"))
        );
        assert_eq!(
            compile_font(&one("1", "1", "1", "1.5")),
            Err(ForgeError::WrongType("advance"))
        );

        // `glyph` n'est pas un tableau de tables.
        assert_eq!(
            compile_font("line_height = 1\nglyph = 3\n"),
            Err(ForgeError::WrongType("glyph"))
        );
        assert_eq!(
            compile_font("line_height = 1\nglyph = [1]\n"),
            Err(ForgeError::WrongType("glyph"))
        );
    }

    #[test]
    fn fnt_f4_the_compilation_is_deterministic_and_independent_of_field_order() {
        let first = compile_font(SOURCE).expect("source valide");
        assert_eq!(compile_font(SOURCE), Ok(first.clone()));
        // Ordre des champs dans la source, au premier niveau et dans les glyphes.
        let permuted = "line_height = 2571\n\
            [[glyph]]\nadvance = 6\nwidth = 5\ncodepoint = 65\n\
            [[glyph]]\nadvance = 8\ncodepoint = 233\nwidth = 7\n\
            [[glyph]]\nwidth = 0\nadvance = 9\ncodepoint = 128512\n";
        // Avec `glyph` en ligne, `line_height` peut aussi suivre le tableau.
        let inline = "glyph = [{ advance = 6, width = 5, codepoint = 65 }, \
            { codepoint = 233, width = 7, advance = 8 }, \
            { codepoint = 128512, width = 0, advance = 9 }]\nline_height = 2571\n";
        assert_eq!(compile_font(inline), Ok(first.clone()));
        assert_eq!(compile_font(permuted), Ok(first));
    }

    #[test]
    fn fnt_f5_the_output_carries_no_field_name_and_has_the_expected_length() {
        let bytes = compile_font(SOURCE).expect("source valide");
        assert_eq!(bytes.len(), 38);
        for name in ["line_height", "glyph", "codepoint", "width", "advance"] {
            assert!(
                !bytes
                    .windows(name.len())
                    .any(|window| window == name.as_bytes()),
                "le nom de champ « {name} » figure dans la sortie"
            );
        }
    }

    #[test]
    fn fnt_f6_semantic_rules_are_refused_by_the_forge_and_accepted_by_the_reader() {
        // Tableau vide : refusé ; le lecteur accepte pourtant une police sans glyphe (FNT-R6).
        assert_eq!(
            compile_font("line_height = 1\nglyph = []\n"),
            Err(ForgeError::EmptyGlyphs)
        );
        // Plus de 65535 glyphes : refusé avant toute analyse des glyphes.
        let too_many = format!("line_height = 1\nglyph = [{}]\n", "{},".repeat(65536));
        assert_eq!(compile_font(&too_many), Err(ForgeError::TooManyGlyphs));

        // Point de code non scalaire : substituts et au-delà de 0x10FFFF.
        for bad in ["55296", "57343", "1114112"] {
            assert_eq!(
                compile_font(&one("1", bad, "1", "1")),
                Err(ForgeError::InvalidCodepoint),
                "point de code {bad}"
            );
        }
        // Bornes valides voisines des substituts.
        for good in ["0", "55295", "57344", "1114111"] {
            assert!(compile_font(&one("1", good, "1", "1")).is_ok(), "{good}");
        }

        // Avance nulle : refusée ; largeur nulle : acceptée.
        assert_eq!(
            compile_font(&one("1", "1", "1", "0")),
            Err(ForgeError::ZeroValue("advance"))
        );
        assert!(compile_font(&one("1", "1", "0", "1")).is_ok());

        // Ordre des points de code : égal ou décroissant refusé.
        let pair = |first: u32, second: u32| {
            format!(
                "line_height = 1\n[[glyph]]\ncodepoint = {first}\nwidth = 1\nadvance = 1\n\
                 [[glyph]]\ncodepoint = {second}\nwidth = 1\nadvance = 1\n"
            )
        };
        assert_eq!(compile_font(&pair(66, 65)), Err(ForgeError::CodepointOrder));
        assert_eq!(compile_font(&pair(65, 65)), Err(ForgeError::CodepointOrder));
        assert!(compile_font(&pair(65, 66)).is_ok());

        // Le lecteur, lui, accepte un ordre quelconque et une avance nulle (I2).
        let mut bytes = compile_font(&pair(65, 66)).expect("source valide");
        // Échange des deux points de code, avance du premier glyphe mise à zéro.
        bytes[14..18].copy_from_slice(&66u32.to_le_bytes());
        bytes[22..26].copy_from_slice(&65u32.to_le_bytes());
        bytes[20..22].copy_from_slice(&0u16.to_le_bytes());
        let font = Font::read(&bytes).expect("conforme pour le lecteur");
        assert_eq!(
            font.glyph(0).map(|g| (g.codepoint, g.advance)),
            Some((66, 0))
        );
    }
}
