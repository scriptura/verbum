// crates/assets/src/tileset.rs

//! Artefact `tileset.bin` : layout et lecture (I3, I4, I5, I8, I10).
//!
//! Layout, little-endian explicite, 18 octets (préfixe commun de 10 octets, puis charge
//! utile de longueur fixe) :
//!
//! | Offset | Taille | Champ |
//! |---:|---:|---|
//! | 0 | 4 | identité du format ([`TILESET_FORMAT_ID`]) |
//! | 4 | 2 | version, `u16` ([`TILESET_FORMAT_VERSION`]) |
//! | 6 | 4 | longueur de la charge utile, `u32` ([`TILESET_PAYLOAD_LEN`]) |
//! | 10 | 2 | largeur d'une tuile, `u16` |
//! | 12 | 2 | hauteur d'une tuile, `u16` |
//! | 14 | 2 | nombre de colonnes, `u16` |
//! | 16 | 2 | nombre de tuiles, `u16` |
//!
//! Le lecteur ne contrôle que la conformité binaire (I5). Il n'applique aucune
//! validation de cohérence métier : des zéros, ou `columns > count`, sont conformes
//! (I2) ; seule la Forge refuse ces valeurs.

use crate::header::{PREFIX_LEN, read_prefix};
use crate::sprite::ReadError;

/// Identité du format, en tête d'artefact. Choix local provisoire, non invariant.
pub const TILESET_FORMAT_ID: [u8; 4] = *b"VTLS";

/// Version du layout acceptée par ce lecteur. Choix local provisoire.
pub const TILESET_FORMAT_VERSION: u16 = 1;

/// Longueur de la charge utile : quatre `u16`.
pub const TILESET_PAYLOAD_LEN: usize = 8;

/// Longueur totale exacte d'un artefact conforme.
pub const TILESET_ARTIFACT_LEN: usize = PREFIX_LEN + TILESET_PAYLOAD_LEN;

// Cohérence des constantes de layout, vérifiée à la compilation.
const _: () = {
    assert!(TILESET_PAYLOAD_LEN == 4 * size_of::<u16>());
    assert!(TILESET_ARTIFACT_LEN == 18);
    assert!(TILESET_PAYLOAD_LEN <= u32::MAX as usize);
};

/// Descripteur de tileset décodé. Donnée pure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tileset {
    pub tile_width: u16,
    pub tile_height: u16,
    pub columns: u16,
    pub count: u16,
}

impl Tileset {
    /// Décode un artefact complet à partir de ses octets.
    ///
    /// Ordre des contrôles : préfixe complet, identité, version, longueur de charge
    /// utile déclarée, puis longueur totale (troncature, surplus).
    pub fn read(bytes: &[u8]) -> Result<Self, ReadError> {
        let prefix = read_prefix(bytes, TILESET_FORMAT_ID, TILESET_FORMAT_VERSION)?;
        if prefix.declared_len != TILESET_PAYLOAD_LEN as u32 {
            return Err(ReadError::PayloadLength);
        }
        match prefix.rest {
            [w0, w1, h0, h1, c0, c1, n0, n1] => Ok(Self {
                tile_width: u16::from_le_bytes([*w0, *w1]),
                tile_height: u16::from_le_bytes([*h0, *h1]),
                columns: u16::from_le_bytes([*c0, *c1]),
                count: u16::from_le_bytes([*n0, *n1]),
            }),
            [_, _, _, _, _, _, _, _, ..] => Err(ReadError::TrailingBytes),
            _ => Err(ReadError::Truncated),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Artefact littéral écrit à la main, indépendant des constantes du module :
    /// identité `VTLS`, version 1, charge utile 8, `tile_width` 0x0102,
    /// `tile_height` 0x0304, `columns` 0x0506, `count` 0x0708 (valeurs distinctes par
    /// champ : une permutation ou une inversion d'endianness est visible).
    const VALID: [u8; 18] = [
        0x56, 0x54, 0x4C, 0x53, // identité
        0x01, 0x00, // version, little-endian
        0x08, 0x00, 0x00, 0x00, // longueur de charge utile, little-endian
        0x02, 0x01, // tile_width 0x0102
        0x04, 0x03, // tile_height 0x0304
        0x06, 0x05, // columns 0x0506
        0x08, 0x07, // count 0x0708
    ];

    #[test]
    fn tls_r1_reads_the_literal_artifact() {
        assert_eq!(
            Tileset::read(&VALID),
            Ok(Tileset {
                tile_width: 0x0102,
                tile_height: 0x0304,
                columns: 0x0506,
                count: 0x0708
            })
        );
    }

    #[test]
    fn tls_r2_every_truncation_is_rejected() {
        for len in 0..VALID.len() {
            assert_eq!(
                Tileset::read(&VALID[..len]),
                Err(ReadError::Truncated),
                "préfixe de {len} octets"
            );
        }
    }

    #[test]
    fn tls_r3_altered_format_identity_is_rejected() {
        for index in 0..4 {
            let mut bytes = VALID;
            bytes[index] ^= 0xFF;
            assert_eq!(
                Tileset::read(&bytes),
                Err(ReadError::FormatIdentity),
                "octet d'identité {index} altéré"
            );
        }
        // L'identité d'un autre format connu n'est pas celle du tileset.
        let mut sprite = VALID;
        sprite[..4].copy_from_slice(b"VSPR");
        assert_eq!(Tileset::read(&sprite), Err(ReadError::FormatIdentity));
    }

    #[test]
    fn tls_r4_unknown_version_is_rejected() {
        for version in [[0x00, 0x00], [0x02, 0x00], [0x00, 0x01], [0xFF, 0xFF]] {
            let mut bytes = VALID;
            bytes[4..6].copy_from_slice(&version);
            assert_eq!(
                Tileset::read(&bytes),
                Err(ReadError::Version),
                "version {version:?}"
            );
        }
    }

    #[test]
    fn tls_r5_wrong_payload_length_or_surplus_is_rejected() {
        for declared in [
            [0x00, 0x00, 0x00, 0x00],
            [0x07, 0x00, 0x00, 0x00],
            [0x09, 0x00, 0x00, 0x00],
            [0x08, 0x00, 0x00, 0x01],
            [0xFF, 0xFF, 0xFF, 0xFF],
        ] {
            let mut bytes = VALID;
            bytes[6..10].copy_from_slice(&declared);
            assert_eq!(
                Tileset::read(&bytes),
                Err(ReadError::PayloadLength),
                "longueur déclarée {declared:?}"
            );
        }

        let mut surplus = [0u8; 19];
        surplus[..18].copy_from_slice(&VALID);
        assert_eq!(Tileset::read(&surplus), Err(ReadError::TrailingBytes));
    }

    #[test]
    fn tls_r6_boundary_values_are_structurally_conform_and_accepted() {
        // Zéros et `columns > count` : conformes au layout (I2), refusés par la Forge seule.
        let mut zeros = VALID;
        zeros[10..18].copy_from_slice(&[0x00; 8]);
        assert_eq!(
            Tileset::read(&zeros),
            Ok(Tileset {
                tile_width: 0,
                tile_height: 0,
                columns: 0,
                count: 0
            })
        );

        let mut maxima = VALID;
        maxima[10..18].copy_from_slice(&[0xFF; 8]);
        assert_eq!(
            Tileset::read(&maxima),
            Ok(Tileset {
                tile_width: u16::MAX,
                tile_height: u16::MAX,
                columns: u16::MAX,
                count: u16::MAX
            })
        );

        // `columns` 3, `count` 2 : incohérent pour la Forge, conforme pour le lecteur.
        let mut inverted = VALID;
        inverted[14..18].copy_from_slice(&[0x03, 0x00, 0x02, 0x00]);
        assert_eq!(
            Tileset::read(&inverted),
            Ok(Tileset {
                tile_width: 0x0102,
                tile_height: 0x0304,
                columns: 3,
                count: 2
            })
        );
    }
}
