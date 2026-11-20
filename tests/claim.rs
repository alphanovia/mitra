//! The claims of `model/claim.mitra`, and one walk across the seam.

use mitra_core::claim::*;
use mitra_core::model::{no, opinion, trusted, yes, Model, Piece, Settings, Statement, Voice};
use mitra_core::standard::{less, number, truth, whole, Exact, Table};

type Value = mitra_core::value::Value<Exact>;
use mitra_core::value::Expression::*;

/// The weight of a base rate that these are recorded at.
const W: f64 = 0.2;
/// What to expect of the completeness of a tie when nothing says.
const C: f64 = 0.6;
/// The settings these are recorded at: a piece nobody gave a base rate
/// is expected wrong, and a voice nobody gave one is not judged.
const S: Settings = Settings {
    weight: W,
    statement: 0.0,
    evidence: 0.0,
    tie: 0.0,
    completeness: C,
    reliability: 1.0,
};

const MITRA: &str = "machine:mitra";
fn mitra() -> Voice {
    Voice::named(MITRA)
}

fn table() -> Table {
    Table::default()
}
fn truly() -> mitra_core::value::Expression<Exact> {
    Given(mitra_core::standard::yes())
}
fn falsely() -> mitra_core::value::Expression<Exact> {
    Given(mitra_core::standard::no())
}
fn one() -> mitra_core::value::Expression<Exact> {
    Given(number(1))
}

fn claim(name: &str, expression: mitra_core::value::Expression<Exact>) -> Statement<Claim> {
    Statement {
        piece: Piece::named(name),
        content: Claim { expression },
    }
}

// mitra: evidence for claims::a_number_is_not_a_statement
#[test]
fn a_number_is_not_a_statement() {
    assert!(!reads(&claim("c", one())));
    assert!(reads(&claim("c", truly())));
}

// mitra: evidence for claims::true_says_yes
#[test]
fn true_says_yes() {
    let e = observe(&claim("c", truly()), &[], &mitra(), &table()).unwrap();
    assert_eq!((e.value, e.by), (yes(), mitra()));
}

// mitra: evidence for claims::false_says_no
#[test]
fn false_says_no() {
    let seen = observe(&claim("c", falsely()), &[], &mitra(), &table()).unwrap();
    assert_eq!(seen.value, no());
}

// mitra: evidence for claims::no_value_says_nothing
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

// mitra: evidence for claims::an_observation_is_about_an_instance
#[test]
fn an_observation_is_about_an_instance() {
    for statement in [truly(), falsely()] {
        let c = claim("c", statement);
        let none: [(String, Value); 0] = [];
        let about = observe(&c, &none, &mitra(), &table()).unwrap().about;
        assert_eq!(about, instance(&c, &none));
        assert_ne!(about, c.piece);
    }
}

/// Without `uniform`, recording an observation relates nothing to the
/// claim itself.
// mitra: evidence for claims::without_uniform_nothing_leads_to_the_claim
#[test]
fn without_uniform_nothing_leads_to_the_claim() {
    let c = claim("c", truly());
    let seen = observe(&c, &[], &mitra(), &table()).unwrap();
    let m = recorded(&Model::empty(S), &c, seen.clone(), None, None).unwrap();
    assert!(m.related.is_empty());
    assert_eq!(opinion(&m, &c.piece).r, 0.0);
    let uniform = Piece::named("c::uniform");
    let with = recorded(&Model::empty(S), &c, seen.clone(), None, Some(&uniform)).unwrap();
    assert_eq!(
        with.related[0].content.pieces,
        [c.piece.clone(), seen.about.clone(), uniform]
    );
}

/// Across the seam, end to end: a value is computed, it becomes
/// evidence, the evidence moves the opinion of the claim at that input.
#[test]
fn a_statement_computed_moves_the_opinion_of_its_instance() {
    // claim small(x) { x < 3 }
    let small = claim(
        "small",
        Apply(
            less(&whole()),
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
        ..Model::empty(S)
    };
    let at_one = instance(&small, &at(1));
    assert!(!trusted(&model, &at_one, 0.0));
    model.evidence.push(seen);
    // One computation is one yes, about the claim at 1 and not the
    // claim itself.
    assert_eq!(opinion(&model, &at_one).r, 1.0);
    assert!(trusted(&model, &at_one, 0.0));
    assert_eq!(opinion(&model, &small.piece).r, 0.0);

    let against = observe(&small, &at(5), &mitra(), &table()).unwrap();
    assert_ne!(against.piece, model.evidence[0].piece);
    model.base.push((against.piece.clone(), 1.0));
    model.evidence.push(against);
    let o = opinion(&model, &instance(&small, &at(5)));
    assert!(o.s == 1.0 && o.s >= o.r);
}

/// A claim whose reading by bounds is unknown says nothing: the ranges
/// table has no equality.
// mitra: evidence for claims::an_unknown_reading_says_nothing
#[test]
fn an_unknown_reading_says_nothing() {
    use mitra_core::bounds::Ranges;
    use mitra_core::standard::equal;
    let same = Apply(equal(&whole()), vec![one(), one()]);
    assert_eq!(
        bounded(&claim("c", same), &[], 10.0, &mitra(), &Ranges),
        None
    );
    let sure = bounded(&claim("c", truly()), &[], 10.0, &mitra(), &Ranges).unwrap();
    assert_eq!((sure.value.r, sure.value.s), (10.0, 0.0));
    assert_eq!(sure.about, Piece::named("c @ "));
}
