// crates/assets/src/header.rs

//! Préfixe commun des artefacts (I3, I5, I10).
//!
//! Tous les formats débutent par le même préfixe de 10 octets : identité (4), version
//! `u16` (2), longueur de charge utile `u32` (4), little-endian lu champ à champ. Ce
//! module est le seul endroit qui le décode ; les lecteurs de format poursuivent avec
//! les contrôles qui leur sont propres (longueur déclarée, longueur totale).

use crate::sprite::ReadError;

/// Longueur du préfixe commun : identité (4) + version (2) + longueur de charge utile (4).
pub(crate) const PREFIX_LEN: usize = 10;

/// Préfixe contrôlé : longueur de charge utile déclarée et octets qui suivent le préfixe.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Prefix<'b> {
    /// Longueur de charge utile déclarée par l'artefact, non encore confrontée aux octets.
    pub(crate) declared_len: u32,
    /// Octets situés après le préfixe.
    pub(crate) rest: &'b [u8],
}

/// Contrôle le préfixe dans l'ordre : complétude (`Truncated`), identité
/// (`FormatIdentity`), version (`Version`). Rend la longueur déclarée et les octets
/// restants ; la suite des contrôles revient au lecteur du format.
pub(crate) fn read_prefix<'b>(
    bytes: &'b [u8],
    identity: [u8; 4],
    version: u16,
) -> Result<Prefix<'b>, ReadError> {
    if bytes.len() < PREFIX_LEN {
        return Err(ReadError::Truncated);
    }
    let (head, rest) = bytes.split_at(PREFIX_LEN);
    if head[..4] != identity {
        return Err(ReadError::FormatIdentity);
    }
    if u16::from_le_bytes([head[4], head[5]]) != version {
        return Err(ReadError::Version);
    }
    Ok(Prefix {
        declared_len: u32::from_le_bytes([head[6], head[7], head[8], head[9]]),
        rest,
    })
}

/// Identité portée par `bytes`, si le préfixe de 10 octets est complet ; `None` sinon.
pub(crate) fn identity_of(bytes: &[u8]) -> Option<[u8; 4]> {
    if bytes.len() < PREFIX_LEN {
        return None;
    }
    Some([bytes[0], bytes[1], bytes[2], bytes[3]])
}
