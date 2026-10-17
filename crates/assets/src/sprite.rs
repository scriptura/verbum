// crates/assets/src/sprite.rs

//! Artefact `sprite.bin` : layout et lecture (I3, I4, I5, I8, I10).
//!
//! Layout, little-endian explicite, 14 octets :
//!
//! | Offset | Taille | Champ |
//! |---:|---:|---|
//! | 0 | 4 | identité du format ([`FORMAT_ID`]) |
//! | 4 | 2 | version, `u16` ([`FORMAT_VERSION`]) |
//! | 6 | 4 | longueur de la charge utile, `u32` ([`PAYLOAD_LEN`]) |
//! | 10 | 2 | largeur, `u16` |
//! | 12 | 2 | hauteur, `u16` |
//!
//! Le lecteur ne contrôle que la conformité binaire (I5). Il n'applique aucune
//! validation de cohérence métier : une largeur nulle est conforme (I2, P1.6).

/// Identité du format, en tête d'artefact. Choix local provisoire, non invariant.
pub const FORMAT_ID: [u8; 4] = *b"VSPR";

/// Version du layout acceptée par ce lecteur. Choix local provisoire.
pub const FORMAT_VERSION: u16 = 1;

/// Longueur de l'en-tête : identité (4) + version (2) + longueur de charge utile (4).
pub const HEADER_LEN: usize = 10;

/// Longueur de la charge utile : largeur (2) + hauteur (2).
pub const PAYLOAD_LEN: usize = 4;

/// Longueur totale exacte d'un artefact conforme.
pub const ARTIFACT_LEN: usize = HEADER_LEN + PAYLOAD_LEN;

// Cohérence des constantes de layout, vérifiée à la compilation.
const _: () = {
    assert!(HEADER_LEN == FORMAT_ID.len() + size_of::<u16>() + size_of::<u32>());
    assert!(PAYLOAD_LEN == 2 * size_of::<u16>());
    assert!(ARTIFACT_LEN == HEADER_LEN + PAYLOAD_LEN);
    assert!(PAYLOAD_LEN <= u32::MAX as usize);
};

/// Rejet d'un artefact non conforme au layout. Énumération `Copy`, sans allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadError {
    /// Moins d'octets que le layout n'en exige.
    Truncated,
    /// Identité du format différente de [`FORMAT_ID`].
    FormatIdentity,
    /// Version différente de [`FORMAT_VERSION`].
    Version,
    /// Longueur de charge utile déclarée différente de [`PAYLOAD_LEN`].
    PayloadLength,
    /// Octets au-delà de [`ARTIFACT_LEN`].
    TrailingBytes,
}

/// Descripteur de sprite décodé. Donnée pure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub width: u16,
    pub height: u16,
}

impl Sprite {
    /// Décode un artefact complet à partir de ses octets.
    ///
    /// Ordre des contrôles : en-tête complet, identité, version, longueur de charge
    /// utile déclarée, puis longueur totale (troncature, surplus).
    pub fn read(bytes: &[u8]) -> Result<Self, ReadError> {
        let [i0, i1, i2, i3, v0, v1, l0, l1, l2, l3, rest @ ..] = bytes else {
            return Err(ReadError::Truncated);
        };
        if [*i0, *i1, *i2, *i3] != FORMAT_ID {
            return Err(ReadError::FormatIdentity);
        }
        if u16::from_le_bytes([*v0, *v1]) != FORMAT_VERSION {
            return Err(ReadError::Version);
        }
        if u32::from_le_bytes([*l0, *l1, *l2, *l3]) != PAYLOAD_LEN as u32 {
            return Err(ReadError::PayloadLength);
        }
        match rest {
            [w0, w1, h0, h1] => Ok(Self {
                width: u16::from_le_bytes([*w0, *w1]),
                height: u16::from_le_bytes([*h0, *h1]),
            }),
            [_, _, _, _, ..] => Err(ReadError::TrailingBytes),
            _ => Err(ReadError::Truncated),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Artefact littéral écrit à la main, indépendant des constantes du module :
    /// identité `VSPR`, version 1, charge utile 4, largeur 0x0102, hauteur 0x0304.
    const VALID: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, // identité
        0x01, 0x00, // version, little-endian
        0x04, 0x00, 0x00, 0x00, // longueur de charge utile, little-endian
        0x02, 0x01, // largeur 0x0102, little-endian
        0x04, 0x03, // hauteur 0x0304, little-endian
    ];

    #[test]
    fn p1_1_reads_the_literal_artifact() {
        assert_eq!(
            Sprite::read(&VALID),
            Ok(Sprite {
                width: 0x0102,
                height: 0x0304
            })
        );
    }

    #[test]
    fn p1_2_every_truncation_is_rejected() {
        for len in 0..VALID.len() {
            assert_eq!(
                Sprite::read(&VALID[..len]),
                Err(ReadError::Truncated),
                "préfixe de {len} octets"
            );
        }
    }

    #[test]
    fn p1_3_altered_format_identity_is_rejected() {
        for index in 0..4 {
            let mut bytes = VALID;
            bytes[index] ^= 0xFF;
            assert_eq!(
                Sprite::read(&bytes),
                Err(ReadError::FormatIdentity),
                "octet d'identité {index} altéré"
            );
        }
        // Le texte d'un `sprite.toml` n'est pas un artefact (I1).
        let source = b"width = 258\nheight = 772\n";
        assert!(source.len() >= ARTIFACT_LEN);
        assert_eq!(Sprite::read(source), Err(ReadError::FormatIdentity));
    }

    #[test]
    fn p1_4_unknown_version_is_rejected() {
        for version in [[0x00, 0x00], [0x02, 0x00], [0x00, 0x01], [0xFF, 0xFF]] {
            let mut bytes = VALID;
            bytes[4..6].copy_from_slice(&version);
            assert_eq!(
                Sprite::read(&bytes),
                Err(ReadError::Version),
                "version {version:?}"
            );
        }
    }

    #[test]
    fn p1_5_wrong_payload_length_or_surplus_is_rejected() {
        for declared in [
            [0x00, 0x00, 0x00, 0x00],
            [0x03, 0x00, 0x00, 0x00],
            [0x05, 0x00, 0x00, 0x00],
            [0x04, 0x00, 0x00, 0x01],
            [0xFF, 0xFF, 0xFF, 0xFF],
        ] {
            let mut bytes = VALID;
            bytes[6..10].copy_from_slice(&declared);
            assert_eq!(
                Sprite::read(&bytes),
                Err(ReadError::PayloadLength),
                "longueur déclarée {declared:?}"
            );
        }

        let mut surplus = [0u8; 15];
        surplus[..14].copy_from_slice(&VALID);
        assert_eq!(Sprite::read(&surplus), Err(ReadError::TrailingBytes));
    }

    #[test]
    fn p1_6_zero_width_is_structurally_conform_and_accepted() {
        let mut bytes = VALID;
        bytes[10..12].copy_from_slice(&[0x00, 0x00]);
        assert_eq!(
            Sprite::read(&bytes),
            Ok(Sprite {
                width: 0,
                height: 0x0304
            })
        );
    }
}
