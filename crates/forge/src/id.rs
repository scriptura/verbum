// crates/forge/src/id.rs

//! Identifiant d'asset : FNV-1a 64 bits de la clé logique (I4, I12).
//!
//! Algorithme (RFC 9923 §2) : `hash = offset_basis` ; pour chaque octet,
//! `hash = hash XOR octet` puis `hash = hash × FNV_Prime mod 2^64`. Constantes 64 bits
//! (RFC 9923 §5). La clé est prise en octets UTF-8 exacts : sensible à la casse, sans
//! normalisation. L'identifiant ne dépend ni du contenu de l'asset, ni de sa position.

/// `offset_basis` FNV 64 bits.
const FNV64_OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;

/// `FNV_Prime` 64 bits (2⁴⁰ + 2⁸ + 0xB3).
const FNV64_PRIME: u64 = 0x0000_0100_0000_01B3;

/// FNV-1a 64 bits de `bytes`.
pub(crate) fn fnv1a_64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV64_OFFSET_BASIS, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV64_PRIME)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Vecteurs de référence du RFC 9923 §8.3, écrits en littéraux : ils ne sont pas
    // calculés par la fonction testée.

    #[test]
    fn f1_the_empty_input_yields_the_offset_basis() {
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn f1_the_single_byte_a_matches_the_reference_vector() {
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn f1_foobar_matches_the_reference_vector() {
        assert_eq!(fnv1a_64(b"foobar"), 0x8594_4171_f739_67e8);
    }
}
