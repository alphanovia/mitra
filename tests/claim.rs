//! The claims of `model/claim.mitra`, and one walk across the seam.

use mitra_core::claim::*;
use mitra_core::piece::{no, opinion, trusted, yes, Model, Piece, Voice};
use mitra_core::value::Expression::*;
use mitra_core::value::{Type, Value};

const MITRA: &str = "machine:mitra";
fn mitra() -> Voice {
    Voice::named(MITRA)
}

fn claim(name: &str, statement: mitra_core::value::Expression) -> Claim {
    Claim {
        piece: Piece::named(name),
        statement,
    }
}

// mitra: evidence for claims::a_number_is_not_a_statement
#[test]
fn a_number_is_not_a_statement() {
    assert!(!reads(&claim("c", Literal(1)), &[]));
    assert!(reads(&claim("c", Yes), &[]));
}

// mitra: evidence for claims::true_says_yes
#[test]
fn true_says_yes() {
    let e = observe(&claim("c", Yes), &[], &mitra()).unwrap();
    assert_eq!((e.value, e.by), (yes(), mitra()));
}

// mitra: evidence for claims::false_says_no
#[test]
fn false_says_no() {
    assert_eq!(observe(&claim("c", No), &[], &mitra()).unwrap().value, no());
}

// mitra: evidence for claims::no_value_says_nothing
#[test]
fn no_value_says_nothing() {
    assert_eq!(
        observe(&claim("c", Input("x".to_string())), &[], &mitra()),
        None
    );
    assert_eq!(observe(&claim("c", Literal(1)), &[], &mitra()), None);
}

// mitra: evidence for claims::about_its_own_piece
#[test]
fn about_its_own_piece() {
    for statement in [Yes, No] {
        let c = claim("c", statement);
        assert_eq!(observe(&c, &[], &mitra()).unwrap().about, c.piece);
    }
}

/// Across the seam, end to end: a value is computed, it becomes
/// evidence, the evidence moves the opinion of the claim.
#[test]
fn a_statement_computed_moves_the_opinion_of_its_claim() {
    // claim small(x) { x < 3 }
    let small = claim(
        "small",
        Op("<".to_string(), vec![Input("x".to_string()), Literal(3)]),
    );
    assert!(reads(&small, &[("x".to_string(), Type::OfNumber)]));
    let at = |n| vec![("x".to_string(), Value::Number(n))];

    let seen = observe(&small, &at(1), &mitra()).unwrap();
    // How far what Mitra computes is believed is a base rate like any other.
    let mut model = Model {
        base: vec![(seen.piece.clone(), 1.0)],
        ..Model::default()
    };
    assert!(!trusted(&model, &small.piece, 0.9));
    model.evidence.push(seen);
    assert!(trusted(&model, &small.piece, 0.9));

    let against = observe(&small, &at(5), &mitra()).unwrap();
    assert_ne!(against.piece, model.evidence[0].piece);
    model.base.push((against.piece.clone(), 1.0));
    model.evidence.push(against);
    let o = opinion(&model, &small.piece);
    assert!(!trusted(&model, &small.piece, 0.9) && o.d > 0.0);
}
