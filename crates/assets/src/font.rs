// crates/assets/src/font.rs

//! Artefact `font.bin` : layout, lecture et vue empruntée (I3, I4, I5, I8, I10).
//!
//! Layout, little-endian explicite, à charge utile variable (préfixe commun de 10
//! octets, puis charge utile) :
//!
//! | Offset | Taille | Champ |
//! |---:|---:|---|
//! | 0 | 4 | identité du format ([`FONT_FORMAT_ID`]) |
//! | 4 | 2 | version, `u16` ([`FONT_FORMAT_VERSION`]) |
//! | 6 | 4 | longueur de la charge utile, `u32` : `4 + 8·glyph_count` |
//! | 10 | 2 | nombre de glyphes, `u16` |
//! | 12 | 2 | hauteur de ligne, `u16` |
//! | 14 | `8·glyph_count` | enregistrements de glyphe |
//!
//! Enregistrement de glyphe, 8 octets : point de code `u32`, largeur `u16`, avance `u16`.
//!
//! Le lecteur ne contrôle que la conformité binaire (I5) : un point de code quelconque,
//! une avance nulle ou un ordre quelconque des enregistrements sont conformes si la
//! longueur l'est (I2) ; seule la Forge applique ces règles sémantiques. La longueur
//! déduite est calculée en `u64`, sans débordement ni panique.
//!
//! [`Font`] est une **vue** : les octets de l'artefact restent possédés par
//! l'infrastructure de chargement (A-2, option a) ; les enregistrements sont décodés à
//! la demande.

use crate::header::read_prefix;
use crate::sprite::ReadError;

/// Identité du format, en tête d'artefact. Choix local provisoire, non invariant.
pub const FONT_FORMAT_ID: [u8; 4] = *b"VFNT";

/// Version du layout acceptée par ce lecteur. Choix local provisoire.
pub const FONT_FORMAT_VERSION: u16 = 1;

/// Partie fixe de la charge utile : nombre de glyphes (2) + hauteur de ligne (2).
pub const FONT_FIXED_PAYLOAD_LEN: usize = 4;

/// Taille d'un enregistrement de glyphe : point de code (4) + largeur (2) + avance (2).
pub const FONT_RECORD_LEN: usize = 8;

// Cohérence des constantes de layout, vérifiée à la compilation.
const _: () = {
    assert!(FONT_FIXED_PAYLOAD_LEN == 2 * size_of::<u16>());
    assert!(FONT_RECORD_LEN == size_of::<u32>() + 2 * size_of::<u16>());
};

/// Glyphe décodé. Donnée pure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    pub codepoint: u32,
    pub width: u16,
    pub advance: u16,
}

/// Police : vue empruntée sur les octets d'un artefact `font.bin` conforme.
///
/// Aucune allocation, aucun `unsafe` : les accès décodent à la demande par
/// `from_le_bytes`, et rendent `None` hors bornes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Font<'a> {
    line_height: u16,
    glyph_count: u16,
    /// Enregistrements : exactement `8·glyph_count` octets.
    records: &'a [u8],
}

impl<'a> Font<'a> {
    /// Décode un artefact complet à partir de ses octets.
    ///
    /// Ordre des contrôles : préfixe complet, identité, version, longueur déclarée
    /// inférieure à la partie fixe (`PayloadLength`), octets restants inférieurs ou
    /// supérieurs à la longueur déclarée (`Truncated`, `TrailingBytes`), puis longueur
    /// déduite de `glyph_count` (en `u64`) différente de la longueur déclarée
    /// (`PayloadLength`).
    pub fn read(bytes: &'a [u8]) -> Result<Self, ReadError> {
        let prefix = read_prefix(bytes, FONT_FORMAT_ID, FONT_FORMAT_VERSION)?;
        let declared = u64::from(prefix.declared_len);
        if declared < FONT_FIXED_PAYLOAD_LEN as u64 {
            return Err(ReadError::PayloadLength);
        }
        let available = u64::try_from(prefix.rest.len()).unwrap_or(u64::MAX);
        if available < declared {
            return Err(ReadError::Truncated);
        }
        if available > declared {
            return Err(ReadError::TrailingBytes);
        }
        let Some((fixed, records)) = prefix.rest.split_at_checked(FONT_FIXED_PAYLOAD_LEN) else {
            return Err(ReadError::Truncated);
        };
        let Ok(&[n0, n1, h0, h1]) = <&[u8; FONT_FIXED_PAYLOAD_LEN]>::try_from(fixed) else {
            return Err(ReadError::Truncated);
        };
        let glyph_count = u16::from_le_bytes([n0, n1]);
        let deduced =
            FONT_FIXED_PAYLOAD_LEN as u64 + FONT_RECORD_LEN as u64 * u64::from(glyph_count);
        if deduced != declared {
            return Err(ReadError::PayloadLength);
        }
        Ok(Self {
            line_height: u16::from_le_bytes([h0, h1]),
            glyph_count,
            records,
        })
    }

    /// Hauteur de ligne.
    pub fn line_height(&self) -> u16 {
        self.line_height
    }

    /// Nombre de glyphes.
    pub fn len(&self) -> u16 {
        self.glyph_count
    }

    /// Vrai si la police ne compte aucun glyphe.
    pub fn is_empty(&self) -> bool {
        self.glyph_count == 0
    }

    /// Glyphe d'indice `index`, ou `None` hors de la police. Ne lit jamais au-delà de la
    /// charge utile validée.
    pub fn glyph(&self, index: u16) -> Option<Glyph> {
        if index >= self.glyph_count {
            return None;
        }
        let offset = usize::from(index) * FONT_RECORD_LEN;
        match self
            .records
            .get(offset..offset.checked_add(FONT_RECORD_LEN)?)?
        {
            [c0, c1, c2, c3, w0, w1, a0, a1] => Some(Glyph {
                codepoint: u32::from_le_bytes([*c0, *c1, *c2, *c3]),
                width: u16::from_le_bytes([*w0, *w1]),
                advance: u16::from_le_bytes([*a0, *a1]),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Police écrite à la main : identité `VFNT`, version 1, charge utile 28 (`4 + 8·3`),
    /// 3 glyphes, hauteur de ligne 0x0A0B ; glyphes (0x41, 5, 6), (0xE9, 7, 8),
    /// (0x1F600, 0, 9).
    const VALID: [u8; 38] = [
        0x56, 0x46, 0x4E, 0x54, // identité
        0x01, 0x00, // version
        0x1C, 0x00, 0x00, 0x00, // longueur de charge utile 28
        0x03, 0x00, // 3 glyphes
        0x0B, 0x0A, // hauteur de ligne 0x0A0B
        0x41, 0x00, 0x00, 0x00, 0x05, 0x00, 0x06, 0x00, // glyphe 0
        0xE9, 0x00, 0x00, 0x00, 0x07, 0x00, 0x08, 0x00, // glyphe 1
        0x00, 0xF6, 0x01, 0x00, 0x00, 0x00, 0x09, 0x00, // glyphe 2
    ];

    /// Police sans glyphe : charge utile de 4 octets.
    const EMPTY: [u8; 14] = [
        0x56, 0x46, 0x4E, 0x54, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, // préfixe, charge 4
        0x00, 0x00, // 0 glyphe
        0x0B, 0x0A, // hauteur de ligne
    ];

    #[test]
    fn fnt_r1_reads_the_literal_artifact() {
        let font = Font::read(&VALID).expect("artefact conforme");
        assert_eq!(font.len(), 3);
        assert_eq!(font.line_height(), 0x0A0B);
        assert_eq!(
            font.glyph(0),
            Some(Glyph {
                codepoint: 0x41,
                width: 5,
                advance: 6
            })
        );
        assert_eq!(
            font.glyph(1),
            Some(Glyph {
                codepoint: 0xE9,
                width: 7,
                advance: 8
            })
        );
        assert_eq!(
            font.glyph(2),
            Some(Glyph {
                codepoint: 0x1_F600,
                width: 0,
                advance: 9
            })
        );
    }

    #[test]
    fn fnt_r2_every_truncation_is_rejected() {
        for len in 0..VALID.len() {
            assert_eq!(
                Font::read(&VALID[..len]).err(),
                Some(ReadError::Truncated),
                "préfixe de {len} octets"
            );
        }
    }

    #[test]
    fn fnt_r3_altered_format_identity_is_rejected() {
        for index in 0..4 {
            let mut bytes = VALID;
            bytes[index] ^= 0xFF;
            assert_eq!(
                Font::read(&bytes).err(),
                Some(ReadError::FormatIdentity),
                "octet d'identité {index} altéré"
            );
        }
    }

    #[test]
    fn fnt_r4_unknown_version_is_rejected() {
        for version in [[0x00, 0x00], [0x02, 0x00], [0x00, 0x01], [0xFF, 0xFF]] {
            let mut bytes = VALID;
            bytes[4..6].copy_from_slice(&version);
            assert_eq!(
                Font::read(&bytes).err(),
                Some(ReadError::Version),
                "version {version:?}"
            );
        }
    }

    #[test]
    fn fnt_r5_inconsistent_lengths_are_rejected_in_the_documented_order() {
        // Longueur déclarée inférieure à la partie fixe (4) : `PayloadLength`.
        for declared in [0u32, 1, 3] {
            let mut bytes = VALID;
            bytes[6..10].copy_from_slice(&declared.to_le_bytes());
            assert_eq!(Font::read(&bytes).err(), Some(ReadError::PayloadLength));
        }
        // Longueur déclarée supérieure aux octets restants : `Truncated`.
        let mut bytes = VALID;
        bytes[6..10].copy_from_slice(&29u32.to_le_bytes());
        assert_eq!(Font::read(&bytes).err(), Some(ReadError::Truncated));
        // Longueur déclarée inférieure aux octets restants (≥ 4) : `TrailingBytes`.
        let mut bytes = VALID;
        bytes[6..10].copy_from_slice(&27u32.to_le_bytes());
        assert_eq!(Font::read(&bytes).err(), Some(ReadError::TrailingBytes));
        // Surplus d'un octet, longueur déclarée inchangée : `TrailingBytes`.
        let mut surplus = [0u8; 39];
        surplus[..38].copy_from_slice(&VALID);
        assert_eq!(Font::read(&surplus).err(), Some(ReadError::TrailingBytes));
        // Déclarée = octets restants, mais ≠ longueur déduite : `PayloadLength`.
        for count in [2u16, 4] {
            let mut bytes = VALID;
            bytes[10..12].copy_from_slice(&count.to_le_bytes());
            assert_eq!(
                Font::read(&bytes).err(),
                Some(ReadError::PayloadLength),
                "{count} glyphes annoncés pour 3 enregistrements"
            );
        }
    }

    #[test]
    fn fnt_r6_a_font_without_glyph_is_conform_and_a_zero_length_payload_is_not() {
        let font = Font::read(&EMPTY).expect("police sans glyphe conforme");
        assert_eq!((font.len(), font.line_height()), (0, 0x0A0B));
        assert!(font.is_empty());
        assert_eq!(font.glyph(0), None);

        // Charge utile de longueur nulle : invalide (inférieure à la partie fixe).
        let mut zero = [0u8; 10];
        zero.copy_from_slice(&EMPTY[..10]);
        zero[6..10].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(Font::read(&zero).err(), Some(ReadError::PayloadLength));
    }

    #[test]
    fn fnt_r7_an_oversized_glyph_count_is_rejected_without_panic_or_wraparound() {
        // 0xFFFF glyphes annoncés, charge utile de 4 octets : `4 + 8·65535 = 524 284`.
        let mut bytes = EMPTY;
        bytes[10..12].copy_from_slice(&[0xFF, 0xFF]);
        assert_eq!(Font::read(&bytes).err(), Some(ReadError::PayloadLength));

        // Même nombre, longueur déclarée maximale, peu d'octets : troncature.
        let mut huge = EMPTY;
        huge[10..12].copy_from_slice(&[0xFF, 0xFF]);
        huge[6..10].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(Font::read(&huge).err(), Some(ReadError::Truncated));

        // 0x2000 glyphes : `8·0x2000 = 65 536` vaut 0 en `u16`. Un calcul sur 16 bits
        // déduirait 4 et accepterait cette charge utile de 4 octets.
        let mut wrapped = EMPTY;
        wrapped[10..12].copy_from_slice(&[0x00, 0x20]);
        assert_eq!(Font::read(&wrapped).err(), Some(ReadError::PayloadLength));
    }

    #[test]
    fn fnt_r8_glyph_access_is_bounded_by_the_validated_payload() {
        let font = Font::read(&VALID).expect("artefact conforme");
        // Dernier indice accessible, premier inaccessible, indices extrêmes.
        assert!(font.glyph(2).is_some());
        assert_eq!(font.glyph(3), None);
        assert_eq!(font.glyph(4), None);
        assert_eq!(font.glyph(u16::MAX), None);
        let empty = Font::read(&EMPTY).expect("police sans glyphe conforme");
        assert_eq!(empty.glyph(0), None);
        assert_eq!(empty.glyph(u16::MAX), None);
    }
}
