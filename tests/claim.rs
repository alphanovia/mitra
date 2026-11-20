//! The claims of `model/claim.mitra`, and one walk across the seam.

use mitra_core::claim::*;
use mitra_core::model::{no, opinion, trusted, yes, Model, Piece, Statement, Voice};
use mitra_core::standard::{less, number, truth, Table};
use mitra_core::value::Expression::*;
use mitra_core::value::Value;

/// The weight of a base rate that these are recorded at.
const W: f64 = 0.2;
/// What to expect of the completeness of a tie when nothing says.
const C: f64 = 0.6;

const MITRA: &str = "machine:mitra";
fn mitra() -> Voice {
    Voice::named(MITRA)
}

fn table() -> Table {
    Table::default()
}
fn truly() -> mitra_core::value::Expression {
    Given(mitra_core::standard::yes())
}
fn falsely() -> mitra_core::value::Expression {
    Given(mitra_core::standard::no())
}
fn one() -> mitra_core::value::Expression {
    Given(number(1))
}

fn claim(name: &str, expression: mitra_core::value::Expression) -> Statement<Claim> {
    Statement {
        piece: Piece::named(name),
        content: Claim { expression },
    }
}

// mitra: evidence for tests::a_number_is_not_a_statement
#[test]
fn a_number_is_not_a_statement() {
    assert!(!reads(&claim("c", one())));
    assert!(reads(&claim("c", truly())));
}

// mitra: evidence for tests::true_says_yes
#[test]
fn true_says_yes() {
    let e = observe(&claim("c", truly()), &[], &mitra(), &table()).unwrap();
    assert_eq!((e.value, e.by), (yes(), mitra()));
}

// mitra: evidence for tests::false_says_no
#[test]
fn false_says_no() {
    let seen = observe(&claim("c", falsely()), &[], &mitra(), &table()).unwrap();
    assert_eq!(seen.value, no());
}

// mitra: evidence for tests::no_value_says_nothing
#[test]
fn no_value_says_nothing() {
    assert_eq!(
        observe(
            &claim("c", Input("x".to_string(), truth())),
            &[],
            &mitra(),
            &table()
        ),
        None
    );
    assert_eq!(observe(&claim("c", one()), &[], &mitra(), &table()), None);
}

// mitra: evidence for tests::about_its_own_piece
#[test]
fn about_its_own_piece() {
    for statement in [truly(), falsely()] {
        let c = claim("c", statement);
        assert_eq!(observe(&c, &[], &mitra(), &table()).unwrap().about, c.piece);
    }
}

/// Across the seam, end to end: a value is computed, it becomes
/// evidence, the evidence moves the opinion of the claim.
#[test]
fn a_statement_computed_moves_the_opinion_of_its_claim() {
    // claim small(x) { x < 3 }
    let small = claim(
        "small",
        Apply(
            less(),
            vec![
                Input("x".to_string(), mitra_core::standard::whole()),
                Given(number(3)),
            ],
        ),
    );
    assert!(reads(&small));
    let at = |n| -> Vec<(String, Value)> { vec![("x".to_string(), number(n))] };

    let seen = observe(&small, &at(1), &mitra(), &table()).unwrap();
    // How far what Mitra computes is believed is a base rate like any other.
    let mut model = Model {
        base: vec![(seen.piece.clone(), 1.0)],
        ..Model::empty(W, C)
    };
    assert!(!trusted(&model, &small.piece, 0.0));
    model.evidence.push(seen);
    // One computation is one yes.
    assert_eq!(opinion(&model, &small.piece).r, 1.0);
    assert!(trusted(&model, &small.piece, 0.0));

    let against = observe(&small, &at(5), &mitra(), &table()).unwrap();
    assert_ne!(against.piece, model.evidence[0].piece);
    model.base.push((against.piece.clone(), 1.0));
    model.evidence.push(against);
    let o = opinion(&model, &small.piece);
    assert!(o.s == 1.0 && o.s >= o.r);
}
