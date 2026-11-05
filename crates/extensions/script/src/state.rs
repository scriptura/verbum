// crates/extensions/script/src/state.rs

//! État d'exécution d'une instance de script et son encodage canonique.
//!
//! `ScriptState` est un composant par entité : valeur `Copy`, sans référence ni
//! durée de vie, champs privés. Deux constructeurs seulement (`start`,
//! `from_bytes`) et une seule transition (`step`) : tout état observable est
//! donc valide par construction (table de validité ci-dessous).
//!
//! # Encodage canonique
//!
//! L'encodage sert exclusivement à reconstruire l'état ; il est de
//! `ENCODED_LEN` octets, little-endian, champ par champ, sans remplissage ni
//! cast :
//!
//! | Octets | Champ |
//! |---|---|
//! | 0 à 1 | `script` (`u16`) |
//! | 2 à 3 | `pc` (`u16`) |
//! | 4 | statut : 0 `Running`, 1 `Waiting`, 2 `Halted`, 3 `Faulted` |
//! | 5 | faute : 0 aucune, 1 `NoScript`, 2 `Budget` |
//! | 6 à 9 | `wait` (`u32`) |
//! | 10 à `ENCODED_LEN` | registres (`i32` chacun) |
//!
//! # Table de validité
//!
//! | Statut | Faute | `wait` |
//! |---|---|---|
//! | `Running` | 0 | 0 |
//! | `Waiting` | 0 | au moins 1 |
//! | `Halted` | 0 | 0 |
//! | `Faulted(NoScript)` | 1 | 0 |
//! | `Faulted(Budget)` | 2 | 0 |
//!
//! Toute autre combinaison est refusée par `from_bytes`. `script` et `pc` ne
//! sont pas contraints par l'encodage.

use verbum_assets::{SCRIPT_REGISTERS, SlotIndex};

const STATUS_RUNNING: u8 = 0;
const STATUS_WAITING: u8 = 1;
const STATUS_HALTED: u8 = 2;
const STATUS_FAULTED: u8 = 3;
const FAULT_NONE: u8 = 0;
const FAULT_NO_SCRIPT: u8 = 1;
const FAULT_BUDGET: u8 = 2;

/// Longueur de l'encodage canonique d'un état, en octets.
pub const ENCODED_LEN: usize = 10 + 4 * SCRIPT_REGISTERS;

/// Cause d'un arrêt en faute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Le script de l'instance est absent de la table.
    NoScript,
    /// Le budget d'instructions par tick est épuisé.
    Budget,
}

/// Statut d'exécution d'une instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Prête à exécuter l'instruction désignée par `pc`.
    Running,
    /// Suspendue par `Wait` ; `wait` vaut au moins 1.
    Waiting,
    /// Terminée.
    Halted,
    /// Arrêtée en faute ; état terminal.
    Faulted(Fault),
}

/// Refus de `ScriptState::start` : l'indice de slot ne tient pas dans un `u16`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotTooLarge;

/// Refus de `ScriptState::from_bytes` : combinaison statut, faute et `wait`
/// hors de la table de validité, ou code inconnu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateError;

/// État d'exécution d'une instance de script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptState {
    script: u16,
    pc: u16,
    status: Status,
    wait: u32,
    registers: [i32; SCRIPT_REGISTERS],
}

impl ScriptState {
    /// État initial d'une instance du script du slot `slot` : `Running`,
    /// `pc = 0`, registres nuls.
    ///
    /// # Errors
    ///
    /// Rend `Err(SlotTooLarge)` si `slot` ne tient pas dans un `u16`.
    pub fn start(slot: SlotIndex) -> Result<Self, SlotTooLarge> {
        let script = u16::try_from(slot.get()).map_err(|_| SlotTooLarge)?;
        Ok(Self {
            script,
            pc: 0,
            status: Status::Running,
            wait: 0,
            registers: [0; SCRIPT_REGISTERS],
        })
    }

    /// Slot du script de l'instance.
    #[must_use]
    pub const fn script(&self) -> u16 {
        self.script
    }

    /// Indice de la prochaine instruction.
    #[must_use]
    pub const fn pc(&self) -> u16 {
        self.pc
    }

    /// Statut d'exécution.
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }

    /// Ticks d'attente restants (0 hors de `Waiting`).
    #[must_use]
    pub const fn wait(&self) -> u32 {
        self.wait
    }

    /// Registres, rendus par copie.
    #[must_use]
    pub const fn registers(&self) -> [i32; SCRIPT_REGISTERS] {
        self.registers
    }

    /// Encodage canonique de l'état.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; ENCODED_LEN] {
        let (status, fault) = match self.status {
            Status::Running => (STATUS_RUNNING, FAULT_NONE),
            Status::Waiting => (STATUS_WAITING, FAULT_NONE),
            Status::Halted => (STATUS_HALTED, FAULT_NONE),
            Status::Faulted(Fault::NoScript) => (STATUS_FAULTED, FAULT_NO_SCRIPT),
            Status::Faulted(Fault::Budget) => (STATUS_FAULTED, FAULT_BUDGET),
        };
        let mut bytes = [0u8; ENCODED_LEN];
        bytes[0..2].copy_from_slice(&self.script.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.pc.to_le_bytes());
        bytes[4] = status;
        bytes[5] = fault;
        bytes[6..10].copy_from_slice(&self.wait.to_le_bytes());
        for (index, value) in self.registers.iter().enumerate() {
            let offset = 10 + 4 * index;
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    /// Reconstruit un état depuis son encodage canonique.
    ///
    /// # Errors
    ///
    /// Rend `Err(StateError)` si la combinaison statut, faute et `wait` est
    /// hors de la table de validité.
    pub fn from_bytes(bytes: &[u8; ENCODED_LEN]) -> Result<Self, StateError> {
        let wait = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
        let status = match (bytes[4], bytes[5], wait) {
            (STATUS_RUNNING, FAULT_NONE, 0) => Status::Running,
            (STATUS_WAITING, FAULT_NONE, 1..) => Status::Waiting,
            (STATUS_HALTED, FAULT_NONE, 0) => Status::Halted,
            (STATUS_FAULTED, FAULT_NO_SCRIPT, 0) => Status::Faulted(Fault::NoScript),
            (STATUS_FAULTED, FAULT_BUDGET, 0) => Status::Faulted(Fault::Budget),
            _ => return Err(StateError),
        };
        let mut registers = [0i32; SCRIPT_REGISTERS];
        for (index, register) in registers.iter_mut().enumerate() {
            let offset = 10 + 4 * index;
            *register = i32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
        }
        Ok(Self {
            script: u16::from_le_bytes([bytes[0], bytes[1]]),
            pc: u16::from_le_bytes([bytes[2], bytes[3]]),
            status,
            wait,
            registers,
        })
    }

    // --- Transitions réservées à `step` (visibilité de crate) -------------

    /// Valeur du registre `index` ; un index hors bornes est inatteignable.
    pub(crate) fn register(&self, index: u8) -> i32 {
        match self.registers.get(usize::from(index)) {
            Some(value) => *value,
            None => unreachable!("registre {index} hors bornes : le lecteur de script les exclut"),
        }
    }

    /// Affecte le registre `index` ; un index hors bornes est inatteignable.
    pub(crate) fn set_register(&mut self, index: u8, value: i32) {
        match self.registers.get_mut(usize::from(index)) {
            Some(register) => *register = value,
            None => unreachable!("registre {index} hors bornes : le lecteur de script les exclut"),
        }
    }

    /// Instruction suivante.
    ///
    /// `step` n'appelle `advance` qu'après avoir lu une instruction à `pc`, donc
    /// avec `pc < n <= 65 535` : `pc + 1` tient dans un `u16`.
    pub(crate) fn advance(&mut self) {
        self.pc += 1;
    }

    /// Saut : `pc` prend la valeur `target`.
    pub(crate) fn jump_to(&mut self, target: u16) {
        self.pc = target;
    }

    /// Passe à `Halted` ; `pc` n'est pas modifié.
    pub(crate) fn halt(&mut self) {
        self.status = Status::Halted;
        self.wait = 0;
    }

    /// Passe à `Faulted(fault)` ; `pc` n'est pas modifié.
    pub(crate) fn fault(&mut self, fault: Fault) {
        self.status = Status::Faulted(fault);
        self.wait = 0;
    }

    /// Passe à `Waiting` pour `ticks` ticks (au moins 1, garanti par le lecteur).
    pub(crate) fn wait_for(&mut self, ticks: u32) {
        self.status = Status::Waiting;
        self.wait = ticks;
    }

    /// Décompte d'un tick d'attente ; rend vrai si l'attente est terminée
    /// (l'état est alors `Running`).
    pub(crate) fn countdown(&mut self) -> bool {
        self.wait = self.wait.saturating_sub(1);
        if self.wait == 0 {
            self.status = Status::Running;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGS: [i32; SCRIPT_REGISTERS] = [0; SCRIPT_REGISTERS];

    /// Octets d'un état de `script = 7`, `pc = 9`, registres nuls.
    fn raw(status: u8, fault: u8, wait: u32) -> [u8; ENCODED_LEN] {
        let mut bytes = [0u8; ENCODED_LEN];
        bytes[0..2].copy_from_slice(&7u16.to_le_bytes());
        bytes[2..4].copy_from_slice(&9u16.to_le_bytes());
        bytes[4] = status;
        bytes[5] = fault;
        bytes[6..10].copy_from_slice(&wait.to_le_bytes());
        bytes
    }

    fn assert_copy_static<T: Copy + 'static>() {}

    // ST-2 : table de validité appliquée par `from_bytes`.
    #[test]
    fn st2_from_bytes_validity_table() {
        let valid = [
            (0u8, 0u8, 0u32, Status::Running),
            (1, 0, 1, Status::Waiting),
            (1, 0, u32::MAX, Status::Waiting),
            (2, 0, 0, Status::Halted),
            (3, 1, 0, Status::Faulted(Fault::NoScript)),
            (3, 2, 0, Status::Faulted(Fault::Budget)),
        ];
        for (status, fault, wait, expected) in valid {
            let state = ScriptState::from_bytes(&raw(status, fault, wait))
                .expect("combinaison valide acceptée");
            assert_eq!(state.status(), expected);
            assert_eq!(state.wait(), wait);
            assert_eq!(state.script(), 7);
            assert_eq!(state.pc(), 9);
            assert_eq!(state.registers(), REGS);
        }

        let invalid = [
            // Waiting avec wait = 0.
            (1u8, 0u8, 0u32),
            // wait non nul hors de Waiting.
            (0, 0, 1),
            (2, 0, 1),
            (2, 0, u32::MAX),
            (3, 1, 1),
            (3, 2, 1),
            // faute non nulle hors de Faulted.
            (0, 1, 0),
            (0, 2, 0),
            (1, 1, 1),
            (1, 2, 1),
            (2, 1, 0),
            (2, 2, 0),
            // Faulted avec faute 0, 3 ou inconnue.
            (3, 0, 0),
            (3, 3, 0),
            (3, 255, 0),
            // faute inconnue hors de Faulted.
            (0, 255, 0),
            (1, 255, 1),
            (2, 255, 0),
            // statut inconnu.
            (4, 0, 0),
            (255, 0, 0),
            (4, 0, 1),
            (255, 1, 1),
        ];
        for (status, fault, wait) in invalid {
            assert_eq!(
                ScriptState::from_bytes(&raw(status, fault, wait)),
                Err(StateError),
                "statut {status}, faute {fault}, wait {wait} doit être refusé",
            );
        }
    }

    // ST-3 : bijection entre états valides et octets valides.
    #[test]
    fn st3_encoding_is_a_bijection() {
        assert_copy_static::<ScriptState>();

        let register_sets = [
            REGS,
            [i32::MIN; SCRIPT_REGISTERS],
            [i32::MAX; SCRIPT_REGISTERS],
            [-1; SCRIPT_REGISTERS],
        ];
        let combos = [
            (0u8, 0u8, 0u32),
            (1, 0, 1),
            (1, 0, u32::MAX),
            (2, 0, 0),
            (3, 1, 0),
            (3, 2, 0),
        ];
        let bounds = [
            (0u16, 0u16),
            (u16::MAX, u16::MAX),
            (0, u16::MAX),
            (u16::MAX, 0),
        ];

        for (status, fault, wait) in combos {
            for (script, pc) in bounds {
                for registers in register_sets {
                    let mut bytes = raw(status, fault, wait);
                    bytes[0..2].copy_from_slice(&script.to_le_bytes());
                    bytes[2..4].copy_from_slice(&pc.to_le_bytes());
                    for (index, value) in registers.iter().enumerate() {
                        let offset = 10 + 4 * index;
                        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
                    }
                    let state = ScriptState::from_bytes(&bytes).expect("octets valides");
                    assert_eq!(state.script(), script);
                    assert_eq!(state.pc(), pc);
                    assert_eq!(state.registers(), registers);
                    // to_bytes ∘ from_bytes = id
                    assert_eq!(state.to_bytes(), bytes);
                    // from_bytes ∘ to_bytes = id
                    assert_eq!(ScriptState::from_bytes(&state.to_bytes()), Ok(state));
                }
            }
        }

        let started = ScriptState::start(SlotIndex::new(4)).expect("slot 4");
        assert_eq!(ScriptState::from_bytes(&started.to_bytes()), Ok(started));
    }
}
