// crates/engine/shell/tests/temporal_structure.rs

//! ADR-S04 (unité T5c) : contrat statique du Shell (D7), vérifié sur les sources.
//!
//! - `[dependencies]` de `verbum-shell` vide ;
//! - aucun trait public, aucun `dyn`, aucun `Box`, aucun `unsafe`, aucun flottant, aucun type
//!   de fenêtrage (`winit`, `wgpu`) dans le code des sources (commentaires exclus) ;
//! - `Instant` n'apparaît que dans l'horloge et dans le cadencement provisoire : il ne
//!   franchit ni la politique, ni l'échéancier, ni le `Progressor`, ni le cœur d'itération.
//!
//! Ces vérifications sont structurelles et complètent la revue et `cargo tree -p verbum-shell
//! -e normal` ; elles ne prouvent rien sur les crates tiers.

const MANIFEST: &str = include_str!("../Cargo.toml");

const SOURCES: [(&str, &str); 8] = [
    ("lib.rs", include_str!("../src/lib.rs")),
    ("shell.rs", include_str!("../src/shell.rs")),
    ("inbox.rs", include_str!("../src/inbox.rs")),
    ("pacing.rs", include_str!("../src/pacing.rs")),
    ("policy.rs", include_str!("../src/policy.rs")),
    ("schedule.rs", include_str!("../src/schedule.rs")),
    ("clock.rs", include_str!("../src/clock.rs")),
    ("progressor.rs", include_str!("../src/progressor.rs")),
];

/// Code d'une source : lignes de commentaire retirées, commentaires de fin de ligne coupés.
/// Les tests unitaires (`#[cfg(test)]`) font partie de la source et sont analysés comme le reste.
fn code(source: &str) -> String {
    let mut sortie = String::new();
    for ligne in source.lines() {
        let utile = ligne.split("//").next().unwrap_or("");
        sortie.push_str(utile);
        sortie.push('\n');
    }
    sortie
}

fn est_identifiant(octet: u8) -> bool {
    octet.is_ascii_alphanumeric() || octet == b'_'
}

/// Vrai si `mot` apparaît comme mot entier (frontières d'identifiant).
fn contient_mot(texte: &str, mot: &str) -> bool {
    let octets = texte.as_bytes();
    texte.match_indices(mot).any(|(debut, _)| {
        let avant = debut == 0 || !est_identifiant(octets[debut - 1]);
        let fin = debut + mot.len();
        let apres = fin >= octets.len() || !est_identifiant(octets[fin]);
        avant && apres
    })
}

/// D7 : la section `[dependencies]` existe et ne contient aucune entrée.
#[test]
fn d7_dependances_vides() {
    let mut lignes = MANIFEST.lines();
    assert!(
        lignes.any(|l| l.trim() == "[dependencies]"),
        "section [dependencies] attendue"
    );
    for ligne in lignes {
        let ligne = ligne.trim();
        if ligne.starts_with('[') {
            break;
        }
        assert!(
            ligne.is_empty() || ligne.starts_with('#'),
            "dépendance de production interdite : {ligne}"
        );
    }
}

/// D7 : aucun trait public, `dyn`, `Box`, `unsafe`, flottant ni type de fenêtrage.
#[test]
fn d7_aucun_trait_public_dyn_box_unsafe_flottant_fenetrage() {
    for (nom, source) in SOURCES {
        let code = code(source);
        for interdit in ["dyn", "Box", "unsafe", "f32", "f64", "winit", "wgpu"] {
            assert!(
                !contient_mot(&code, interdit),
                "{nom} : `{interdit}` interdit"
            );
        }
        assert!(
            !code.contains("pub trait") && !code.contains("pub(crate) trait"),
            "{nom} : aucun trait public ni crate-public"
        );
    }
}

/// `Instant` reste confiné à l'horloge et au cadencement provisoire.
#[test]
fn d7_instant_confine_a_l_horloge_et_au_cadencement() {
    for (nom, source) in SOURCES {
        let code = code(source);
        let autorise = nom == "clock.rs" || nom == "pacing.rs";
        assert!(
            autorise || !contient_mot(&code, "Instant"),
            "{nom} : `Instant` ne doit pas franchir ce module"
        );
    }
}
