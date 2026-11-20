//! The claims of `model/claim.mitra`, and one walk across the seam.

use mitra_core::claim::*;
use mitra_core::piece::{no, opinion, trusted, yes, Known, Piece};
use mitra_core::value::Expression::*;
use mitra_core::value::{Type, Value};

const MITRA: &str = "machine:mitra";

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
    let e = observe(&claim("c", Yes), &[], MITRA).unwrap();
    assert_eq!((e.value, e.by.as_str()), (yes(), MITRA));
}

// mitra: evidence for claims::false_says_no
#[test]
fn false_says_no() {
    assert_eq!(observe(&claim("c", No), &[], MITRA).unwrap().value, no());
}

// mitra: evidence for claims::no_value_says_nothing
#[test]
fn no_value_says_nothing() {
    assert_eq!(
        observe(&claim("c", Input("x".to_string())), &[], MITRA),
        None
    );
    assert_eq!(observe(&claim("c", Literal(1)), &[], MITRA), None);
}

// mitra: evidence for claims::about_its_own_piece
#[test]
fn about_its_own_piece() {
    for statement in [Yes, No] {
        let c = claim("c", statement);
        assert_eq!(observe(&c, &[], MITRA).unwrap().about, c.piece);
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

    let seen = observe(&small, &at(1), MITRA).unwrap();
    // How far what Mitra computes is believed is a base rate like any other.
    let mut known = Known {
        base: vec![(seen.piece.clone(), 1.0)],
        ..Known::default()
    };
    assert!(!trusted(&known, &small.piece, 0.9));
    known.evidence.push(seen);
    assert!(trusted(&known, &small.piece, 0.9));

    let against = observe(&small, &at(5), MITRA).unwrap();
    assert_ne!(against.piece, known.evidence[0].piece);
    known.base.push((against.piece.clone(), 1.0));
    known.evidence.push(against);
    let o = opinion(&known, &small.piece);
    assert!(!trusted(&known, &small.piece, 0.9) && o.d > 0.0);
}
