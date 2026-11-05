// crates/extensions/script/src/step.rs

//! `step` : unique transition d'un `ScriptState`, fonction pure.
//!
//! `step` ne lit que ses trois paramètres (script, état, sink) et n'écrit que
//! l'état et le sink : aucun état externe, aucune allocation.

use verbum_assets::{Instruction, SCRIPT_MAX_STEPS_PER_TICK, Script};

use crate::emissions::{Emission, Emissions, Full};
use crate::state::{Fault, ScriptState, Status};

/// Exécute un tick de l'instance `state` du script `script`.
///
/// # Errors
///
/// Rend `Err(Full)` si une instruction `Emit` rencontre un sink plein ; le
/// compteur `pc` n'est alors pas avancé et l'état reste `Running`.
pub fn step<'a, const CAP: usize>(
    script: Option<&Script<'a>>,
    state: &mut ScriptState,
    sink: &mut Emissions<CAP>,
) -> Result<(), Full> {
    match state.status() {
        Status::Halted | Status::Faulted(_) => return Ok(()),
        Status::Waiting => {
            if !state.countdown() {
                return Ok(());
            }
        }
        Status::Running => {}
    }

    let Some(script) = script else {
        state.fault(Fault::NoScript);
        return Ok(());
    };

    let mut executed = 0usize;
    loop {
        let Some(instruction) = script.instruction(state.pc()) else {
            // Fin de programme implicite : `pc` reste à `n`.
            state.halt();
            return Ok(());
        };
        if executed == SCRIPT_MAX_STEPS_PER_TICK {
            state.fault(Fault::Budget);
            return Ok(());
        }
        executed += 1;

        match instruction {
            Instruction::Halt => {
                state.halt();
                return Ok(());
            }
            Instruction::Emit { channel, value } => {
                sink.push(Emission { channel, value })?;
                state.advance();
            }
            Instruction::Wait { ticks } => {
                state.advance();
                state.wait_for(ticks);
                return Ok(());
            }
            Instruction::Set { register, value } => {
                state.set_register(register, value);
                state.advance();
            }
            Instruction::Add { register, value } => {
                state.set_register(register, state.register(register).wrapping_add(value));
                state.advance();
            }
            Instruction::Jump { target } => state.jump_to(target),
            Instruction::JumpIfNonZero { register, target } => {
                if state.register(register) == 0 {
                    state.advance();
                } else {
                    state.jump_to(target);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ENCODED_LEN;
    use verbum_assets::{SCRIPT_REGISTERS, SlotIndex};

    // --- Fabrique d'artefacts de test (sans `Vec`) -------------------------

    fn rec(opcode: u8, register: u8, arg16: u16, arg32: [u8; 4]) -> [u8; 8] {
        let channel = arg16.to_le_bytes();
        [
            opcode, register, channel[0], channel[1], arg32[0], arg32[1], arg32[2], arg32[3],
        ]
    }
    fn halt() -> [u8; 8] {
        rec(0, 0, 0, [0; 4])
    }
    fn emit(channel: u16, value: i32) -> [u8; 8] {
        rec(1, 0, channel, value.to_le_bytes())
    }
    fn wait(ticks: u32) -> [u8; 8] {
        rec(2, 0, 0, ticks.to_le_bytes())
    }
    fn set(register: u8, value: i32) -> [u8; 8] {
        rec(3, register, 0, value.to_le_bytes())
    }
    fn add(register: u8, value: i32) -> [u8; 8] {
        rec(4, register, 0, value.to_le_bytes())
    }
    fn jump(target: u16) -> [u8; 8] {
        rec(5, 0, target, [0; 4])
    }
    fn jnz(register: u8, target: u16) -> [u8; 8] {
        rec(6, register, target, [0; 4])
    }

    /// Artefact `script.bin` de `L` octets contenant `records`.
    fn build<const L: usize>(records: &[[u8; 8]]) -> [u8; L] {
        assert_eq!(L, 12 + 8 * records.len());
        let mut bytes = [0u8; L];
        bytes[0..4].copy_from_slice(b"VSCR");
        bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
        let payload = u32::try_from(2 + 8 * records.len()).expect("charge utile");
        bytes[6..10].copy_from_slice(&payload.to_le_bytes());
        let count = u16::try_from(records.len()).expect("nombre d'instructions");
        bytes[10..12].copy_from_slice(&count.to_le_bytes());
        for (index, record) in records.iter().enumerate() {
            bytes[12 + 8 * index..20 + 8 * index].copy_from_slice(record);
        }
        bytes
    }

    /// Scénario de référence : littéral écrit à la main, indépendant des constantes.
    const REFERENCE: [u8; 68] = [
        0x56, 0x53, 0x43, 0x52, // VSCR
        0x01, 0x00, // version 1
        0x3A, 0x00, 0x00, 0x00, // charge utile 58
        0x07, 0x00, // n = 7
        0x03, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, // 0 : Set r0, 3
        0x01, 0x00, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, // 1 : Emit canal 1, valeur 10
        0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, // 2 : Wait 3
        0x04, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, // 3 : Add r0, -1
        0x06, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // 4 : JumpIfNonZero r0, cible 1
        0x01, 0x00, 0x02, 0x00, 0x63, 0x00, 0x00, 0x00, // 5 : Emit canal 2, valeur 99
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 6 : Halt
    ];

    const BUDGET_LEN: usize = 12 + 8 * (SCRIPT_MAX_STEPS_PER_TICK + 1);
    const BOUND_LEN: usize = 12 + 8 * SCRIPT_MAX_STEPS_PER_TICK;

    fn started(slot: u32) -> ScriptState {
        ScriptState::start(SlotIndex::new(slot)).expect("slot valide")
    }

    /// Un tick avec un sink neuf de capacité 8 ; rend le sink.
    fn tick(script: Option<&Script<'_>>, state: &mut ScriptState) -> Emissions<8> {
        let mut sink = Emissions::<8>::new();
        step(script, state, &mut sink).expect("sink assez grand");
        sink
    }

    fn emission(channel: u16, value: i32) -> Emission {
        Emission { channel, value }
    }

    /// 14 premiers octets littéraux, les 12 suivants nuls (registres r1 à r3).
    fn encoded(head: [u8; 14]) -> [u8; ENCODED_LEN] {
        let mut bytes = [0u8; ENCODED_LEN];
        bytes[..14].copy_from_slice(&head);
        bytes
    }

    fn assert_valid(state: &ScriptState) {
        assert_eq!(ScriptState::from_bytes(&state.to_bytes()), Ok(*state));
        assert!(!(state.status() == Status::Waiting && state.wait() == 0));
    }

    // ST-1 : longueur et encodage de la trajectoire de référence (états obtenus
    // par exécution réelle de `step`).
    #[test]
    fn st1_encoding_of_reference_trajectory() {
        assert_eq!(ENCODED_LEN, 26);
        assert_eq!(ENCODED_LEN, 10 + 4 * SCRIPT_REGISTERS);

        let script = Script::read(&REFERENCE).expect("référence");
        let mut state = started(4);
        assert_eq!(
            state.to_bytes(),
            encoded([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        );
        for tick_number in 1u32..=12 {
            let sink = tick(Some(&script), &mut state);
            match tick_number {
                1 | 4 | 7 => assert_eq!(sink.as_slice(), &[emission(1, 10)], "tick {tick_number}"),
                10 => assert_eq!(sink.as_slice(), &[emission(2, 99)], "tick {tick_number}"),
                _ => assert!(sink.is_empty(), "tick {tick_number}"),
            }
            let expected = match tick_number {
                1 => Some([4, 0, 3, 0, 1, 0, 3, 0, 0, 0, 3, 0, 0, 0]),
                2 => Some([4, 0, 3, 0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0]),
                3 => Some([4, 0, 3, 0, 1, 0, 1, 0, 0, 0, 3, 0, 0, 0]),
                4 => Some([4, 0, 3, 0, 1, 0, 3, 0, 0, 0, 2, 0, 0, 0]),
                10..=12 => Some([4, 0, 6, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                _ => None,
            };
            if let Some(head) = expected {
                assert_eq!(state.to_bytes(), encoded(head), "fin du tick {tick_number}");
            }
        }
    }

    // ST-4 : tout état atteint par `step` est valide ; constructeur `start`.
    #[test]
    fn st4_reached_states_are_valid() {
        let reference = Script::read(&REFERENCE).expect("référence");
        let mut state = started(4);
        assert_valid(&state);
        for _ in 0..14 {
            let _ = tick(Some(&reference), &mut state);
            assert_valid(&state);
        }

        let budget_records = [add(0, 1); SCRIPT_MAX_STEPS_PER_TICK + 1];
        let budget_bytes = build::<BUDGET_LEN>(&budget_records);
        let budget = Script::read(&budget_bytes).expect("budget");
        let mut state = started(0);
        for _ in 0..3 {
            let _ = tick(Some(&budget), &mut state);
            assert_valid(&state);
        }

        let mut state = started(2);
        for _ in 0..3 {
            let _ = tick(None, &mut state);
            assert_valid(&state);
        }

        let halt_bytes = build::<20>(&[halt()]);
        let halting = Script::read(&halt_bytes).expect("halt");
        let mut state = started(1);
        for _ in 0..3 {
            let _ = tick(Some(&halting), &mut state);
            assert_valid(&state);
        }

        let end_bytes = build::<20>(&[set(0, 1)]);
        let ending = Script::read(&end_bytes).expect("fin de programme");
        let mut state = started(1);
        for _ in 0..3 {
            let _ = tick(Some(&ending), &mut state);
            assert_valid(&state);
        }

        // Constructeur `start`.
        let state = ScriptState::start(SlotIndex::new(65_535)).expect("65 535 accepté");
        assert_eq!(state.script(), 65_535);
        assert_eq!(
            ScriptState::start(SlotIndex::new(65_536)),
            Err(crate::state::SlotTooLarge)
        );
        assert_eq!(
            ScriptState::start(SlotIndex::new(u32::MAX)),
            Err(crate::state::SlotTooLarge)
        );
        let state = started(4);
        assert_eq!(state.script(), 4);
        assert_eq!(state.status(), Status::Running);
        assert_eq!(state.pc(), 0);
        assert_eq!(state.wait(), 0);
        assert_eq!(state.registers(), [0; SCRIPT_REGISTERS]);
    }

    // IN-1 : `Emit` pousse dans l'ordre et avance `pc`.
    #[test]
    fn in1_emit() {
        let bytes = build::<44>(&[emit(1, 10), emit(2, 20), emit(3, -30), wait(1)]);
        let script = Script::read(&bytes).expect("script");
        let mut state = started(0);
        let sink = tick(Some(&script), &mut state);
        assert_eq!(
            sink.as_slice(),
            &[emission(1, 10), emission(2, 20), emission(3, -30)]
        );
        assert_eq!(state.pc(), 4);
        assert_eq!(state.status(), Status::Waiting);
    }

    // IN-2 : `Set`, `Add`, arithmétique enveloppante.
    #[test]
    fn in2_set_add() {
        let bytes = build::<92>(&[
            set(0, 5),
            add(0, 7),
            add(0, -2),
            set(3, -9),
            set(1, i32::MAX),
            add(1, 1),
            set(2, i32::MIN),
            add(2, -1),
            set(3, -9),
            wait(1),
        ]);
        let script = Script::read(&bytes).expect("script");
        let mut state = started(0);
        let sink = tick(Some(&script), &mut state);
        assert!(sink.is_empty());
        assert_eq!(state.registers(), [10, i32::MIN, i32::MAX, -9]);
        assert_eq!(state.pc(), 10);
    }

    // IN-3 : `Jump` et `JumpIfNonZero`.
    #[test]
    fn in3_jumps() {
        let bytes = build::<44>(&[jump(2), set(0, 99), set(1, 7), halt()]);
        let script = Script::read(&bytes).expect("jump");
        let mut state = started(0);
        let _ = tick(Some(&script), &mut state);
        assert_eq!(state.registers(), [0, 7, 0, 0]);
        assert_eq!(state.status(), Status::Halted);
        assert_eq!(state.pc(), 3);

        for register in [0u8, 3] {
            // Pris : le registre est non nul, `Set r0, 111` est sauté.
            let taken = build::<44>(&[set(register, 1), jnz(register, 3), set(0, 111), set(1, 5)]);
            let script = Script::read(&taken).expect("jnz pris");
            let mut state = started(0);
            let _ = tick(Some(&script), &mut state);
            let mut expected = [0, 5, 0, 0];
            expected[usize::from(register)] = 1;
            assert_eq!(state.registers(), expected, "pris, registre {register}");

            // Non pris : le registre est nul, `Set r0, 111` est exécuté.
            let not_taken =
                build::<44>(&[set(register, 0), jnz(register, 3), set(0, 111), set(1, 5)]);
            let script = Script::read(&not_taken).expect("jnz non pris");
            let mut state = started(0);
            let _ = tick(Some(&script), &mut state);
            assert_eq!(
                state.registers(),
                [111, 5, 0, 0],
                "non pris, registre {register}"
            );
        }
    }

    // IN-4 : `Wait n` reprend au tick t + n exactement.
    #[test]
    fn in4_wait() {
        for n in 1u32..=3 {
            let bytes = build::<36>(&[wait(n), set(0, 42), halt()]);
            let script = Script::read(&bytes).expect("wait");
            let mut state = started(0);

            // Tick t : `Wait` termine le tour, `Set` n'est pas exécuté.
            let _ = tick(Some(&script), &mut state);
            assert_eq!(state.status(), Status::Waiting, "n = {n}");
            assert_eq!(state.wait(), n);
            assert_eq!(state.pc(), 1);
            assert_eq!(state.registers()[0], 0);

            // Ticks t + 1 .. t + n - 1 : attente, `wait` décroît.
            for k in 1..n {
                let _ = tick(Some(&script), &mut state);
                assert_eq!(state.status(), Status::Waiting, "n = {n}, k = {k}");
                assert_eq!(state.wait(), n - k, "n = {n}, k = {k}");
                assert_eq!(state.registers()[0], 0, "n = {n}, k = {k}");
            }

            // Tick t + n : reprise, l'instruction suivante est exécutée.
            let _ = tick(Some(&script), &mut state);
            assert_eq!(state.status(), Status::Halted, "n = {n}");
            assert_eq!(state.wait(), 0);
            assert_eq!(state.registers()[0], 42);
            assert_eq!(state.pc(), 2);
        }
    }

    // IN-5 : `Halt`, fin de programme, inertie des états terminaux.
    #[test]
    fn in5_terminal_states() {
        // Halt : pc reste sur le Halt.
        let bytes = build::<28>(&[set(0, 5), halt()]);
        let script = Script::read(&bytes).expect("halt");
        let mut state = started(0);
        let _ = tick(Some(&script), &mut state);
        assert_eq!(state.status(), Status::Halted);
        assert_eq!(state.pc(), 1);
        assert_eq!(state.registers()[0], 5);
        let frozen = state.to_bytes();
        for _ in 0..3 {
            let sink = tick(Some(&script), &mut state);
            assert!(sink.is_empty());
            assert_eq!(state.to_bytes(), frozen);
        }

        // Fin de programme implicite : pc = n.
        let bytes = build::<20>(&[set(0, 1)]);
        let ending = Script::read(&bytes).expect("fin");
        let mut state = started(0);
        let _ = tick(Some(&ending), &mut state);
        assert_eq!(state.status(), Status::Halted);
        assert_eq!(state.pc(), 1);
        assert_eq!(state.registers()[0], 1);

        // Faulted(NoScript) puis Faulted(Budget) atteints par `step`, puis figés.
        let mut state = started(2);
        let _ = tick(None, &mut state);
        assert_eq!(state.status(), Status::Faulted(Fault::NoScript));
        let frozen = state.to_bytes();
        let _ = tick(None, &mut state);
        let _ = tick(Some(&script), &mut state);
        assert_eq!(state.to_bytes(), frozen);

        let records = [add(0, 1); SCRIPT_MAX_STEPS_PER_TICK + 1];
        let budget_bytes = build::<BUDGET_LEN>(&records);
        let budget = Script::read(&budget_bytes).expect("budget");
        let mut state = started(0);
        let _ = tick(Some(&budget), &mut state);
        assert_eq!(state.status(), Status::Faulted(Fault::Budget));
        let frozen = state.to_bytes();
        for _ in 0..3 {
            let _ = tick(Some(&budget), &mut state);
            assert_eq!(state.to_bytes(), frozen);
        }

        // États terminaux construits par `from_bytes`, `pc` sur l'instruction
        // `Set` non terminale (pc = 0) : ils restent inertes.
        let bytes = build::<28>(&[set(0, 5), halt()]);
        let script = Script::read(&bytes).expect("Set r0, 5 ; Halt");
        for (status, fault) in [(2u8, 0u8), (3, 1), (3, 2)] {
            let mut raw = [0u8; ENCODED_LEN];
            raw[0..2].copy_from_slice(&4u16.to_le_bytes());
            raw[4] = status;
            raw[5] = fault;
            raw[10..14].copy_from_slice(&1i32.to_le_bytes());
            raw[14..18].copy_from_slice(&7i32.to_le_bytes());
            raw[18..22].copy_from_slice(&(-3i32).to_le_bytes());
            raw[22..26].copy_from_slice(&9i32.to_le_bytes());
            let mut state = ScriptState::from_bytes(&raw).expect("état terminal valide");
            let input = state.to_bytes();
            let mut sink = Emissions::<8>::new();
            step(Some(&script), &mut state, &mut sink).expect("terminal");
            assert_eq!(state.to_bytes(), input, "statut {status}, faute {fault}");
            assert_eq!(state.registers()[0], 1);
            assert!(sink.is_empty());
        }
    }

    // IN-budget : budget exact, sans décalage.
    #[test]
    fn in_budget_exact() {
        let records = [add(0, 1); SCRIPT_MAX_STEPS_PER_TICK + 1];
        let bytes = build::<BUDGET_LEN>(&records);
        let script = Script::read(&bytes).expect("budget");
        let limit = i32::try_from(SCRIPT_MAX_STEPS_PER_TICK).expect("budget");
        let limit_pc = u16::try_from(SCRIPT_MAX_STEPS_PER_TICK).expect("budget");

        let mut first = started(0);
        let sink = tick(Some(&script), &mut first);
        assert_eq!(first.status(), Status::Faulted(Fault::Budget));
        assert_eq!(first.registers()[0], limit);
        assert_eq!(first.pc(), limit_pc);
        assert!(sink.is_empty());

        let mut second = started(0);
        let _ = tick(Some(&script), &mut second);
        assert_eq!(first, second);
    }

    // IN-bound : budget exactement consommé par une dernière instruction `Wait 1`.
    #[test]
    fn in_bound_wait_at_limit() {
        let mut records = [add(0, 1); SCRIPT_MAX_STEPS_PER_TICK];
        records[SCRIPT_MAX_STEPS_PER_TICK - 1] = wait(1);
        let bytes = build::<BOUND_LEN>(&records);
        let script = Script::read(&bytes).expect("limite");
        let before_wait = i32::try_from(SCRIPT_MAX_STEPS_PER_TICK - 1).expect("budget");
        let limit_pc = u16::try_from(SCRIPT_MAX_STEPS_PER_TICK).expect("budget");

        let mut state = started(0);
        let _ = tick(Some(&script), &mut state);
        assert_eq!(state.status(), Status::Waiting);
        assert_eq!(state.wait(), 1);
        assert_eq!(state.registers()[0], before_wait);
        assert_eq!(state.pc(), limit_pc);

        // Reprise normale : fin de programme implicite, aucune faute.
        let _ = tick(Some(&script), &mut state);
        assert_eq!(state.status(), Status::Halted);
        assert_eq!(state.pc(), limit_pc);
    }

    // IN-end : budget exactement consommé puis fin de programme.
    #[test]
    fn in_end_at_limit() {
        let records = [add(0, 1); SCRIPT_MAX_STEPS_PER_TICK];
        let bytes = build::<BOUND_LEN>(&records);
        let script = Script::read(&bytes).expect("limite");
        let mut state = started(0);
        let _ = tick(Some(&script), &mut state);
        assert_eq!(state.status(), Status::Halted);
        assert_eq!(
            state.registers()[0],
            i32::try_from(SCRIPT_MAX_STEPS_PER_TICK).expect("budget")
        );
    }

    // IN-miss : script absent.
    #[test]
    fn in_miss_absent_script() {
        use crate::table::ScriptTable;

        let a_bytes = build::<28>(&[set(0, 11), halt()]);
        let b_bytes = build::<28>(&[set(0, 22), halt()]);
        let a = Script::read(&a_bytes).expect("A");
        let b = Script::read(&b_bytes).expect("B");
        let mut table = ScriptTable::<6>::new();
        table.deposit(SlotIndex::new(4), a).expect("slot 4");
        table.deposit(SlotIndex::new(1), b).expect("slot 1");

        // Scripts présents : chaque instance exécute son propre script.
        let mut at4 = started(4);
        let _ = tick(table.get(at4.script()), &mut at4);
        assert_eq!(at4.registers()[0], 11);
        let mut at1 = started(1);
        let _ = tick(table.get(at1.script()), &mut at1);
        assert_eq!(at1.registers()[0], 22);

        // Slot 2 sans dépôt (slot vacant, ou occupé par un autre format), slot 6
        // égal à `CAP`, slot `u16::MAX`, et script absent direct.
        for slot in [2u32, 6, 65_535] {
            let mut state = started(slot);
            let index = state.script();
            let _ = tick(table.get(index), &mut state);
            assert_eq!(
                state.status(),
                Status::Faulted(Fault::NoScript),
                "slot {slot}"
            );
        }
        let mut state = started(4);
        let _ = tick(None, &mut state);
        assert_eq!(state.status(), Status::Faulted(Fault::NoScript));

        // Instance `Waiting` dont le script manque : la faute survient à la reprise.
        let mut raw = [0u8; ENCODED_LEN];
        raw[0..2].copy_from_slice(&2u16.to_le_bytes());
        raw[2..4].copy_from_slice(&3u16.to_le_bytes());
        raw[4] = 1;
        raw[6..10].copy_from_slice(&2u32.to_le_bytes());
        let mut waiting = ScriptState::from_bytes(&raw).expect("Waiting valide");
        let _ = tick(None, &mut waiting);
        assert_eq!(waiting.status(), Status::Waiting);
        assert_eq!(waiting.wait(), 1);
        let _ = tick(None, &mut waiting);
        assert_eq!(waiting.status(), Status::Faulted(Fault::NoScript));
    }

    // IN-full : sink plein, `pc` non avancé.
    #[test]
    fn in_full_sink() {
        let bytes = build::<28>(&[emit(1, 10), emit(2, 20)]);
        let script = Script::read(&bytes).expect("script");
        let mut state = started(0);
        let mut sink = Emissions::<1>::new();
        assert_eq!(step(Some(&script), &mut state, &mut sink), Err(Full));
        assert_eq!(state.pc(), 1);
        assert_eq!(state.status(), Status::Running);
        assert_eq!(sink.as_slice(), &[emission(1, 10)]);
    }

    // IN-det : même entrée, même sortie.
    #[test]
    fn in_det_same_input_same_output() {
        let script = Script::read(&REFERENCE).expect("référence");
        let mut a = started(4);
        let mut b = started(4);
        for _ in 0..12 {
            let sink_a = tick(Some(&script), &mut a);
            let sink_b = tick(Some(&script), &mut b);
            assert_eq!(a, b);
            assert_eq!(sink_a, sink_b);
        }
    }
}
