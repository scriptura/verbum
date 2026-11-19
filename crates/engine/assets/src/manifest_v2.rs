// crates/engine/assets/src/manifest_v2.rs

//! Manifeste `VMAN` version 2 : constantes de layout et lecteur du niveau 1 (V2-L).
//!
//! Layout, little-endian explicite :
//!
//! | Champ | Taille |
//! |---|---:|
//! | signature [`SIGNATURE`] | 4 |
//! | version, `u16` ([`VERSION`]) | 2 |
//! | quantum `q_ns`, `u64`, en nanosecondes, strictement positif | 8 |
//! | nombre d'entrées `n`, `u32` | 4 |
//! | par entrée : [`AssetId`](crate::AssetId), `u64` | 8 |
//! | par entrée : `artifact_index`, `u32` | 4 |
//! | par entrée : `slot_index`, `u32` | 4 |
//!
//! Longueur totale attendue exactement `18 + 16 × n`. Les entrées sont identiques à
//! celles de la version 1. Ces constantes sont distinctes des constantes `MANIFEST_*`
//! de la version 1, qui conservent leur sens et leurs valeurs.

use core::num::NonZeroU64;

use crate::manifest::{MANIFEST_ENTRY_LEN, MANIFEST_SIGNATURE, Manifest, ManifestError};

/// Signature du manifeste v2 : celle de la version 1 (D-1d).
pub const SIGNATURE: [u8; 4] = *b"VMAN";

/// Version du layout du manifeste lue par le lecteur v2.
pub const VERSION: u16 = 2;

/// Longueur de l'en-tête : signature (4) + version (2) + `q_ns` (8) + nombre d'entrées (4).
pub const HEADER_LEN: usize = 18;

/// Longueur d'une entrée : identique à la version 1.
pub const ENTRY_LEN: usize = 16;

// Cohérence des constantes de layout, vérifiée à la compilation. `PartialEq` n'étant pas
// `const`, les signatures se comparent par leur valeur entière.
const _: () = {
    assert!(HEADER_LEN == SIGNATURE.len() + size_of::<u16>() + size_of::<u64>() + size_of::<u32>());
    assert!(ENTRY_LEN == MANIFEST_ENTRY_LEN);
    assert!(u32::from_le_bytes(SIGNATURE) == u32::from_le_bytes(MANIFEST_SIGNATURE));
};

/// Rejet d'un manifeste v2 au niveau 1, avant conversion en erreur publique (V2-P).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManifestV2Error {
    /// Erreur de lecture du manifeste, partagée avec la version 1.
    Manifest(ManifestError),
    /// `q_ns` vaut zéro.
    ZeroQuantum,
}

/// Niveau 1 pour un manifeste v2 : rend le quantum et la vue sur les entrées.
///
/// Disponibilité contrôlée champ par champ, dans l'ordre : six octets (`Truncated`),
/// signature (`Signature`), version (`Version`, avant toute exigence sur la suite), huit
/// octets de `q_ns` (`Truncated`), `q_ns` nul (`ZeroQuantum`), quatre octets de `n`
/// (`Truncated`), puis la validation commune des entrées : longueur attendue, longueur
/// exacte, unicité des identifiants. Aucune allocation.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "utilisé par le chargement de paquet, V2-P")
)]
pub(crate) fn read_manifest_v2(
    bytes: &[u8],
) -> Result<(NonZeroU64, Manifest<'_>), ManifestV2Error> {
    let truncated = ManifestV2Error::Manifest(ManifestError::Truncated);
    let [s0, s1, s2, s3, v0, v1, rest @ ..] = bytes else {
        return Err(truncated);
    };
    if [*s0, *s1, *s2, *s3] != SIGNATURE {
        return Err(ManifestV2Error::Manifest(ManifestError::Signature));
    }
    if u16::from_le_bytes([*v0, *v1]) != VERSION {
        return Err(ManifestV2Error::Manifest(ManifestError::Version));
    }
    let [q0, q1, q2, q3, q4, q5, q6, q7, rest @ ..] = rest else {
        return Err(truncated);
    };
    let Some(quantum) =
        NonZeroU64::new(u64::from_le_bytes([*q0, *q1, *q2, *q3, *q4, *q5, *q6, *q7]))
    else {
        return Err(ManifestV2Error::ZeroQuantum);
    };
    let [n0, n1, n2, n3, entries @ ..] = rest else {
        return Err(truncated);
    };
    let manifest = Manifest::from_region(u32::from_le_bytes([*n0, *n1, *n2, *n3]), entries)
        .map_err(ManifestV2Error::Manifest)?;
    Ok((quantum, manifest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{AssetId, ManifestEntry};

    /// Manifeste v2 littéral à deux entrées, écrit à la main, indépendant des constantes :
    /// signature `VMAN`, version 2, `q_ns` = 0x0102030405060708, n = 2.
    /// Entrée 0 : id 0x1122334455667788, artefact 0, slot 1.
    /// Entrée 1 : id 0x99AABBCCDDEEFF00, artefact 1, slot 0.
    const LITERAL: [u8; 50] = [
        0x56, 0x4D, 0x41, 0x4E, // signature
        0x02, 0x00, // version 2, little-endian
        0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // q_ns, little-endian
        0x02, 0x00, 0x00, 0x00, // n = 2, little-endian
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id 0
        0x00, 0x00, 0x00, 0x00, // artifact_index 0
        0x01, 0x00, 0x00, 0x00, // slot_index 1
        0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id 1
        0x01, 0x00, 0x00, 0x00, // artifact_index 1
        0x00, 0x00, 0x00, 0x00, // slot_index 0
    ];

    /// Manifeste v1 littéral équivalent (mêmes entrées), 42 octets.
    const LITERAL_V1: [u8; 42] = [
        0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // en-tête v1, n = 2
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, // entrée 0
        0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, // entrée 1
    ];

    const Q: u64 = 0x0102_0304_0506_0708;
    const ID_0: u64 = 0x1122_3344_5566_7788;
    const ID_1: u64 = 0x99AA_BBCC_DDEE_FF00;

    fn truncated() -> ManifestV2Error {
        ManifestV2Error::Manifest(ManifestError::Truncated)
    }

    fn error_of(bytes: &[u8]) -> Option<ManifestV2Error> {
        read_manifest_v2(bytes).err()
    }

    fn assert_entry(entry: ManifestEntry, id: u64, artifact: u32, slot: u32) {
        assert_eq!(entry.id(), AssetId::from_raw(id));
        assert_eq!(entry.artifact_index().get(), artifact);
        assert_eq!(entry.slot_index().get(), slot);
    }

    /// Manifeste v2 de `n` entrées (au plus 3) aux identifiants distincts, `q_ns` = 5.
    fn synthetic(n: u32, buffer: &mut [u8; 18 + 16 * 3]) -> &[u8] {
        buffer[..4].copy_from_slice(b"VMAN");
        buffer[4..6].copy_from_slice(&2u16.to_le_bytes());
        buffer[6..14].copy_from_slice(&5u64.to_le_bytes());
        buffer[14..18].copy_from_slice(&n.to_le_bytes());
        for i in 0..n {
            let at = 18 + 16 * i as usize;
            buffer[at..at + 8].copy_from_slice(&(0x100 + u64::from(i)).to_le_bytes());
            buffer[at + 8..at + 12].copy_from_slice(&i.to_le_bytes());
            buffer[at + 12..at + 16].copy_from_slice(&(n - 1 - i).to_le_bytes());
        }
        &buffer[..18 + 16 * n as usize]
    }

    #[test]
    fn mv1_reads_the_literal_manifest_and_its_quantum() {
        let (quantum, manifest) = read_manifest_v2(&LITERAL).expect("manifeste conforme");
        assert_eq!(quantum.get(), Q);
        assert_eq!(manifest.len(), 2);
        let mut entries = manifest.entries();
        assert_entry(entries.next().unwrap(), ID_0, 0, 1);
        assert_entry(entries.next().unwrap(), ID_1, 1, 0);
        assert!(entries.next().is_none());
        assert_entry(manifest.find(AssetId::from_raw(ID_0)).unwrap(), ID_0, 0, 1);
        assert_entry(manifest.find(AssetId::from_raw(ID_1)).unwrap(), ID_1, 1, 0);
        assert!(manifest.find(AssetId::from_raw(ID_0 ^ 1)).is_none());
    }

    #[test]
    fn mv2_a_zero_quantum_is_rejected_with_field_by_field_priority() {
        let zero_q = |bytes: &mut [u8]| bytes[6..14].fill(0);

        let mut bytes = LITERAL;
        zero_q(&mut bytes);
        assert_eq!(error_of(&bytes), Some(ManifestV2Error::ZeroQuantum));

        // Quatorze octets : q complet et nul, `n` absent : `ZeroQuantum` prime.
        let mut short = LITERAL;
        zero_q(&mut short);
        assert_eq!(error_of(&short[..14]), Some(ManifestV2Error::ZeroQuantum));

        // Longueur incohérente (n énorme, peu d'octets) : `ZeroQuantum` prime encore.
        let mut huge = LITERAL;
        zero_q(&mut huge);
        huge[14..18].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(error_of(&huge), Some(ManifestV2Error::ZeroQuantum));

        // q incomplet (13 octets) : `Truncated`, q n'est pas encore lisible.
        let mut partial = LITERAL;
        zero_q(&mut partial);
        assert_eq!(error_of(&partial[..13]), Some(truncated()));

        // q non nul : 14 à 17 octets donnent `Truncated`.
        for len in 14..18 {
            assert_eq!(error_of(&LITERAL[..len]), Some(truncated()), "{len} octets");
        }
    }

    #[test]
    fn mv3_the_length_must_be_exactly_18_plus_16_n() {
        for n in 0..=3u32 {
            let mut buffer = [0u8; 18 + 16 * 3];
            let bytes = synthetic(n, &mut buffer);
            assert_eq!(bytes.len(), 18 + 16 * n as usize);
            let (quantum, manifest) = read_manifest_v2(bytes).expect("longueur conforme");
            assert_eq!(quantum.get(), 5);
            assert_eq!(manifest.len(), n as usize);
            assert_eq!(manifest.entries().count(), n as usize);
        }

        for len in 0..LITERAL.len() {
            assert_eq!(error_of(&LITERAL[..len]), Some(truncated()), "{len} octets");
        }

        let mut surplus = [0u8; 51];
        surplus[..50].copy_from_slice(&LITERAL);
        assert_eq!(
            error_of(&surplus),
            Some(ManifestV2Error::Manifest(ManifestError::TrailingBytes))
        );
        let mut fewer = LITERAL;
        fewer[14..18].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(
            error_of(&fewer),
            Some(ManifestV2Error::Manifest(ManifestError::TrailingBytes))
        );

        for count in [u32::MAX, u32::MAX - 1, 0x1000_0000, 3] {
            let mut bytes = LITERAL;
            bytes[14..18].copy_from_slice(&count.to_le_bytes());
            assert_eq!(error_of(&bytes), Some(truncated()), "n = {count}");
        }
    }

    #[test]
    fn mv4_quantum_and_count_are_little_endian() {
        for (raw, expected) in [
            ([0x00, 0x01, 0, 0, 0, 0, 0, 0], 256u64),
            ([0, 0, 0, 0, 0, 0, 0, 0x01], 1u64 << 56),
            ([0xFF; 8], u64::MAX),
            ([0x01, 0, 0, 0, 0, 0, 0, 0], 1),
        ] {
            let mut bytes = LITERAL;
            bytes[6..14].copy_from_slice(&raw);
            let (quantum, _) = read_manifest_v2(&bytes).expect("q non nul");
            assert_eq!(quantum.get(), expected, "octets {raw:?}");
        }
        // Un `n` lu en big-endian vaudrait 0x0200_0000 : le littéral serait tronqué.
        let (_, manifest) = read_manifest_v2(&LITERAL).expect("conforme");
        assert_eq!(manifest.len(), 2);
    }

    #[test]
    fn mv5_a_version_one_manifest_never_yields_a_quantum() {
        assert_eq!(
            error_of(&LITERAL_V1),
            Some(ManifestV2Error::Manifest(ManifestError::Version))
        );
        // Manifeste v1 vide de 10 octets, plus court que l'en-tête v2.
        let empty_v1 = [0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(
            error_of(&empty_v1),
            Some(ManifestV2Error::Manifest(ManifestError::Version))
        );
        for version in [
            [0x00, 0x00],
            [0x01, 0x00],
            [0x03, 0x00],
            [0x00, 0x01],
            [0xFF, 0xFF],
        ] {
            let mut bytes = LITERAL;
            bytes[4..6].copy_from_slice(&version);
            assert_eq!(
                error_of(&bytes),
                Some(ManifestV2Error::Manifest(ManifestError::Version)),
                "version {version:?}"
            );
        }
        // La signature est contrôlée avant la version.
        for index in 0..4 {
            let mut bytes = LITERAL_V1;
            bytes[index] ^= 0xFF;
            assert_eq!(
                error_of(&bytes),
                Some(ManifestV2Error::Manifest(ManifestError::Signature)),
                "octet de signature {index}"
            );
        }
        // Moins de six octets, et six octets exactement : `Truncated`.
        for len in 0..=6 {
            assert_eq!(error_of(&LITERAL[..len]), Some(truncated()), "{len} octets");
        }
    }

    #[test]
    fn mv6_duplicate_asset_ids_are_rejected() {
        let mut adjacent = LITERAL;
        let first_id: [u8; 8] = LITERAL[18..26].try_into().unwrap();
        adjacent[34..42].copy_from_slice(&first_id);
        assert_eq!(
            error_of(&adjacent),
            Some(ManifestV2Error::Manifest(ManifestError::DuplicateId))
        );
    }

    #[test]
    fn mv7_the_shared_helper_gives_the_same_results_as_the_v1_reader() {
        // Mêmes régions d'entrées derrière un en-tête v1 puis v2, `n` déclaré = 2.
        let region = &LITERAL[18..];
        let mut variants: [([u8; 33], usize); 5] = [([0; 33], 0); 5];
        for (bytes, len) in &mut variants {
            bytes[..32].copy_from_slice(region);
            *len = 32;
        }
        variants[1].1 = 31; // tronquée
        variants[2].1 = 33; // surplus
        let first_id: [u8; 8] = region[..8].try_into().unwrap();
        variants[3].0[16..24].copy_from_slice(&first_id); // doublon
        variants[4].1 = 0; // région absente

        for (bytes, len) in variants {
            let mut v1 = [0u8; 10 + 33];
            v1[..10].copy_from_slice(&LITERAL_V1[..10]);
            v1[10..10 + len].copy_from_slice(&bytes[..len]);
            let mut v2 = [0u8; 18 + 33];
            v2[..18].copy_from_slice(&LITERAL[..18]);
            v2[18..18 + len].copy_from_slice(&bytes[..len]);

            let old = Manifest::parse(&v1[..10 + len]);
            let new = read_manifest_v2(&v2[..18 + len]);
            match (old, new) {
                (Ok(a), Ok((_, b))) => assert!(a.entries().eq(b.entries())),
                (Err(a), Err(ManifestV2Error::Manifest(b))) => assert_eq!(a, b),
                (a, b) => panic!("résultats divergents : {:?} / {:?}", a.err(), b.err()),
            }
        }
    }

    #[test]
    fn mv8_the_v1_reader_still_rejects_a_v2_manifest() {
        assert_eq!(
            Manifest::parse(&LITERAL).err(),
            Some(ManifestError::Version)
        );
        assert!(Manifest::parse(&LITERAL_V1).is_ok());
    }

    #[test]
    fn mv9_the_v2_constants_keep_their_literal_values() {
        assert_eq!(SIGNATURE, [0x56, 0x4D, 0x41, 0x4E]);
        assert_eq!(VERSION, 2);
        assert_eq!(HEADER_LEN, 18);
        assert_eq!(ENTRY_LEN, 16);
    }
}
