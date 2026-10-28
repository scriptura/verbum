// crates/render/tests/representation.rs

//! Tests de représentation (U2) : RP-1 à RP-4, RP-6 et RP-7.
//!
//! RP-5 (aucune modification d'une `Frame` à travers `&Frame`) est porté par les
//! doctests `compile_fail` de `src/frame.rs`.
//!
//! Aucune assertion n'affirme un ordre de la séquence d'une `Frame` (plan C8,
//! A-1 = Y) : les comparaisons de `Frame` se font en multiensemble, avec
//! multiplicité conservée (aucun doublon effacé).

use verbum_render::{Contribution, Contributions, Frame, Full};

/// Contribution de test, identifiée par `n`.
fn contribution(n: i32) -> Contribution {
    Contribution {
        x: n,
        y: n + 1,
        width: 8,
        height: 8,
        color: [n as u8, 0, 0, 255],
    }
}

/// Clé de tri locale au test. `Contribution` n'implémente volontairement ni
/// `Ord` ni `Hash` : cette fonction ne pose aucune relation d'ordre du moteur.
fn cle(c: &Contribution) -> (i32, i32, u32, u32, [u8; 4]) {
    (c.x, c.y, c.width, c.height, c.color)
}

/// Multiensemble : tri par clé locale, multiplicité conservée.
fn multiensemble(contributions: &[Contribution]) -> Vec<Contribution> {
    let mut copie = contributions.to_vec();
    copie.sort_by_key(cle);
    copie
}

/// RP-1 : jusqu'à `CAP`, `push` accepte ; la lecture rend les contributions
/// dans l'ordre d'acceptation ; `len` et `capacity` sont exacts.
#[test]
fn rp1_push_accepte_jusqu_a_la_capacite_dans_l_ordre() {
    let mut acc = Contributions::<3>::new();
    assert_eq!(acc.capacity(), 3);
    assert_eq!(acc.len(), 0);
    assert!(acc.as_slice().is_empty());

    assert_eq!(acc.push(contribution(1)), Ok(()));
    assert_eq!(acc.len(), 1);
    assert_eq!(acc.push(contribution(2)), Ok(()));
    assert_eq!(acc.push(contribution(3)), Ok(()));

    assert_eq!(acc.len(), 3);
    assert_eq!(acc.capacity(), 3);
    assert_eq!(
        acc.as_slice(),
        &[contribution(1), contribution(2), contribution(3)]
    );
}

/// RP-2 : au-delà de `CAP`, `push` rend `Err(Full)` ; contenu, ordre, longueur
/// et capacité inchangés ; avec `CAP = 0`, tout `push` est refusé.
#[test]
fn rp2_push_au_dela_de_la_capacite_est_refuse_sans_effet() {
    let mut acc = Contributions::<2>::new();
    acc.push(contribution(1)).unwrap();
    acc.push(contribution(2)).unwrap();

    assert_eq!(acc.push(contribution(3)), Err(Full));
    assert_eq!(acc.push(contribution(4)), Err(Full));
    assert_eq!(acc.as_slice(), &[contribution(1), contribution(2)]);
    assert_eq!(acc.len(), 2);
    assert_eq!(acc.capacity(), 2);

    let mut vide = Contributions::<0>::new();
    assert_eq!(vide.push(contribution(1)), Err(Full));
    assert_eq!(vide.len(), 0);
    assert_eq!(vide.capacity(), 0);
    assert!(vide.as_slice().is_empty());
}

/// RP-3 : après `clear`, l'accumulateur est vide, indiscernable de l'état
/// initial et réutilisable.
#[test]
fn rp3_clear_remet_a_l_etat_initial_et_reste_reutilisable() {
    let mut acc = Contributions::<4>::new();
    acc.push(contribution(1)).unwrap();
    acc.push(contribution(2)).unwrap();

    acc.clear();
    assert!(acc.is_empty());
    assert_eq!(acc.len(), 0);
    assert!(acc.as_slice().is_empty());
    assert_eq!(acc, Contributions::<4>::new());
    assert_eq!(acc.capacity(), 4);

    acc.push(contribution(9)).unwrap();
    assert_eq!(acc.as_slice(), &[contribution(9)]);
}

/// RP-4 : `Frame::compose` conserve toutes les contributions, sans écart,
/// sans arbitrage et sans déduplication (doublon `Q1` inclus). Source pleine
/// acceptée. Aucune assertion d'ordre de séquence.
#[test]
fn rp4_compose_conserve_toutes_les_contributions_sans_deduplication() {
    let (q1, q2) = (contribution(1), contribution(2));

    let mut acc = Contributions::<4>::new();
    acc.push(q1).unwrap();
    acc.push(q1).unwrap();
    acc.push(q2).unwrap();

    let frame = Frame::compose(&acc);
    assert_eq!(frame.len(), 3);
    assert_eq!(
        multiensemble(frame.as_slice()),
        multiensemble(&[q1, q1, q2])
    );

    // Source pleine : capacité entièrement utilisée.
    let mut pleine = Contributions::<3>::new();
    pleine.push(q1).unwrap();
    pleine.push(q1).unwrap();
    pleine.push(q2).unwrap();
    let frame_pleine = Frame::compose(&pleine);
    assert_eq!(frame_pleine.len(), 3);
    assert_eq!(
        multiensemble(frame_pleine.as_slice()),
        multiensemble(&[q1, q1, q2])
    );
}

/// RP-6 : `Frame::empty()` est l'état initial déterministe : vide, comparable,
/// égal à la composition d'une source vide.
#[test]
fn rp6_frame_empty_est_l_etat_initial_deterministe() {
    let frame = Frame::<4>::empty();
    assert!(frame.is_empty());
    assert_eq!(frame.len(), 0);
    assert!(frame.as_slice().is_empty());
    assert_eq!(frame.capacity(), 4);

    assert_eq!(frame, Frame::<4>::empty());
    assert_eq!(frame, Frame::compose(&Contributions::<4>::new()));
}

/// RP-7 : la `Frame` composée est une valeur indépendante de sa source : `clear`
/// et `push` ultérieurs sur l'accumulateur ne la modifient pas.
#[test]
fn rp7_frame_composee_est_independante_de_sa_source() {
    let mut acc = Contributions::<4>::new();
    acc.push(contribution(1)).unwrap();
    acc.push(contribution(2)).unwrap();

    let frame = Frame::compose(&acc);
    let avant = frame.clone();

    acc.clear();
    acc.push(contribution(7)).unwrap();

    assert_eq!(frame, avant);
    assert_eq!(frame.len(), 2);
}
