// crates/engine/forge/src/script.rs

//! Compilation d'un `script.toml` en artefact `script.bin` (I1, I2, I4, I9, I11).
//!
//! Source : un seul champ de premier niveau, `step`, tableau de tables. Chaque pas porte
//! `op` (`halt`, `emit`, `wait`, `set`, `add`, `jump`, `jnz`), un `label` optionnel
//! (chaîne non vide, unique) et les opérandes de son opération :
//!
//! | `op` | Opérandes |
//! |---|---|
//! | `halt` | aucun |
//! | `emit` | `channel` dans `0..=65535`, `value` dans `i32` |
//! | `wait` | `ticks` dans `1..=u32::MAX` |
//! | `set`, `add` | `register` dans `0..SCRIPT_REGISTERS`, `value` dans `i32` |
//! | `jump` | `target` (étiquette) |
//! | `jnz` | `register` dans `0..SCRIPT_REGISTERS`, `target` (étiquette) |
//!
//! Les étiquettes sont résolues en positions ; aucune chaîne n'est écrite dans la sortie.
//! Les octets ignorés par une opération sont écrits à zéro. Ces règles sont celles de la
//! Forge ; la terminaison (absence de cycle sans `Wait`) et le budget par tick
//! (`SCRIPT_MAX_STEPS_PER_TICK`) sont vérifiés par [`check_flow`], le lecteur runtime ne
//! contrôlant que la structure binaire (I2).

use std::collections::HashMap;

use verbum_assets::{
    Instruction, SCRIPT_FIXED_PAYLOAD_LEN, SCRIPT_FORMAT_ID, SCRIPT_FORMAT_VERSION,
    SCRIPT_INSTRUCTION_LEN, SCRIPT_MAX_STEPS_PER_TICK, SCRIPT_REGISTERS,
};

use crate::ForgeError;

/// Pas analysé : une instruction complète, ou un saut dont la cible reste une étiquette.
enum Step<'t> {
    Plain(Instruction),
    Jump(&'t str),
    JumpIfNonZero(u8, &'t str),
}

/// Compile une source `script.toml` en octets d'artefact.
///
/// Ordre des contrôles : syntaxe, champ inconnu de premier niveau, `step` (présence,
/// tableau), nombre de pas ([`checked_step_count`]) ; puis, pas par pas dans l'ordre :
/// table, `op` (présence, chaîne, opération connue), champ inconnu du pas, étiquette
/// (vide, dupliquée), opérandes dans l'ordre de la table ci-dessus (présence, type,
/// intervalle) ; puis résolution des cibles, enfin [`check_flow`] (cycle sans `Wait`,
/// puis budget). Aucun artefact n'est produit en cas d'erreur.
pub fn compile_script(source: &str) -> Result<Vec<u8>, ForgeError> {
    let table: toml::Table = source
        .parse()
        .map_err(|error: toml::de::Error| ForgeError::Syntax(error.to_string()))?;

    if let Some(unknown) = table.keys().find(|key| key.as_str() != "step") {
        return Err(ForgeError::UnknownField(unknown.clone()));
    }
    let steps = table.get("step").ok_or(ForgeError::MissingField("step"))?;
    let toml::Value::Array(items) = steps else {
        return Err(ForgeError::WrongType("step"));
    };
    let count = checked_step_count(items.len())?;

    let mut labels: HashMap<&str, u16> = HashMap::with_capacity(items.len());
    let mut parsed: Vec<Step<'_>> = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let toml::Value::Table(step) = item else {
            return Err(ForgeError::WrongType("step"));
        };
        let position = u16::try_from(index).map_err(|_| ForgeError::TooManySteps)?;
        parsed.push(parse_step(step, index, position, &mut labels)?);
    }

    let resolve = |name: &str| {
        labels
            .get(name)
            .copied()
            .ok_or_else(|| ForgeError::UnknownTarget(name.to_string()))
    };
    let mut instructions: Vec<Instruction> = Vec::with_capacity(parsed.len());
    for step in &parsed {
        instructions.push(match step {
            Step::Plain(instruction) => *instruction,
            Step::Jump(name) => Instruction::Jump {
                target: resolve(name)?,
            },
            Step::JumpIfNonZero(register, name) => Instruction::JumpIfNonZero {
                register: *register,
                target: resolve(name)?,
            },
        });
    }

    check_flow(&instructions)?;

    let payload = SCRIPT_FIXED_PAYLOAD_LEN + SCRIPT_INSTRUCTION_LEN * instructions.len();
    let payload_len = u32::try_from(payload).map_err(|_| ForgeError::PayloadTooLarge)?;
    let mut bytes = Vec::with_capacity(10 + payload);
    bytes.extend_from_slice(&SCRIPT_FORMAT_ID);
    bytes.extend_from_slice(&SCRIPT_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&payload_len.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    for instruction in &instructions {
        bytes.extend_from_slice(&encode_record(instruction));
    }
    debug_assert_eq!(bytes.len(), 10 + payload);
    Ok(bytes)
}

/// Contrôle du nombre de pas : au moins un, au plus 65 535 (`u16::MAX`). Règle isolée de
/// l'analyse TOML pour pouvoir être testée à coût constant.
pub(crate) fn checked_step_count(len: usize) -> Result<u16, ForgeError> {
    if len == 0 {
        return Err(ForgeError::EmptyScript);
    }
    u16::try_from(len).map_err(|_| ForgeError::TooManySteps)
}

/// Analyse un pas : `index` est sa position dans la source, `position` la même en `u16`
/// (valeur des étiquettes).
fn parse_step<'t>(
    step: &'t toml::Table,
    index: usize,
    position: u16,
    labels: &mut HashMap<&'t str, u16>,
) -> Result<Step<'t>, ForgeError> {
    let op = step.get("op").ok_or(ForgeError::MissingField("op"))?;
    let toml::Value::String(op) = op else {
        return Err(ForgeError::WrongType("op"));
    };
    let operands: &[&str] = match op.as_str() {
        "halt" => &[],
        "emit" => &["channel", "value"],
        "wait" => &["ticks"],
        "set" | "add" => &["register", "value"],
        "jump" => &["target"],
        "jnz" => &["register", "target"],
        other => return Err(ForgeError::UnknownOperation(other.to_string())),
    };
    if let Some(unknown) = step
        .keys()
        .find(|key| !matches!(key.as_str(), "op" | "label") && !operands.contains(&key.as_str()))
    {
        return Err(ForgeError::UnknownField(unknown.clone()));
    }

    if let Some(label) = step.get("label") {
        let toml::Value::String(name) = label else {
            return Err(ForgeError::WrongType("label"));
        };
        if name.is_empty() {
            return Err(ForgeError::EmptyLabel { step: index });
        }
        if labels.insert(name.as_str(), position).is_some() {
            return Err(ForgeError::DuplicateLabel(name.clone()));
        }
    }

    Ok(match op.as_str() {
        "emit" => {
            let channel = u16::try_from(integer(step, "channel")?)
                .map_err(|_| ForgeError::OutOfRange("channel"))?;
            Step::Plain(Instruction::Emit {
                channel,
                value: value(step)?,
            })
        }
        "wait" => {
            let ticks = integer(step, "ticks")?;
            if ticks == 0 {
                return Err(ForgeError::ZeroValue("ticks"));
            }
            let ticks = u32::try_from(ticks).map_err(|_| ForgeError::OutOfRange("ticks"))?;
            Step::Plain(Instruction::Wait { ticks })
        }
        "set" => {
            let register = register(step)?;
            Step::Plain(Instruction::Set {
                register,
                value: value(step)?,
            })
        }
        "add" => {
            let register = register(step)?;
            Step::Plain(Instruction::Add {
                register,
                value: value(step)?,
            })
        }
        "jump" => Step::Jump(target(step)?),
        "jnz" => {
            let register = register(step)?;
            Step::JumpIfNonZero(register, target(step)?)
        }
        _ => Step::Plain(Instruction::Halt),
    })
}

/// Extrait un champ entier (présence, type).
fn integer(table: &toml::Table, field: &'static str) -> Result<i64, ForgeError> {
    match table.get(field).ok_or(ForgeError::MissingField(field))? {
        toml::Value::Integer(value) => Ok(*value),
        _ => Err(ForgeError::WrongType(field)),
    }
}

/// Champ `register` : dans `0..SCRIPT_REGISTERS` (zéro admis).
fn register(table: &toml::Table) -> Result<u8, ForgeError> {
    match u8::try_from(integer(table, "register")?) {
        Ok(register) if usize::from(register) < SCRIPT_REGISTERS => Ok(register),
        _ => Err(ForgeError::OutOfRange("register")),
    }
}

/// Champ `value` : un `i32`.
fn value(table: &toml::Table) -> Result<i32, ForgeError> {
    i32::try_from(integer(table, "value")?).map_err(|_| ForgeError::OutOfRange("value"))
}

/// Champ `target` : une chaîne (étiquette).
fn target(table: &toml::Table) -> Result<&str, ForgeError> {
    match table
        .get("target")
        .ok_or(ForgeError::MissingField("target"))?
    {
        toml::Value::String(name) => Ok(name.as_str()),
        _ => Err(ForgeError::WrongType("target")),
    }
}

/// Encode une instruction en enregistrement de `SCRIPT_INSTRUCTION_LEN` octets : opcode,
/// registre, `arg16`, `arg32`, little-endian ; les champs ignorés sont nuls.
fn encode_record(instruction: &Instruction) -> [u8; SCRIPT_INSTRUCTION_LEN] {
    let (opcode, register, arg16, arg32): (u8, u8, u16, [u8; 4]) = match *instruction {
        Instruction::Halt => (0, 0, 0, [0; 4]),
        Instruction::Emit { channel, value } => (1, 0, channel, value.to_le_bytes()),
        Instruction::Wait { ticks } => (2, 0, 0, ticks.to_le_bytes()),
        Instruction::Set { register, value } => (3, register, 0, value.to_le_bytes()),
        Instruction::Add { register, value } => (4, register, 0, value.to_le_bytes()),
        Instruction::Jump { target } => (5, 0, target, [0; 4]),
        Instruction::JumpIfNonZero { register, target } => (6, register, target, [0; 4]),
    };
    let [a0, a1] = arg16.to_le_bytes();
    let [b0, b1, b2, b3] = arg32;
    [opcode, register, a0, a1, b0, b1, b2, b3]
}

/// Couleurs du parcours en profondeur.
const WHITE: u8 = 0;
const GREY: u8 = 1;
const BLACK: u8 = 2;

/// Analyse de flot sur les instructions résolues : terminaison et budget par tick.
///
/// Graphe sur les `n` instructions et un nœud virtuel `FIN` (indice `n`) ; successeurs :
/// `Halt` et `Wait` n'en ont aucun, `Jump` sa cible, `JumpIfNonZero` sa cible et `pc + 1`,
/// les autres `pc + 1` (`pc + 1 == n` désigne `FIN`). Une cible au-delà de `n` est ramenée
/// à `FIN` : la fonction ne panique sur aucune entrée.
///
/// 1. Tout cycle, atteignable ou non depuis l'instruction 0, est refusé
///    (`CycleWithoutYield`) : parcours en profondeur itératif (pile explicite, trois
///    couleurs) lancé depuis chaque nœud non visité, sans récursion.
/// 2. Le coût maximal de chaque nœud est calculé une fois les cycles écartés :
///    `Halt` et `Wait` coûtent 1, `FIN` coûte 0, tout autre nœud `1 +` le plus grand coût
///    de ses successeurs.
/// 3. Le premier nœud, par position croissante, dont le coût dépasse
///    `SCRIPT_MAX_STEPS_PER_TICK` est rapporté (`BudgetExceeded`). La vérification porte
///    sur tous les nœuds : elle couvre l'instruction 0, les reprises après un `Wait` et le
///    code inaccessible.
///
/// Les cycles l'emportent sur le budget : tous sont écartés avant tout compte de coût.
pub(crate) fn check_flow(instructions: &[Instruction]) -> Result<(), ForgeError> {
    let n = instructions.len();
    let successors = |pc: usize| -> [Option<usize>; 2] {
        let next = if pc + 1 >= n { n } else { pc + 1 };
        match instructions.get(pc) {
            None | Some(Instruction::Halt | Instruction::Wait { .. }) => [None, None],
            Some(Instruction::Jump { target }) => [Some(usize::from(*target).min(n)), None],
            Some(Instruction::JumpIfNonZero { target, .. }) => {
                [Some(usize::from(*target).min(n)), Some(next)]
            }
            Some(_) => [Some(next), None],
        }
    };

    let mut color = vec![WHITE; n];
    let mut cost = vec![0usize; n + 1];
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for start in 0..n {
        if color[start] != WHITE {
            continue;
        }
        color[start] = GREY;
        stack.push((start, 0));
        while let Some(top) = stack.last_mut() {
            let (node, edge) = *top;
            if edge >= 2 {
                // Fin du nœud : tous ses successeurs sont noirs.
                stack.pop();
                color[node] = BLACK;
                cost[node] = match instructions.get(node) {
                    Some(Instruction::Halt | Instruction::Wait { .. }) => 1,
                    _ => {
                        1 + successors(node)
                            .iter()
                            .flatten()
                            .map(|&next| cost[next])
                            .max()
                            .unwrap_or(0)
                    }
                };
                continue;
            }
            top.1 += 1;
            let Some(next) = successors(node)[edge] else {
                continue;
            };
            if next == n {
                continue;
            }
            match color[next] {
                WHITE => {
                    color[next] = GREY;
                    stack.push((next, 0));
                }
                GREY => return Err(ForgeError::CycleWithoutYield),
                _ => {}
            }
        }
    }

    for (entry, &node_cost) in cost[..n].iter().enumerate() {
        if node_cost > SCRIPT_MAX_STEPS_PER_TICK {
            return Err(ForgeError::BudgetExceeded { entry });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetKind, AssetSource, compile_assets};
    use verbum_assets::{Manifest, SCRIPT_MAX_STEPS_PER_TICK, SCRIPT_REGISTERS, Script, SlotIndex};

    const MAX: usize = SCRIPT_MAX_STEPS_PER_TICK;

    /// Source du scénario de référence.
    const REFERENCE: &str = "[[step]]\nop = \"set\"\nregister = 0\nvalue = 3\n\n\
        [[step]]\nlabel = \"loop\"\nop = \"emit\"\nchannel = 1\nvalue = 10\n\n\
        [[step]]\nop = \"wait\"\nticks = 3\n\n\
        [[step]]\nop = \"add\"\nregister = 0\nvalue = -1\n\n\
        [[step]]\nop = \"jnz\"\nregister = 0\ntarget = \"loop\"\n\n\
        [[step]]\nop = \"emit\"\nchannel = 2\nvalue = 99\n\n\
        [[step]]\nop = \"halt\"\n";

    /// Artefact attendu pour `REFERENCE`, écrit à la main (68 octets), indépendant des
    /// constantes du layout.
    const REFERENCE_BYTES: [u8; 68] = [
        0x56, 0x53, 0x43, 0x52, // identité
        0x01, 0x00, // version
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

    /// Second script de scène : `emit 3,7 ; wait 1 ; emit 3,8 ; halt`.
    const SCRIPT_B: &str = "[[step]]\nop = \"emit\"\nchannel = 3\nvalue = 7\n\
        [[step]]\nop = \"wait\"\nticks = 1\n\
        [[step]]\nop = \"emit\"\nchannel = 3\nvalue = 8\n\
        [[step]]\nop = \"halt\"\n";

    const ADD_STEP: &str = "[[step]]\nop = \"add\"\nregister = 0\nvalue = 1\n";
    const WAIT_STEP: &str = "[[step]]\nop = \"wait\"\nticks = 1\n";
    const HALT_STEP: &str = "[[step]]\nop = \"halt\"\n";

    /// Source TOML : chaque motif répété `count` fois, dans l'ordre.
    fn source(parts: &[(&str, usize)]) -> String {
        let mut text = String::new();
        for (step, count) in parts {
            for _ in 0..*count {
                text.push_str(step);
            }
        }
        text
    }

    fn add() -> Instruction {
        Instruction::Add {
            register: 0,
            value: 1,
        }
    }

    fn wait() -> Instruction {
        Instruction::Wait { ticks: 1 }
    }

    fn adds(count: usize) -> Vec<Instruction> {
        vec![add(); count]
    }

    fn jnz(target: usize) -> Instruction {
        Instruction::JumpIfNonZero {
            register: 0,
            target: u16::try_from(target).expect("cible"),
        }
    }

    fn jump(target: usize) -> Instruction {
        Instruction::Jump {
            target: u16::try_from(target).expect("cible"),
        }
    }

    fn halt() -> Instruction {
        Instruction::Halt
    }

    fn budget(entry: usize) -> Result<(), ForgeError> {
        Err(ForgeError::BudgetExceeded { entry })
    }

    #[test]
    fn scr_f1_the_reference_source_produces_exactly_the_literal_bytes() {
        assert_eq!(
            compile_script(REFERENCE).as_deref(),
            Ok(REFERENCE_BYTES.as_slice())
        );
        // Étiquette renommée : mêmes octets.
        let renamed = REFERENCE.replace("loop", "again");
        assert_eq!(
            compile_script(&renamed).as_deref(),
            Ok(REFERENCE_BYTES.as_slice())
        );
        // Chaque pas étiqueté, la cible désignant l'étiquette du pas 1 : mêmes octets.
        let mut labelled = String::new();
        for (index, step) in REFERENCE.split("[[step]]\n").skip(1).enumerate() {
            let step = step.replace("label = \"loop\"\n", "");
            labelled.push_str(&format!("[[step]]\nlabel = \"l{index}\"\n{step}"));
        }
        let labelled = labelled.replace("target = \"loop\"", "target = \"l1\"");
        assert_eq!(
            compile_script(&labelled).as_deref(),
            Ok(REFERENCE_BYTES.as_slice())
        );
    }

    #[test]
    fn scr_f2_forge_output_is_read_back_by_the_runtime_reader() {
        let bytes = compile_script(REFERENCE).expect("source valide");
        let script = Script::read(&bytes).expect("artefact conforme");
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
        assert_eq!(usize::from(script.len()), expected.len());
        for (index, instruction) in (0u16..).zip(expected) {
            assert_eq!(script.instruction(index), Some(instruction));
        }

        let bytes = compile_script(SCRIPT_B).expect("source valide");
        let script = Script::read(&bytes).expect("artefact conforme");
        let expected = [
            Instruction::Emit {
                channel: 3,
                value: 7,
            },
            Instruction::Wait { ticks: 1 },
            Instruction::Emit {
                channel: 3,
                value: 8,
            },
            Instruction::Halt,
        ];
        assert_eq!(usize::from(script.len()), expected.len());
        for (index, instruction) in (0u16..).zip(expected) {
            assert_eq!(script.instruction(index), Some(instruction));
        }
    }

    #[test]
    fn scr_f3_every_invalid_source_is_refused() {
        let step = |body: &str| format!("[[step]]\n{body}");
        let regs = SCRIPT_REGISTERS;
        let unknown = |name: &str| ForgeError::UnknownField(name.to_string());
        let cases: Vec<(String, ForgeError)> = vec![
            // Premier niveau, `step`.
            (String::new(), ForgeError::MissingField("step")),
            ("step = 3\n".into(), ForgeError::WrongType("step")),
            ("step = []\n".into(), ForgeError::EmptyScript),
            (
                format!("foo = 1\n{}", step("op = \"halt\"\n")),
                unknown("foo"),
            ),
            ("step = [1]\n".into(), ForgeError::WrongType("step")),
            // `op`.
            (step("label = \"a\"\n"), ForgeError::MissingField("op")),
            (step("op = 3\n"), ForgeError::WrongType("op")),
            (
                step("op = \"nop\"\n"),
                ForgeError::UnknownOperation("nop".to_string()),
            ),
            // Champ inconnu, par opération.
            (step("op = \"halt\"\nx = 1\n"), unknown("x")),
            (
                step("op = \"emit\"\nchannel = 1\nvalue = 1\nx = 1\n"),
                unknown("x"),
            ),
            (step("op = \"wait\"\nticks = 1\nx = 1\n"), unknown("x")),
            (
                step("op = \"set\"\nregister = 0\nvalue = 1\nx = 1\n"),
                unknown("x"),
            ),
            (
                step("op = \"add\"\nregister = 0\nvalue = 1\nx = 1\n"),
                unknown("x"),
            ),
            (step("op = \"jump\"\ntarget = \"a\"\nx = 1\n"), unknown("x")),
            (
                step("op = \"jnz\"\nregister = 0\ntarget = \"a\"\nx = 1\n"),
                unknown("x"),
            ),
            (
                step("op = \"wait\"\nticks = 1\nchannel = 1\n"),
                unknown("channel"),
            ),
            // Opérande manquant, par opération.
            (
                step("op = \"emit\"\nvalue = 1\n"),
                ForgeError::MissingField("channel"),
            ),
            (
                step("op = \"emit\"\nchannel = 1\n"),
                ForgeError::MissingField("value"),
            ),
            (step("op = \"wait\"\n"), ForgeError::MissingField("ticks")),
            (
                step("op = \"set\"\nvalue = 1\n"),
                ForgeError::MissingField("register"),
            ),
            (
                step("op = \"set\"\nregister = 0\n"),
                ForgeError::MissingField("value"),
            ),
            (
                step("op = \"add\"\nvalue = 1\n"),
                ForgeError::MissingField("register"),
            ),
            (
                step("op = \"add\"\nregister = 0\n"),
                ForgeError::MissingField("value"),
            ),
            (step("op = \"jump\"\n"), ForgeError::MissingField("target")),
            (
                step("op = \"jnz\"\ntarget = \"a\"\n"),
                ForgeError::MissingField("register"),
            ),
            (
                step("op = \"jnz\"\nregister = 0\n"),
                ForgeError::MissingField("target"),
            ),
            // Mauvais type, par opération.
            (
                step("op = \"emit\"\nchannel = \"1\"\nvalue = 1\n"),
                ForgeError::WrongType("channel"),
            ),
            (
                step("op = \"emit\"\nchannel = 1\nvalue = 1.5\n"),
                ForgeError::WrongType("value"),
            ),
            (
                step("op = \"wait\"\nticks = \"1\"\n"),
                ForgeError::WrongType("ticks"),
            ),
            (
                step("op = \"set\"\nregister = \"0\"\nvalue = 1\n"),
                ForgeError::WrongType("register"),
            ),
            (
                step("op = \"set\"\nregister = 0\nvalue = true\n"),
                ForgeError::WrongType("value"),
            ),
            (
                step("op = \"add\"\nregister = 1.0\nvalue = 1\n"),
                ForgeError::WrongType("register"),
            ),
            (
                step("op = \"add\"\nregister = 0\nvalue = \"x\"\n"),
                ForgeError::WrongType("value"),
            ),
            (
                step("op = \"jump\"\ntarget = 3\n"),
                ForgeError::WrongType("target"),
            ),
            (
                step("op = \"jnz\"\nregister = \"0\"\ntarget = \"a\"\n"),
                ForgeError::WrongType("register"),
            ),
            (
                step("op = \"jnz\"\nregister = 0\ntarget = 3\n"),
                ForgeError::WrongType("target"),
            ),
            // Registre : `SCRIPT_REGISTERS` et négatif.
            (
                step(&format!("op = \"set\"\nregister = {regs}\nvalue = 1\n")),
                ForgeError::OutOfRange("register"),
            ),
            (
                step("op = \"add\"\nregister = -1\nvalue = 1\n"),
                ForgeError::OutOfRange("register"),
            ),
            (
                step(&format!(
                    "op = \"jnz\"\nregister = {regs}\ntarget = \"a\"\n"
                )),
                ForgeError::OutOfRange("register"),
            ),
            // Valeur hors `i32`.
            (
                step("op = \"emit\"\nchannel = 1\nvalue = 2147483648\n"),
                ForgeError::OutOfRange("value"),
            ),
            (
                step("op = \"set\"\nregister = 0\nvalue = -2147483649\n"),
                ForgeError::OutOfRange("value"),
            ),
            // Canal : négatif et 65536.
            (
                step("op = \"emit\"\nchannel = -1\nvalue = 1\n"),
                ForgeError::OutOfRange("channel"),
            ),
            (
                step("op = \"emit\"\nchannel = 65536\nvalue = 1\n"),
                ForgeError::OutOfRange("channel"),
            ),
            // Ticks : zéro, négatif, hors `u32`.
            (
                step("op = \"wait\"\nticks = 0\n"),
                ForgeError::ZeroValue("ticks"),
            ),
            (
                step("op = \"wait\"\nticks = -1\n"),
                ForgeError::OutOfRange("ticks"),
            ),
            (
                step("op = \"wait\"\nticks = 4294967296\n"),
                ForgeError::OutOfRange("ticks"),
            ),
            // Étiquettes.
            (
                format!(
                    "{}{}",
                    step("op = \"halt\"\n"),
                    step("label = \"\"\nop = \"halt\"\n")
                ),
                ForgeError::EmptyLabel { step: 1 },
            ),
            (
                format!(
                    "{}{}",
                    step("label = \"a\"\nop = \"halt\"\n"),
                    step("label = \"a\"\nop = \"halt\"\n")
                ),
                ForgeError::DuplicateLabel("a".to_string()),
            ),
            // Cible inconnue.
            (
                step("op = \"jump\"\ntarget = \"nowhere\"\n"),
                ForgeError::UnknownTarget("nowhere".to_string()),
            ),
        ];
        for (text, expected) in cases {
            assert_eq!(
                compile_script(&text),
                Err(expected.clone()),
                "source : {text:?}"
            );
        }
        // TOML invalide.
        for text in ["[[step", "step = = 1", "[[step]]\nop = \n"] {
            assert!(
                matches!(compile_script(text), Err(ForgeError::Syntax(_))),
                "source : {text:?}"
            );
        }

        // Bornes acceptées.
        let top = regs - 1;
        for text in [
            step(&format!(
                "op = \"set\"\nregister = {top}\nvalue = 2147483647\n"
            )),
            step("op = \"set\"\nregister = 0\nvalue = -2147483648\n"),
            step("op = \"emit\"\nchannel = 0\nvalue = 0\n"),
            step("op = \"emit\"\nchannel = 65535\nvalue = 0\n"),
            step("op = \"wait\"\nticks = 1\n"),
            step("op = \"wait\"\nticks = 4294967295\n"),
            format!(
                "{}{}",
                step("op = \"jump\"\ntarget = \"end\"\n"),
                step("label = \"end\"\nop = \"halt\"\n")
            ),
        ] {
            assert!(compile_script(&text).is_ok(), "source : {text:?}");
        }
        let bytes =
            compile_script(&step("op = \"wait\"\nticks = 4294967295\n")).expect("source valide");
        let script = Script::read(&bytes).expect("artefact conforme");
        assert_eq!(
            script.instruction(0),
            Some(Instruction::Wait { ticks: u32::MAX })
        );
    }

    #[test]
    fn scr_f4_cycles_without_yield_and_the_step_budget() {
        let flow = check_flow;
        // (a) cycles sans yield refusés ; cycle contenant un `Wait` accepté.
        assert_eq!(flow(&[jump(0)]), Err(ForgeError::CycleWithoutYield));
        assert_eq!(flow(&[add(), jump(0)]), Err(ForgeError::CycleWithoutYield));
        assert_eq!(flow(&[add(), jnz(0)]), Err(ForgeError::CycleWithoutYield));
        assert_eq!(flow(&[wait(), jump(0)]), Ok(()));
        assert_eq!(flow(&[add(), wait(), jump(0)]), Ok(()));

        // (b) chemin de exactement MAX instructions accepté : depuis l'entrée 0, puis depuis
        // une entrée de reprise.
        assert_eq!(flow(&[adds(MAX - 1), vec![halt()]].concat()), Ok(()));
        assert_eq!(
            flow(&[vec![wait()], adds(MAX - 1), vec![halt()]].concat()),
            Ok(())
        );
        // (c) chemin de MAX + 1 refusé, `entry` exact.
        assert_eq!(flow(&[adds(MAX), vec![halt()]].concat()), budget(0));
        assert_eq!(
            flow(&[vec![wait()], adds(MAX), vec![halt()]].concat()),
            budget(1)
        );
        // Deux nœuds en dépassement : le premier est rapporté.
        let two = [adds(MAX), vec![wait()], adds(MAX), vec![halt()]].concat();
        let reported = flow(&two);
        assert_eq!(reported, budget(0));
        assert_ne!(reported, budget(MAX + 1));
        // (d) MAX instructions puis fin du programme accepté ; MAX + 1 refusé.
        assert_eq!(flow(&adds(MAX)), Ok(()));
        assert_eq!(flow(&adds(MAX + 1)), budget(0));
        // (e) `JumpIfNonZero` : bras cible seul trop long.
        let target_long = [vec![jnz(2), halt()], adds(MAX - 1), vec![halt()]].concat();
        assert_eq!(flow(&target_long), budget(0));
        // (f) bras `pc + 1` seul trop long.
        let next_long = [vec![jnz(MAX + 1)], adds(MAX - 1), vec![halt(), halt()]].concat();
        assert_eq!(flow(&next_long), budget(0));
        // (g) plus long chemin au-dessus de la limite, plus court en dessous.
        let long = MAX - 2;
        let branch = [vec![jnz(long + 1)], adds(long), vec![add(), halt()]].concat();
        assert_eq!(flow(&branch), budget(0));
        // (h) les deux bras à la limite exacte : accepté.
        let both = [
            vec![jnz(MAX)],
            adds(MAX - 2),
            vec![halt()],
            adds(MAX - 2),
            vec![halt()],
        ]
        .concat();
        assert_eq!(flow(&both), Ok(()));
        // (i) plus de MAX instructions au total, chaque segment tient : accepté.
        let segment = [adds(MAX - 1), vec![wait()]].concat();
        let segmented = [segment.clone(), segment.clone(), segment, vec![halt()]].concat();
        assert!(segmented.len() > MAX);
        assert_eq!(flow(&segmented), Ok(()));
        // (j) entrée de reprise à la fin du programme (coût 0) acceptée.
        assert_eq!(flow(&[add(), wait()]), Ok(()));
        assert_eq!(flow(&[wait()]), Ok(()));
        // (k1) portion inaccessible acyclique : `BudgetExceeded` sur le premier nœud inaccessible.
        assert_eq!(flow(&[vec![halt()], adds(MAX + 1)].concat()), budget(1));
        assert_eq!(flow(&[vec![halt()], adds(MAX)].concat()), Ok(()));
        // (k2) portion inaccessible cyclique : `CycleWithoutYield` ; acceptée avec un `Wait`.
        assert_eq!(flow(&[halt(), jump(1)]), Err(ForgeError::CycleWithoutYield));
        assert_eq!(flow(&[halt(), wait(), jump(1)]), Ok(()));
        // (l) priorité : le cycle l'emporte sur le budget, dans les deux dispositions.
        let cycle_unreachable = [adds(MAX + 1), vec![halt(), jump(MAX + 2)]].concat();
        assert_eq!(flow(&cycle_unreachable), Err(ForgeError::CycleWithoutYield));
        let cycle_reachable = [vec![jump(0)], adds(MAX + 1)].concat();
        assert_eq!(flow(&cycle_reachable), Err(ForgeError::CycleWithoutYield));

        // Bout en bout, sources TOML générées à partir de `MAX` : limite acceptée, `+ 1` refusé.
        let at_limit = source(&[(ADD_STEP, MAX - 1), (HALT_STEP, 1)]);
        assert!(compile_script(&at_limit).is_ok());
        let over = source(&[(ADD_STEP, MAX), (HALT_STEP, 1)]);
        assert_eq!(compile_script(&over).map(|_| ()), budget(0));
        if !cfg!(miri) {
            let resume_ok = source(&[(WAIT_STEP, 1), (ADD_STEP, MAX - 1), (HALT_STEP, 1)]);
            assert!(compile_script(&resume_ok).is_ok());
            let resume_over = source(&[(WAIT_STEP, 1), (ADD_STEP, MAX), (HALT_STEP, 1)]);
            assert_eq!(compile_script(&resume_over).map(|_| ()), budget(1));
            let cycle = "[[step]]\nlabel = \"a\"\nop = \"jump\"\ntarget = \"a\"\n";
            assert_eq!(compile_script(cycle), Err(ForgeError::CycleWithoutYield));
        }
    }

    #[test]
    fn scr_f5_the_compilation_is_deterministic_and_the_batch_is_order_invariant() {
        let first = compile_script(REFERENCE).expect("source valide");
        assert_eq!(compile_script(REFERENCE), Ok(first));

        let script = |key: &'static str, toml: &'static str, slot: u32| AssetSource {
            kind: AssetKind::Script,
            key,
            toml,
            slot: SlotIndex::new(slot),
        };
        let a = script("a", REFERENCE, 4);
        let b = script("b", SCRIPT_B, 1);
        let reference = compile_assets(&[a, b], 6).expect("lot valide");
        let permuted = compile_assets(&[b, a], 6).expect("lot valide");
        assert_eq!(reference.artifacts()[0], permuted.artifacts()[1]);
        assert_eq!(reference.artifacts()[1], permuted.artifacts()[0]);
        let before = Manifest::parse(reference.manifest()).expect("manifeste");
        let after = Manifest::parse(permuted.manifest()).expect("manifeste");
        for entry in before.entries() {
            let moved = after.find(entry.id()).expect("même identifiant");
            assert_eq!(moved.slot_index(), entry.slot_index());
            assert_ne!(moved.artifact_index(), entry.artifact_index());
        }
    }

    #[test]
    fn scr_f6_the_output_carries_no_string() {
        let bytes = compile_script(REFERENCE).expect("source valide");
        assert_eq!(bytes.len(), 68);
        for name in [
            "loop", "emit", "wait", "set", "add", "jump", "jnz", "halt", "step", "label",
        ] {
            assert!(
                !bytes
                    .windows(name.len())
                    .any(|window| window == name.as_bytes()),
                "« {name} » figure dans la sortie"
            );
        }
    }

    #[test]
    fn scr_f7_the_step_count_limit_is_checked_without_recursion() {
        // Règle du nombre de pas, isolée de l'analyse TOML.
        assert_eq!(checked_step_count(0), Err(ForgeError::EmptyScript));
        assert_eq!(checked_step_count(1), Ok(1));
        assert_eq!(checked_step_count(65_535), Ok(u16::MAX));
        assert_eq!(checked_step_count(65_536), Err(ForgeError::TooManySteps));
        assert_eq!(
            checked_step_count(usize::MAX),
            Err(ForgeError::TooManySteps)
        );

        // `check_flow` sur de grands programmes construits en mémoire, sans récursion.
        // Natif : 65 535 instructions ; Miri : 511 (le motif doit se terminer par
        // `MAX - 1` instructions `Add`).
        let count: usize = if cfg!(miri) { 511 } else { 65_535 };
        let pattern: Vec<Instruction> = (0..count)
            .map(|index| {
                if index % MAX == MAX - 1 {
                    wait()
                } else {
                    add()
                }
            })
            .collect();
        assert_eq!(count % MAX, MAX - 1);
        assert_eq!(check_flow(&pattern), Ok(()));
        assert_eq!(check_flow(&adds(count)), budget(0));
        let chain: Vec<Instruction> = (0..count)
            .map(|index| jump((index + 1).min(count - 1)))
            .collect();
        assert_eq!(check_flow(&chain), Err(ForgeError::CycleWithoutYield));

        // Bout en bout : natif seulement (l'analyse de 65 535 tables TOML est hors de portée de Miri).
        if !cfg!(miri) {
            let full = source(&[(ADD_STEP, MAX - 1), (WAIT_STEP, 1)]).repeat(65_535 / MAX);
            let tail = source(&[(ADD_STEP, 65_535 % MAX)]);
            let at_max = format!("{full}{tail}");
            let bytes = compile_script(&at_max).expect("65 535 pas acceptés");
            assert_eq!(bytes.len(), 12 + 8 * 65_535);
            let script = Script::read(&bytes).expect("artefact conforme");
            assert_eq!(script.len(), u16::MAX);
            let too_many = format!("{at_max}{}", source(&[(ADD_STEP, 1)]));
            assert_eq!(compile_script(&too_many), Err(ForgeError::TooManySteps));
        }
    }
}
