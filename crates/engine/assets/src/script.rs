// crates/engine/assets/src/script.rs

//! Artefact `script.bin` : layout, lecture et vue empruntée (I3, I4, I5, I8, I10).
//!
//! Layout, little-endian explicite, à charge utile variable (préfixe commun de 10
//! octets, puis charge utile) :
//!
//! | Offset | Taille | Champ |
//! |---:|---:|---|
//! | 0 | 4 | identité du format ([`SCRIPT_FORMAT_ID`]) |
//! | 4 | 2 | version, `u16` ([`SCRIPT_FORMAT_VERSION`]) |
//! | 6 | 4 | longueur de la charge utile, `u32` : `2 + 8·n` |
//! | 10 | 2 | nombre d'instructions `n`, `u16` |
//! | 12 | `8·n` | enregistrements d'instruction |
//!
//! Enregistrement d'instruction, 8 octets : opcode `u8`, registre `u8`, `arg16` `u16`,
//! `arg32` `u32`.
//!
//! | Opcode | Instruction | registre | `arg16` | `arg32` |
//! |---:|---|---|---|---|
//! | 0 | [`Instruction::Halt`] | ignoré | ignoré | ignoré |
//! | 1 | [`Instruction::Emit`] | ignoré | canal | valeur `i32` |
//! | 2 | [`Instruction::Wait`] | ignoré | ignoré | ticks `u32`, au moins 1 |
//! | 3 | [`Instruction::Set`] | `< SCRIPT_REGISTERS` | ignoré | valeur `i32` |
//! | 4 | [`Instruction::Add`] | `< SCRIPT_REGISTERS` | ignoré | valeur `i32` |
//! | 5 | [`Instruction::Jump`] | ignoré | cible `< n` | ignoré |
//! | 6 | [`Instruction::JumpIfNonZero`] | `< SCRIPT_REGISTERS` | cible `< n` | ignoré |
//!
//! Le lecteur contrôle la conformité binaire (I5) : longueurs, puis chaque enregistrement
//! (opcode connu, registre et cible dans les bornes, attente non nulle). Les octets
//! ignorés par un opcode ne sont pas contrôlés : seule la Forge les écrit à zéro. Aucune
//! règle sémantique n'est appliquée (I2). La longueur déduite est calculée en `u64`, sans
//! débordement ni panique.
//!
//! [`Script`] est une **vue** : les octets de l'artefact restent possédés par
//! l'infrastructure de chargement (A-2, option a) ; les instructions sont décodées à la
//! demande.

use crate::header::read_prefix;
use crate::sprite::ReadError;

/// Identité du format, en tête d'artefact. Choix local provisoire, non invariant.
pub const SCRIPT_FORMAT_ID: [u8; 4] = *b"VSCR";

/// Version du layout acceptée par ce lecteur. Choix local provisoire.
pub const SCRIPT_FORMAT_VERSION: u16 = 1;

/// Partie fixe de la charge utile : nombre d'instructions (2).
pub const SCRIPT_FIXED_PAYLOAD_LEN: usize = 2;

/// Taille d'un enregistrement d'instruction : opcode (1) + registre (1) + `arg16` (2) +
/// `arg32` (4).
pub const SCRIPT_INSTRUCTION_LEN: usize = 8;

/// Nombre de registres d'un script. Choix local provisoire, non invariant.
pub const SCRIPT_REGISTERS: usize = 4;

/// Nombre maximal d'instructions exécutées par un script en un tick. Choix local
/// provisoire, non invariant.
pub const SCRIPT_MAX_STEPS_PER_TICK: usize = 16;

/// Instruction décodée. Donnée pure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Termine le script.
    Halt,
    /// Émet `value` sur le canal `channel`.
    Emit { channel: u16, value: i32 },
    /// Suspend le script pour `ticks` ticks (au moins 1).
    Wait { ticks: u32 },
    /// Affecte `value` au registre `register`.
    Set { register: u8, value: i32 },
    /// Ajoute `value` au registre `register`.
    Add { register: u8, value: i32 },
    /// Saute à l'instruction d'indice `target`.
    Jump { target: u16 },
    /// Saute à `target` si le registre `register` est non nul.
    JumpIfNonZero { register: u8, target: u16 },
}

/// Script : vue empruntée sur les octets d'un artefact `script.bin` conforme.
///
/// Aucune allocation, aucun `unsafe` : les accès décodent à la demande par
/// `from_le_bytes`, et rendent `None` hors bornes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Script<'a> {
    count: u16,
    /// Enregistrements : exactement `8·count` octets, tous validés par `read`.
    records: &'a [u8],
}

// Cohérence des constantes de layout, vérifiée à la compilation.
const _: () = {
    assert!(SCRIPT_FIXED_PAYLOAD_LEN == size_of::<u16>());
    assert!(SCRIPT_INSTRUCTION_LEN == 2 * size_of::<u8>() + size_of::<u16>() + size_of::<u32>());
};

/// Décode un enregistrement de `SCRIPT_INSTRUCTION_LEN` octets dans un script de `count`
/// instructions ; `None` si l'enregistrement n'est pas conforme (opcode inconnu, registre
/// ou cible hors bornes, attente de zéro tick). Les champs ignorés par l'opcode ne sont
/// pas lus.
fn decode_record(record: &[u8], count: u16) -> Option<Instruction> {
    let &[opcode, register, a0, a1, b0, b1, b2, b3] = record else {
        return None;
    };
    let arg16 = u16::from_le_bytes([a0, a1]);
    let arg32 = [b0, b1, b2, b3];
    let register_in_bounds = usize::from(register) < SCRIPT_REGISTERS;
    match opcode {
        0 => Some(Instruction::Halt),
        1 => Some(Instruction::Emit {
            channel: arg16,
            value: i32::from_le_bytes(arg32),
        }),
        2 => match u32::from_le_bytes(arg32) {
            0 => None,
            ticks => Some(Instruction::Wait { ticks }),
        },
        3 if register_in_bounds => Some(Instruction::Set {
            register,
            value: i32::from_le_bytes(arg32),
        }),
        4 if register_in_bounds => Some(Instruction::Add {
            register,
            value: i32::from_le_bytes(arg32),
        }),
        5 if arg16 < count => Some(Instruction::Jump { target: arg16 }),
        6 if register_in_bounds && arg16 < count => Some(Instruction::JumpIfNonZero {
            register,
            target: arg16,
        }),
        _ => None,
    }
}

impl<'a> Script<'a> {
    /// Décode un artefact complet à partir de ses octets.
    ///
    /// Ordre des contrôles : préfixe complet, identité, version, longueur déclarée
    /// inférieure à la partie fixe (`PayloadLength`), octets restants inférieurs ou
    /// supérieurs à la longueur déclarée (`Truncated`, `TrailingBytes`), longueur
    /// déduite de `n` (en `u64`) différente de la longueur déclarée (`PayloadLength`),
    /// puis chaque enregistrement dans l'ordre (`Instruction`).
    pub fn read(bytes: &'a [u8]) -> Result<Self, ReadError> {
        let prefix = read_prefix(bytes, SCRIPT_FORMAT_ID, SCRIPT_FORMAT_VERSION)?;
        let declared = u64::from(prefix.declared_len);
        if declared < SCRIPT_FIXED_PAYLOAD_LEN as u64 {
            return Err(ReadError::PayloadLength);
        }
        let available = u64::try_from(prefix.rest.len()).unwrap_or(u64::MAX);
        if available < declared {
            return Err(ReadError::Truncated);
        }
        if available > declared {
            return Err(ReadError::TrailingBytes);
        }
        let Some((fixed, records)) = prefix.rest.split_at_checked(SCRIPT_FIXED_PAYLOAD_LEN) else {
            return Err(ReadError::Truncated);
        };
        let Ok(&[n0, n1]) = <&[u8; SCRIPT_FIXED_PAYLOAD_LEN]>::try_from(fixed) else {
            return Err(ReadError::Truncated);
        };
        let count = u16::from_le_bytes([n0, n1]);
        let deduced =
            SCRIPT_FIXED_PAYLOAD_LEN as u64 + SCRIPT_INSTRUCTION_LEN as u64 * u64::from(count);
        if deduced != declared {
            return Err(ReadError::PayloadLength);
        }
        for record in records.chunks_exact(SCRIPT_INSTRUCTION_LEN) {
            if decode_record(record, count).is_none() {
                return Err(ReadError::Instruction);
            }
        }
        Ok(Self { count, records })
    }

    /// Nombre d'instructions.
    pub fn len(&self) -> u16 {
        self.count
    }

    /// Vrai si le script ne compte aucune instruction.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Instruction d'indice `index`, ou `None` hors du script. Ne lit jamais au-delà de la
    /// charge utile validée.
    pub fn instruction(&self, index: u16) -> Option<Instruction> {
        if index >= self.count {
            return None;
        }
        let offset = usize::from(index) * SCRIPT_INSTRUCTION_LEN;
        let record = self
            .records
            .get(offset..offset.checked_add(SCRIPT_INSTRUCTION_LEN)?)?;
        decode_record(record, self.count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{Asset, Format};
    use crate::load::{LoadError, load};
    use crate::manifest::SlotIndex;

    /// Scénario de référence écrit à la main (68 octets) : identité `VSCR`, version 1,
    /// charge utile 58 (`2 + 8·7`), 7 instructions.
    const VALID: [u8; 68] = [
        0x56, 0x53, 0x43, 0x52, // identité
        0x01, 0x00, // version
        0x3A, 0x00, 0x00, 0x00, // longueur de charge utile 58
        0x07, 0x00, // n = 7
        0x03, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, // 0 : Set r0, 3
        0x01, 0x00, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, // 1 : Emit canal 1, valeur 10
        0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, // 2 : Wait 3
        0x04, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, // 3 : Add r0, -1
        0x06, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // 4 : JumpIfNonZero r0, cible 1
        0x01, 0x00, 0x02, 0x00, 0x63, 0x00, 0x00, 0x00, // 5 : Emit canal 2, valeur 99
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 6 : Halt
    ];

    /// Script sans instruction : charge utile de 2 octets.
    const EMPTY: [u8; 12] = [
        0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // préfixe, charge 2
        0x00, 0x00, // n = 0
    ];

    /// Enregistrement littéral : opcode, registre, `arg16`, `arg32`.
    fn rec(opcode: u8, register: u8, arg16: u16, arg32: u32) -> [u8; 8] {
        let a = arg16.to_le_bytes();
        let b = arg32.to_le_bytes();
        [opcode, register, a[0], a[1], b[0], b[1], b[2], b[3]]
    }

    /// Assemble un artefact conforme de `records.len()` instructions (au plus 8) dans `buf`.
    fn build<'b>(buf: &'b mut [u8; 80], records: &[[u8; 8]]) -> &'b [u8] {
        let payload = 2 + 8 * records.len();
        buf[..4].copy_from_slice(b"VSCR");
        buf[4..6].copy_from_slice(&[0x01, 0x00]);
        buf[6..10].copy_from_slice(&(payload as u32).to_le_bytes());
        buf[10..12].copy_from_slice(&(records.len() as u16).to_le_bytes());
        for (i, record) in records.iter().enumerate() {
            buf[12 + 8 * i..20 + 8 * i].copy_from_slice(record);
        }
        &buf[..10 + payload]
    }

    #[test]
    fn scr_r1_reads_the_literal_artifact() {
        let script = Script::read(&VALID).expect("artefact conforme");
        assert_eq!(script.len(), 7);
        assert!(!script.is_empty());
        let expected = [
            Instruction::Set {
                register: 0,
                value: 3,
            },
            Instruction::Emit {
                channel: 1,
                value: 10,
            },
            Instruction::Wait { ticks: 3 },
            Instruction::Add {
                register: 0,
                value: -1,
            },
            Instruction::JumpIfNonZero {
                register: 0,
                target: 1,
            },
            Instruction::Emit {
                channel: 2,
                value: 99,
            },
            Instruction::Halt,
        ];
        for (index, instruction) in expected.iter().enumerate() {
            assert_eq!(
                script.instruction(index as u16),
                Some(*instruction),
                "instruction {index}"
            );
        }
    }

    #[test]
    fn scr_r2_every_truncation_is_rejected() {
        for len in 0..VALID.len() {
            assert_eq!(
                Script::read(&VALID[..len]),
                Err(ReadError::Truncated),
                "préfixe de {len} octets"
            );
        }
    }

    #[test]
    fn scr_r3_altered_format_identity_is_rejected() {
        for index in 0..4 {
            let mut bytes = VALID;
            bytes[index] ^= 0xFF;
            assert_eq!(
                Script::read(&bytes),
                Err(ReadError::FormatIdentity),
                "octet d'identité {index} altéré"
            );
        }
        // Le texte d'un `script.toml` n'est pas un artefact (I1).
        let source = b"[[instruction]]\nop = \"halt\"\n";
        assert!(source.len() >= 12);
        assert_eq!(Script::read(source), Err(ReadError::FormatIdentity));
    }

    #[test]
    fn scr_r4_unknown_version_is_rejected() {
        for version in [[0x00, 0x00], [0x02, 0x00], [0x00, 0x01], [0xFF, 0xFF]] {
            let mut bytes = VALID;
            bytes[4..6].copy_from_slice(&version);
            assert_eq!(
                Script::read(&bytes),
                Err(ReadError::Version),
                "version {version:?}"
            );
        }
    }

    #[test]
    fn scr_r5_declared_length_must_match_the_remaining_bytes_and_the_count() {
        let with_declared = |declared: [u8; 4]| {
            let mut bytes = VALID;
            bytes[6..10].copy_from_slice(&declared);
            bytes
        };
        // Déclarée inférieure à la partie fixe.
        for declared in [[0x00, 0x00, 0x00, 0x00], [0x01, 0x00, 0x00, 0x00]] {
            assert_eq!(
                Script::read(&with_declared(declared)),
                Err(ReadError::PayloadLength),
                "déclarée {declared:?}"
            );
        }
        // Déclarée supérieure aux octets restants.
        assert_eq!(
            Script::read(&with_declared([0x3B, 0x00, 0x00, 0x00])),
            Err(ReadError::Truncated)
        );
        // Déclarée inférieure aux octets restants, mais au moins la partie fixe.
        for declared in [[0x02, 0x00, 0x00, 0x00], [0x39, 0x00, 0x00, 0x00]] {
            assert_eq!(
                Script::read(&with_declared(declared)),
                Err(ReadError::TrailingBytes),
                "déclarée {declared:?}"
            );
        }
        // Surplus d'un octet, déclarée inchangée.
        let mut surplus = [0u8; 69];
        surplus[..68].copy_from_slice(&VALID);
        assert_eq!(Script::read(&surplus), Err(ReadError::TrailingBytes));
        // Déclarée égale aux restants, mais différente de `2 + 8·n`.
        for n in [[0x06, 0x00], [0x08, 0x00]] {
            let mut bytes = VALID;
            bytes[10..12].copy_from_slice(&n);
            assert_eq!(
                Script::read(&bytes),
                Err(ReadError::PayloadLength),
                "n = {n:?}"
            );
        }
        let mut longer = [0u8; 69];
        longer[..68].copy_from_slice(&VALID);
        longer[6] = 0x3B;
        assert_eq!(Script::read(&longer), Err(ReadError::PayloadLength));
        // Une longueur incohérente prime sur un enregistrement invalide (ordre des contrôles).
        let mut both = VALID;
        both[10..12].copy_from_slice(&[0x08, 0x00]);
        both[12] = 0x07; // opcode inconnu dans l'enregistrement 0
        assert_eq!(Script::read(&both), Err(ReadError::PayloadLength));
    }

    #[test]
    fn scr_r6_zero_instruction_is_conform_and_a_zero_length_payload_is_not() {
        let script = Script::read(&EMPTY).expect("zéro instruction conforme");
        assert_eq!(script.len(), 0);
        assert!(script.is_empty());
        assert_eq!(script.instruction(0), None);

        let zero_payload = [0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(Script::read(&zero_payload), Err(ReadError::PayloadLength));
    }

    #[test]
    fn scr_r7_a_hostile_count_is_rejected_without_overflow_or_panic() {
        // n = 0xFFFF, charge utile de 2 octets : longueur déduite 524 282 ≠ 2.
        let huge = [
            0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0xFF, 0xFF,
        ];
        assert_eq!(Script::read(&huge), Err(ReadError::PayloadLength));
        // n = 0xFFFF, charge utile de 4 octets.
        let huge_4 = [
            0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
        ];
        assert_eq!(Script::read(&huge_4), Err(ReadError::PayloadLength));
        // n = 0x2000 : `8·n` sur 16 bits vaudrait 0 et ferait coïncider `2 + 8·n` avec 2.
        let wrap = [
            0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x20,
        ];
        assert_eq!(Script::read(&wrap), Err(ReadError::PayloadLength));
        // Déclarée u32::MAX avec peu d'octets.
        let max_declared = [
            0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        ];
        assert_eq!(Script::read(&max_declared), Err(ReadError::Truncated));
    }

    #[test]
    fn scr_r8_invalid_records_are_rejected_and_ignored_fields_are_not_checked() {
        let halt = rec(0, 0, 0, 0);
        // Un enregistrement invalide par cas, à des positions distinctes (n = 3).
        let invalid: [([u8; 8], usize); 11] = [
            (rec(7, 0, 0, 0), 0),
            (rec(255, 0, 0, 0), 1),
            (rec(3, 4, 0, 0), 2),
            (rec(4, 4, 0, 0), 0),
            (rec(6, 4, 0, 0), 1),
            (rec(3, 255, 0, 0), 1),
            (rec(5, 0, 3, 0), 2),
            (rec(6, 0, 3, 0), 0),
            (rec(6, 0, 0xFFFF, 0), 2),
            (rec(2, 0, 0, 0), 1),
            (rec(2, 0xFF, 0xFFFF, 0), 0),
        ];
        for (record, position) in invalid {
            let mut records = [halt; 3];
            records[position] = record;
            let mut buf = [0u8; 80];
            assert_eq!(
                Script::read(build(&mut buf, &records)),
                Err(ReadError::Instruction),
                "enregistrement {record:?} en position {position}"
            );
        }

        // Champs ignorés non nuls : acceptés, et instruction égale à celle aux champs nuls.
        // (opcode, registre utilisé, arg16 utilisé, arg32 utilisé ; champs ignorés).
        let ignoring: [(u8, u8, u16, u32, [bool; 3]); 7] = [
            (0, 0, 0, 0, [true, true, true]),
            (1, 0, 5, 7, [true, false, false]),
            (2, 0, 0, 3, [true, true, false]),
            (3, 2, 0, 9, [false, true, false]),
            (4, 3, 0, 9, [false, true, false]),
            (5, 0, 0, 0, [true, false, true]),
            (6, 1, 0, 0, [false, false, true]),
        ];
        for (opcode, register, arg16, arg32, ignored) in ignoring {
            let mut buf = [0u8; 80];
            let reference = Script::read(build(&mut buf, &[rec(opcode, register, arg16, arg32)]))
                .expect("référence conforme")
                .instruction(0)
                .expect("une instruction");
            for mask in 1..8u8 {
                let mut r = register;
                let mut a = arg16;
                let mut b = arg32;
                if ignored[0] && mask & 1 != 0 {
                    r = 0xFF;
                }
                if ignored[1] && mask & 2 != 0 {
                    a = 0xFFFF;
                }
                if ignored[2] && mask & 4 != 0 {
                    b = 0xFFFF_FFFF;
                }
                let mut buf = [0u8; 80];
                let script = Script::read(build(&mut buf, &[rec(opcode, r, a, b)]))
                    .unwrap_or_else(|e| panic!("opcode {opcode} masque {mask} rejeté : {e:?}"));
                assert_eq!(
                    script.instruction(0),
                    Some(reference),
                    "opcode {opcode} masque {mask}"
                );
            }
        }

        // Bornes valides : cible n − 1, registre SCRIPT_REGISTERS − 1, Wait maximal.
        let mut buf = [0u8; 80];
        let script = Script::read(build(
            &mut buf,
            &[
                rec(5, 0, 2, 0),
                rec(6, 3, 2, 0),
                rec(2, 0, 0, u32::MAX),
                rec(3, 3, 0, 0x8000_0000),
            ],
        ))
        .expect("bornes valides");
        assert_eq!(script.instruction(0), Some(Instruction::Jump { target: 2 }));
        assert_eq!(
            script.instruction(1),
            Some(Instruction::JumpIfNonZero {
                register: 3,
                target: 2
            })
        );
        assert_eq!(
            script.instruction(2),
            Some(Instruction::Wait { ticks: u32::MAX })
        );
        assert_eq!(
            script.instruction(3),
            Some(Instruction::Set {
                register: 3,
                value: i32::MIN
            })
        );
    }

    #[test]
    fn scr_r9_out_of_range_indices_yield_none() {
        let script = Script::read(&VALID).expect("artefact conforme");
        assert_eq!(script.instruction(6), Some(Instruction::Halt));
        assert_eq!(script.instruction(7), None);
        assert_eq!(script.instruction(u16::MAX), None);
        let empty = Script::read(&EMPTY).expect("zéro instruction");
        assert_eq!(empty.instruction(0), None);
        assert_eq!(empty.instruction(u16::MAX), None);
    }

    #[test]
    fn scr_r10_the_maximal_cardinality_is_accepted_and_one_byte_less_is_truncated() {
        // Natif : cardinalité maximale en binaire ; Miri : réduite (la boucle est coûteuse).
        let count: usize = if cfg!(miri) { 512 } else { 65_535 };
        let mut bytes = [0u8; 12 + 8 * 65_535];
        let len = 12 + 8 * count;
        let payload = (2 + 8 * count) as u32;
        bytes[..4].copy_from_slice(b"VSCR");
        bytes[4..6].copy_from_slice(&[0x01, 0x00]);
        bytes[6..10].copy_from_slice(&payload.to_le_bytes());
        bytes[10..12].copy_from_slice(&(count as u16).to_le_bytes());
        for i in 0..count {
            // Remplissage : Wait 1, registres 0, cibles 0.
            bytes[12 + 8 * i..20 + 8 * i].copy_from_slice(&[0x02, 0, 0, 0, 0x01, 0, 0, 0]);
        }
        let middle = count / 2;
        bytes[12..20].copy_from_slice(&[0x01, 0x00, 0xCD, 0xAB, 0x06, 0x05, 0x04, 0x03]);
        bytes[12 + 8 * middle..20 + 8 * middle]
            .copy_from_slice(&[0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        bytes[12 + 8 * (count - 1)..20 + 8 * (count - 1)].copy_from_slice(&[0; 8]);

        let script = Script::read(&bytes[..len]).expect("cardinalité maximale");
        assert_eq!(usize::from(script.len()), count);
        assert_eq!(
            script.instruction(0),
            Some(Instruction::Emit {
                channel: 0xABCD,
                value: 0x0304_0506
            })
        );
        assert_eq!(
            script.instruction(middle as u16),
            Some(Instruction::Jump { target: 0 })
        );
        assert_eq!(
            script.instruction((count - 1) as u16),
            Some(Instruction::Halt)
        );
        assert_eq!(script.instruction(count as u16), None);
        assert_eq!(Script::read(&bytes[..len - 1]), Err(ReadError::Truncated));
    }

    #[test]
    fn scr_k1_the_provisional_constants_keep_their_planned_values() {
        const MSG: &str = "valeur provisoire du plan § 6 : la modifier est une décision de plan";
        assert_eq!(SCRIPT_REGISTERS, 4, "{MSG}");
        assert_eq!(SCRIPT_MAX_STEPS_PER_TICK, 16, "{MSG}");
        assert_eq!(SCRIPT_INSTRUCTION_LEN, 8, "{MSG}");
        assert_eq!(SCRIPT_FIXED_PAYLOAD_LEN, 2, "{MSG}");
    }

    /// Manifeste d'une seule entrée : id 0x1122334455667788, artefact 0, slot 1.
    const SINGLE: [u8; 26] = [
        0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, // en-tête, n = 1
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id
        0x00, 0x00, 0x00, 0x00, // artifact_index 0
        0x01, 0x00, 0x00, 0x00, // slot_index 1
    ];

    const SPRITE_BYTES: [u8; 14] = [
        0x56, 0x53, 0x50, 0x52, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x20, 0x00, 0x10, 0x00,
    ];
    const TILESET_BYTES: [u8; 18] = [
        0x56, 0x54, 0x4C, 0x53, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x08, 0x00, 0x04,
        0x00, 0x0C, 0x00,
    ];
    /// Préfixe seul : le format du slot est rejeté au niveau 2, avant tout décodage.
    const MAP_PREFIX: [u8; 10] = [0x56, 0x4D, 0x41, 0x50, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
    const FONT_BYTES: [u8; 14] = [
        0x56, 0x46, 0x4E, 0x54, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0x0A,
    ];

    #[test]
    fn scr_l1_every_off_diagonal_pair_of_the_five_formats_is_a_slot_format_error() {
        let formats = [
            Format::Sprite,
            Format::Tileset,
            Format::Map,
            Format::Font,
            Format::Script,
        ];
        let artifacts: [&[u8]; 5] = [
            &SPRITE_BYTES,
            &TILESET_BYTES,
            &MAP_PREFIX,
            &FONT_BYTES,
            &VALID,
        ];
        for (a, artifact) in artifacts.iter().enumerate() {
            for (s, slot_format) in formats.iter().enumerate() {
                if a == s {
                    continue;
                }
                let mut calls = 0;
                let result = load(&SINGLE, &[artifact], &[*slot_format; 2], |_, _| calls += 1);
                assert_eq!(
                    result,
                    Err(LoadError::SlotFormat {
                        slot: SlotIndex::new(1)
                    }),
                    "artefact {:?} lié à un slot {:?}",
                    formats[a],
                    slot_format
                );
                assert_eq!(calls, 0);
            }
        }
    }

    #[test]
    fn scr_l2_a_batch_of_two_scripts_is_deposited_in_each_slot() {
        // Artefact 0 → slot 1 ; artefact 1 → slot 0 : les index diffèrent.
        const MANIFEST: [u8; 42] = [
            0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // n = 2
            0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id 0
            0x00, 0x00, 0x00, 0x00, // artifact_index 0
            0x01, 0x00, 0x00, 0x00, // slot_index 1
            0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id 1
            0x01, 0x00, 0x00, 0x00, // artifact_index 1
            0x00, 0x00, 0x00, 0x00, // slot_index 0
        ];
        let mut deposits: [Option<Script>; 2] = [None, None];
        let mut calls = 0;
        let result = load(
            &MANIFEST,
            &[&VALID, &EMPTY],
            &[Format::Script; 2],
            |slot, asset| {
                let Asset::Script(script) = asset else {
                    panic!("variante inattendue : un script était attendu");
                };
                let cell = &mut deposits[slot.get() as usize];
                assert!(cell.is_none(), "slot {} déposé deux fois", slot.get());
                *cell = Some(script);
                calls += 1;
            },
        );
        assert_eq!(result, Ok(()));
        assert_eq!(calls, 2);
        // Artefact 0 (VALID) → slot 1 ; artefact 1 (EMPTY) → slot 0.
        assert_eq!(deposits[1], Some(Script::read(&VALID).unwrap()));
        assert_eq!(deposits[0], Some(Script::read(&EMPTY).unwrap()));
        assert_eq!(deposits[1].map(|s| s.len()), Some(7));
        assert_eq!(deposits[0].map(|s| s.len()), Some(0));
    }

    #[test]
    fn scr_l3_an_invalid_instruction_after_valid_artifacts_aborts_before_any_deposit() {
        // Trois entrées : artefacts 0, 1, 2 vers les slots 0, 1, 2.
        const MANIFEST: [u8; 58] = [
            0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, // n = 3
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // id 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // artefact 0, slot 0
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // id 2
            0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // artefact 1, slot 1
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // id 3
            0x02, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, // artefact 2, slot 2
        ];
        // Wait de zéro tick, puis registre hors bornes.
        let wait_zero = [
            0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00, //
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let bad_register = [
            0x56, 0x53, 0x43, 0x52, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00, //
            0x03, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        for invalid in [&wait_zero[..], &bad_register[..]] {
            let mut calls = 0;
            let result = load(
                &MANIFEST,
                &[&VALID, &EMPTY, invalid],
                &[Format::Script; 3],
                |_, _| calls += 1,
            );
            let Err(LoadError::Artifact { index, error }) = result else {
                panic!("attendu : erreur d'artefact, obtenu {result:?}");
            };
            assert_eq!(index.get(), 2);
            assert_eq!(error, ReadError::Instruction);
            assert_eq!(calls, 0);
        }
    }
}
