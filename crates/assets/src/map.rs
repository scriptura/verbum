// crates/assets/src/map.rs

//! Artefact `map.bin` : layout, lecture et vue empruntée (I3, I4, I5, I8, I10).
//!
//! Layout, little-endian explicite, à charge utile variable (préfixe commun de 10
//! octets, puis charge utile) :
//!
//! | Offset | Taille | Champ |
//! |---:|---:|---|
//! | 0 | 4 | identité du format ([`MAP_FORMAT_ID`]) |
//! | 4 | 2 | version, `u16` ([`MAP_FORMAT_VERSION`]) |
//! | 6 | 4 | longueur de la charge utile, `u32` : `12 + 2·width·height` |
//! | 10 | 8 | `AssetId` du tileset référencé, `u64` |
//! | 18 | 2 | largeur, `u16` |
//! | 20 | 2 | hauteur, `u16` |
//! | 22 | `2·width·height` | indices de case, `u16`, ligne par ligne |
//!
//! Le lecteur ne contrôle que la conformité binaire (I5) : aucune valeur n'est validée
//! sémantiquement. La longueur déduite de la largeur et de la hauteur est calculée en
//! `u64` : elle ne déborde jamais et un artefact trop court est rejeté sans panique.
//!
//! [`Map`] est une **vue** : les octets de l'artefact restent possédés par
//! l'infrastructure de chargement (A-2, option a). La vue porte les champs d'en-tête
//! décodés, le [`SlotIndex`] du tileset **résolu au chargement** (jamais suivi au
//! runtime) et la tranche empruntée des indices, décodés à la demande.

use crate::header::read_prefix;
use crate::manifest::{AssetId, SlotIndex};
use crate::sprite::ReadError;

/// Identité du format, en tête d'artefact. Choix local provisoire, non invariant.
pub const MAP_FORMAT_ID: [u8; 4] = *b"VMAP";

/// Version du layout acceptée par ce lecteur. Choix local provisoire.
pub const MAP_FORMAT_VERSION: u16 = 1;

/// Partie fixe de la charge utile : identifiant du tileset (8) + largeur (2) + hauteur (2).
pub const MAP_FIXED_PAYLOAD_LEN: usize = 12;

// Cohérence des constantes de layout, vérifiée à la compilation.
const _: () = {
    assert!(MAP_FIXED_PAYLOAD_LEN == size_of::<u64>() + 2 * size_of::<u16>());
};

/// Champs décodés d'un artefact conforme, avant résolution de la référence.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MapParts<'a> {
    /// Identifiant du tileset référencé, tel qu'écrit dans l'artefact.
    pub(crate) tileset: AssetId,
    pub(crate) width: u16,
    pub(crate) height: u16,
    /// Indices de case : exactement `2·width·height` octets.
    pub(crate) cells: &'a [u8],
}

/// Décode un artefact `map.bin` à partir de ses octets.
///
/// Ordre des contrôles : préfixe complet, identité, version, longueur déclarée inférieure
/// à la partie fixe (`PayloadLength`), octets restants inférieurs ou supérieurs à la
/// longueur déclarée (`Truncated`, `TrailingBytes`), puis longueur déduite des champs
/// (en `u64`) différente de la longueur déclarée (`PayloadLength`).
pub(crate) fn read(bytes: &[u8]) -> Result<MapParts<'_>, ReadError> {
    let prefix = read_prefix(bytes, MAP_FORMAT_ID, MAP_FORMAT_VERSION)?;
    let declared = u64::from(prefix.declared_len);
    if declared < MAP_FIXED_PAYLOAD_LEN as u64 {
        return Err(ReadError::PayloadLength);
    }
    let available = u64::try_from(prefix.rest.len()).unwrap_or(u64::MAX);
    if available < declared {
        return Err(ReadError::Truncated);
    }
    if available > declared {
        return Err(ReadError::TrailingBytes);
    }
    let Some((fixed, cells)) = prefix.rest.split_at_checked(MAP_FIXED_PAYLOAD_LEN) else {
        return Err(ReadError::Truncated);
    };
    let Ok(&[t0, t1, t2, t3, t4, t5, t6, t7, w0, w1, h0, h1]) =
        <&[u8; MAP_FIXED_PAYLOAD_LEN]>::try_from(fixed)
    else {
        return Err(ReadError::Truncated);
    };
    let width = u16::from_le_bytes([w0, w1]);
    let height = u16::from_le_bytes([h0, h1]);
    let deduced = MAP_FIXED_PAYLOAD_LEN as u64 + 2 * u64::from(width) * u64::from(height);
    if deduced != declared {
        return Err(ReadError::PayloadLength);
    }
    Ok(MapParts {
        tileset: AssetId::from_raw(u64::from_le_bytes([t0, t1, t2, t3, t4, t5, t6, t7])),
        width,
        height,
        cells,
    })
}

/// Carte déposée : vue empruntée sur les octets d'un artefact `map.bin` conforme.
///
/// Aucune allocation, aucun `unsafe` : les accès décodent à la demande par
/// `from_le_bytes`, et rendent `None` hors bornes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Map<'a> {
    tileset: SlotIndex,
    width: u16,
    height: u16,
    cells: &'a [u8],
}

impl<'a> Map<'a> {
    /// Assemble la vue à partir des champs décodés et du slot du tileset résolu.
    pub(crate) fn new(tileset: SlotIndex, parts: MapParts<'a>) -> Self {
        Self {
            tileset,
            width: parts.width,
            height: parts.height,
            cells: parts.cells,
        }
    }

    /// Slot du tileset référencé, résolu au chargement (le slot, non l'artefact).
    pub fn tileset(&self) -> SlotIndex {
        self.tileset
    }

    /// Largeur en cases.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Hauteur en cases.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Indice de la case `(column, row)`, ou `None` hors de la carte. Ne lit jamais
    /// au-delà de la charge utile validée.
    pub fn tile(&self, column: u16, row: u16) -> Option<u16> {
        if column >= self.width || row >= self.height {
            return None;
        }
        let index = u64::from(row) * u64::from(self.width) + u64::from(column);
        let offset = usize::try_from(index * 2).ok()?;
        match self.cells.get(offset..offset.checked_add(2)?)? {
            [low, high] => Some(u16::from_le_bytes([*low, *high])),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Identifiant de tileset du littéral : 0x1122_3344_5566_7788 (distinct de toute autre
    /// valeur du lot de test).
    const TILESET_ID: u64 = 0x1122_3344_5566_7788;

    /// Carte 3 × 2 écrite à la main : identité `VMAP`, version 1, charge utile 24
    /// (`12 + 2·3·2`), identifiant de tileset, largeur 3, hauteur 2, six indices
    /// 0x0001, 0x0203, 0x0405, 0x0607, 0x0809, 0x0A0B.
    const VALID: [u8; 34] = [
        0x56, 0x4D, 0x41, 0x50, // identité
        0x01, 0x00, // version
        0x18, 0x00, 0x00, 0x00, // longueur de charge utile 24
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // identifiant du tileset
        0x03, 0x00, // largeur 3
        0x02, 0x00, // hauteur 2
        0x01, 0x00, 0x03, 0x02, 0x05, 0x04, // ligne 0
        0x07, 0x06, 0x09, 0x08, 0x0B, 0x0A, // ligne 1
    ];

    /// Carte 0 × 0 : charge utile de 12 octets, aucun indice.
    const EMPTY: [u8; 22] = [
        0x56, 0x4D, 0x41, 0x50, 0x01, 0x00, 0x0C, 0x00, 0x00, 0x00, // préfixe, charge 12
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // identifiant du tileset
        0x00, 0x00, 0x00, 0x00, // 0 × 0
    ];

    fn view(bytes: &[u8]) -> Map<'_> {
        let parts = read(bytes).expect("artefact conforme");
        Map::new(SlotIndex::new(7), parts)
    }

    #[test]
    fn map_r1_reads_the_literal_artifact() {
        let parts = read(&VALID).expect("artefact conforme");
        assert_eq!(parts.tileset, AssetId::from_raw(TILESET_ID));
        assert_eq!(parts.width, 3);
        assert_eq!(parts.height, 2);
        assert_eq!(parts.cells, &VALID[22..]);
    }

    #[test]
    fn map_r2_every_truncation_is_rejected() {
        for len in 0..VALID.len() {
            assert!(
                matches!(read(&VALID[..len]), Err(ReadError::Truncated)),
                "préfixe de {len} octets : {:?}",
                read(&VALID[..len]).err()
            );
        }
    }

    #[test]
    fn map_r3_altered_format_identity_is_rejected() {
        for index in 0..4 {
            let mut bytes = VALID;
            bytes[index] ^= 0xFF;
            assert_eq!(
                read(&bytes).err(),
                Some(ReadError::FormatIdentity),
                "octet d'identité {index} altéré"
            );
        }
    }

    #[test]
    fn map_r4_unknown_version_is_rejected() {
        for version in [[0x00, 0x00], [0x02, 0x00], [0x00, 0x01], [0xFF, 0xFF]] {
            let mut bytes = VALID;
            bytes[4..6].copy_from_slice(&version);
            assert_eq!(
                read(&bytes).err(),
                Some(ReadError::Version),
                "version {version:?}"
            );
        }
    }

    #[test]
    fn map_r5_inconsistent_lengths_are_rejected_in_the_documented_order() {
        // Longueur déclarée inférieure à la partie fixe (12) : `PayloadLength`.
        for declared in [0u32, 1, 11] {
            let mut bytes = VALID;
            bytes[6..10].copy_from_slice(&declared.to_le_bytes());
            assert_eq!(read(&bytes).err(), Some(ReadError::PayloadLength));
        }
        // Longueur déclarée supérieure aux octets restants : `Truncated`.
        let mut bytes = VALID;
        bytes[6..10].copy_from_slice(&25u32.to_le_bytes());
        assert_eq!(read(&bytes).err(), Some(ReadError::Truncated));
        // Longueur déclarée inférieure aux octets restants (≥ 12) : `TrailingBytes`.
        let mut bytes = VALID;
        bytes[6..10].copy_from_slice(&23u32.to_le_bytes());
        assert_eq!(read(&bytes).err(), Some(ReadError::TrailingBytes));
        // Surplus d'un octet avec la longueur déclarée inchangée : `TrailingBytes`.
        let mut surplus = [0u8; 35];
        surplus[..34].copy_from_slice(&VALID);
        assert_eq!(read(&surplus).err(), Some(ReadError::TrailingBytes));
        // Déclarée = octets restants, mais ≠ longueur déduite des champs : `PayloadLength`.
        let mut bytes = VALID;
        bytes[20..22].copy_from_slice(&[0x03, 0x00]); // hauteur 3 : 12 + 2·3·3 = 30 ≠ 24
        assert_eq!(read(&bytes).err(), Some(ReadError::PayloadLength));
        let mut bytes = VALID;
        bytes[18..20].copy_from_slice(&[0x02, 0x00]); // largeur 2 : 12 + 2·2·2 = 20 ≠ 24
        assert_eq!(read(&bytes).err(), Some(ReadError::PayloadLength));
    }

    #[test]
    fn map_r6_an_empty_map_is_conform_and_a_zero_length_payload_is_not() {
        let parts = read(&EMPTY).expect("carte 0 × 0 conforme");
        assert_eq!((parts.width, parts.height, parts.cells.len()), (0, 0, 0));
        assert_eq!(parts.tileset, AssetId::from_raw(TILESET_ID));

        // Charge utile de longueur nulle : invalide (inférieure à la partie fixe).
        let mut zero = [0u8; 10];
        zero[..10].copy_from_slice(&EMPTY[..10]);
        zero[6..10].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(read(&zero).err(), Some(ReadError::PayloadLength));

        // Largeur nulle, hauteur non nulle : zéro case, conforme (I2).
        let mut flat = EMPTY;
        flat[20..22].copy_from_slice(&[0x05, 0x00]);
        assert_eq!(view(&flat).tile(0, 0), None);
    }

    #[test]
    fn map_r7_maximal_dimensions_with_few_bytes_are_rejected_without_panic() {
        // 0xFFFF × 0xFFFF, charge utile de 12 octets : longueur déduite 2·65535² + 12 > u32.
        let mut bytes = EMPTY;
        bytes[18..22].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(read(&bytes).err(), Some(ReadError::PayloadLength));

        // Même dimensions, longueur déclarée maximale, peu d'octets : troncature.
        let mut huge = EMPTY;
        huge[18..22].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        huge[6..10].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(read(&huge).err(), Some(ReadError::Truncated));

        // 0xFFFF × 2 : 12 + 2·65535·2 = 262 152 octets, le test n'en fournit que 12.
        let mut wide = EMPTY;
        wide[18..22].copy_from_slice(&[0xFF, 0xFF, 0x02, 0x00]);
        assert_eq!(read(&wide).err(), Some(ReadError::PayloadLength));

        // 0xFFFF × 0x8001 : 2·width·height = 4 295 032 830 = 2³² + 65 534. Réduit modulo
        // 2³², le produit vaudrait 65 534 : un calcul en `u32` accepterait cette
        // charge utile de 65 546 octets. En `u64` elle est rejetée.
        let mut bytes = [0u8; 10 + 12 + 65_534];
        bytes[..22].copy_from_slice(&EMPTY);
        bytes[6..10].copy_from_slice(&65_546u32.to_le_bytes());
        bytes[18..22].copy_from_slice(&[0xFF, 0xFF, 0x01, 0x80]);
        assert_eq!(read(&bytes).err(), Some(ReadError::PayloadLength));
    }

    #[test]
    fn map_r8_view_access_matches_the_literal_and_never_reads_past_the_payload() {
        let map = view(&VALID);
        assert_eq!((map.width(), map.height()), (3, 2));
        assert_eq!(map.tileset(), SlotIndex::new(7));
        let expected = [[0x0001, 0x0203, 0x0405], [0x0607, 0x0809, 0x0A0B]];
        for (row, line) in expected.iter().enumerate() {
            for (column, value) in line.iter().enumerate() {
                assert_eq!(
                    map.tile(column as u16, row as u16),
                    Some(*value),
                    "case ({column}, {row})"
                );
            }
        }
        // Dernier indice accessible, premier inaccessible : colonne `width`, ligne `height`.
        assert_eq!(map.tile(2, 1), Some(0x0A0B));
        for row in 0..2 {
            assert_eq!(map.tile(3, row), None, "colonne 3, ligne {row}");
        }
        for column in 0..3 {
            assert_eq!(map.tile(column, 2), None, "colonne {column}, ligne 2");
        }
        assert_eq!(map.tile(u16::MAX, u16::MAX), None);
        // Une carte vide n'a aucune case accessible.
        assert_eq!(view(&EMPTY).tile(0, 0), None);
    }
}
