//! Links between parts (finding 4): an ensure's `from` list no one
//! reviewed is taken as written by `assumed:link`, with little weight,
//! and what rests on it is assumed, not explained; a review answers it.
//! A require that names an ensure is that ensure: the checker takes the
//! link. A require said in words is what the world must give: what rests
//! on it is assumed too, and no review of a link lifts it.

use mitra_lang::blame::{blame, Settings as BlameSettings, Suspect};
use mitra_lang::check::check;
use mitra_lang::evidence::{self, Ledger};
use mitra_lang::explain::{explain, Explanation, Settings, Source, Verdict, CHECK, LINK, MODEL};
use mitra_lang::judgement::Prop;
use mitra_lang::model::Model;
use mitra_lang::numbers::{self, Numbers};
use mitra_lang::parse::parse;
use mitra_lang::store::read_store;

const MODEL_TEXT: &str = r#"
subject low {
    fn f(x: integer) -> y: integer where {
        claim pos: y > 0;
    }
    ensure sound: "f gives what is asked";
}
subject high {
    require named: low::sound;
    require world: "the input is small";
    ensure uses: "high uses low well" from (named);
    ensure needs: "high needs the world" from (named, world);
    ensure both: "high, either way" from (named) | (named, world);
    ensure words: "said only in words";
    ensure top: "all of high" from (uses, needs);
    ensure either: "one link and the world, or two links" from (named, world) | (uses);
}
"#;

fn model() -> Model {
    let m = parse(MODEL_TEXT).unwrap();
    assert_eq!(check(&m), vec![]);
    m
}
fn named(p: &str) -> Prop {
    Prop::Named(p.to_string())
}
fn link(pre: &[&str], at: &str) -> Prop {
    Prop::Implies {
        pre: pre.iter().map(|p| named(p)).collect(),
        subject: at.to_string(),
        post: vec![named(at)],
    }
}
fn ledger(m: &Model, text: &str) -> Ledger {
    let (ledger, findings) = read_store("s", text, m, &m.ties).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
    ledger
}
fn of(e: &Explanation, p: &Prop) -> Verdict {
    e.verdicts
        .iter()
        .find(|v| &v.prop == p)
        .unwrap_or_else(|| panic!("no verdict on {p:?}"))
        .verdict
        .clone()
}
fn takings_of<'e>(e: &'e Explanation, p: &Prop) -> Vec<(&'e str, u64, Source)> {
    e.sources
        .iter()
        .filter(|(t, _)| &t.prop == p)
        .map(|(t, s)| (t.by.as_str(), t.count, *s))
        .collect()
}
const SOUND: &str = r#"review low::sound holds by "human:ana";"#;

/// An unreviewed link is taken by `assumed:link`, once; a require that
/// names an ensure, by the checker; a require in words, by `model`.
#[test]
fn each_link_is_taken_by_its_voice() {
    let m = model();
    let e = explain(&m, &Ledger::default(), &Settings::default());
    assert_eq!(
        takings_of(&e, &link(&["high::named"], "high::uses")),
        vec![(LINK, 1, Source::Link)]
    );
    assert_eq!(
        takings_of(&e, &link(&["low::sound"], "high::named")),
        vec![(CHECK, 1, Source::Named)]
    );
    assert_eq!(
        takings_of(&e, &named("high::world")),
        vec![(MODEL, 0, Source::Model)]
    );
}

/// Resting on the evidence and a link no one reviewed: assumed, the
/// link named. On a require in words too: both named, the fewest.
#[test]
fn what_rests_on_a_link_is_assumed() {
    let m = model();
    let e = explain(&m, &ledger(&m, SOUND), &Settings::default());
    assert_eq!(of(&e, &named("low::sound")), Verdict::Explained);
    assert_eq!(of(&e, &named("high::named")), Verdict::Explained);
    assert_eq!(
        of(&e, &named("high::uses")),
        Verdict::Assumed {
            links: vec![link(&["high::named"], "high::uses")],
            requires: vec![]
        }
    );
    assert_eq!(
        of(&e, &named("high::needs")),
        Verdict::Assumed {
            links: vec![link(&["high::named", "high::world"], "high::needs")],
            requires: vec![named("high::world")]
        }
    );
    // Of two lists, the one that assumes the least.
    assert_eq!(
        of(&e, &named("high::both")),
        Verdict::Assumed {
            links: vec![link(&["high::named"], "high::both")],
            requires: vec![]
        }
    );
    // Fewest links first, then fewest requires: a review is what the
    // person can do.
    assert_eq!(
        of(&e, &named("high::either")),
        Verdict::Assumed {
            links: vec![link(&["high::named", "high::world"], "high::either")],
            requires: vec![named("high::world")]
        }
    );
    // Through two links, both named.
    let Verdict::Assumed { links, requires } = of(&e, &named("high::top")) else {
        panic!("top is not assumed");
    };
    assert_eq!(links.len(), 3, "{links:?}");
    assert_eq!(requires, vec![named("high::world")]);
    // A require in words alone: assumed, with no link to review.
    assert_eq!(
        of(&e, &named("high::world")),
        Verdict::Assumed {
            links: vec![],
            requires: vec![named("high::world")]
        }
    );
    // An ensure said only in words is no assumption: unsupported.
    assert_eq!(of(&e, &named("high::words")), Verdict::Unsupported);
    // Without the evidence below, nothing is assumed: unsupported.
    let bare = explain(&m, &Ledger::default(), &Settings::default());
    assert_eq!(of(&bare, &named("high::uses")), Verdict::Unsupported);
}

/// A review of the link answers it: it is not taken as written, and
/// what rests on it is explained. One that fails takes it false. A
/// refused one answers nothing.
#[test]
fn a_review_answers_the_link() {
    let m = model();
    let text = format!("{SOUND}\nreview high::uses from holds by \"human:ana\";");
    let e = explain(&m, &ledger(&m, &text), &Settings::default());
    let uses = link(&["high::named"], "high::uses");
    assert_eq!(
        takings_of(&e, &uses),
        vec![("human:ana", 1, Source::Review(1))]
    );
    assert_eq!(of(&e, &named("high::uses")), Verdict::Explained);
    assert_eq!(of(&e, &uses), Verdict::Explained);
    // A review of one list of two answers that one only.
    let text = format!("{SOUND}\nreview high::both from (high::named) holds by \"human:ana\";");
    let e = explain(&m, &ledger(&m, &text), &Settings::default());
    assert_eq!(of(&e, &named("high::both")), Verdict::Explained);
    assert_eq!(
        takings_of(&e, &link(&["high::named", "high::world"], "high::both")),
        vec![(LINK, 1, Source::Link)]
    );
    // A review that fails.
    let text = format!("{SOUND}\nreview high::uses from fails by \"human:ana\";");
    let e = explain(&m, &ledger(&m, &text), &Settings::default());
    assert_eq!(
        takings_of(&e, &uses),
        vec![("human:ana", 1, Source::Review(1))]
    );
    assert!(matches!(of(&e, &uses), Verdict::Contradicted { .. }));
    assert_eq!(of(&e, &named("high::uses")), Verdict::Unsupported);
    // A test is never a review voice: refused, the link stays assumed.
    let (l, _) = read_store(
        "s",
        &format!("{SOUND}\nreview high::uses from holds by \"test:t\";"),
        &m,
        &m.ties,
    )
    .unwrap();
    let e = explain(&m, &l, &Settings::default());
    assert_eq!(takings_of(&e, &uses), vec![(LINK, 1, Source::Link)]);
    assert!(matches!(
        of(&e, &named("high::uses")),
        Verdict::Assumed { .. }
    ));
}

/// `assumed:link` is Mitra's own voice: no ledger says it.
#[test]
fn the_link_voice_is_mitras_own() {
    let m = model();
    let (l, _) = read_store(
        "s",
        r#"review high::uses from holds by "assumed:link";"#,
        &m,
        &m.ties,
    )
    .unwrap();
    let errors = evidence::check(&m, &l);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0]
            .message
            .contains("`assumed:link` is Mitra's own voice"),
        "{}",
        errors[0].message
    );
}

/// A link weighs `numbers.link` runs: 0.5 when not said, less than a
/// run of a test.
#[test]
fn a_link_weighs_little() {
    let m = model();
    let e = explain(&m, &Ledger::default(), &Settings::default());
    let n = Numbers::of(&e, &numbers::Settings::default());
    let b = n.prop(&link(&["high::named"], "high::uses"));
    assert!((b.belief - 0.2).abs() < 1e-9, "{b:?}");
    assert!((b.expected - 0.6).abs() < 1e-9, "{b:?}");
    let settings = numbers::Settings {
        link: 2.0,
        ..Default::default()
    };
    let n = Numbers::of(&e, &settings);
    let b = n.prop(&link(&["high::named"], "high::uses"));
    assert!((b.belief - 0.5).abs() < 1e-9, "{b:?}");
    // A voice's own weight still wins.
    let mut settings = numbers::Settings::default();
    settings.voices.insert(LINK.to_string(), 0.0);
    let n = Numbers::of(&e, &settings);
    let b = n.prop(&link(&["high::named"], "high::uses"));
    assert_eq!(b.belief, 0.0);
    // A weight that is no number is read as 0.
    let settings = numbers::Settings {
        link: f64::NAN,
        ..Default::default()
    };
    let b = Numbers::of(&e, &settings).prop(&link(&["high::named"], "high::uses"));
    assert_eq!(b.belief, 0.0);
}

/// A link no one reviewed is a prime suspect: blame doubts it before a
/// review, as it does what only the model says.
#[test]
fn blame_doubts_an_unreviewed_link_first() {
    let m = model();
    let text = format!("{SOUND}\nreview high::uses fails by \"human:bo\";");
    let e = explain(&m, &ledger(&m, &text), &Settings::default());
    let b = blame(&m, &e, &BlameSettings::default());
    assert!(!b.conflicts.is_empty());
    let n = Numbers::of(&e, &numbers::Settings::default());
    let ranked = n.rank(&b);
    let uses = link(&["high::named"], "high::uses");
    let first: Vec<&Suspect> = ranked[0].0.suspects.iter().collect();
    assert!(
        matches!(first.as_slice(), [Suspect::Taking(t)] if t.prop == uses && t.by == LINK),
        "{ranked:?}"
    );
    assert_eq!(mitra_lang::blame::kind(&e, first[0]), 0);
    // A require naming an ensure is checked: doubted last.
    let named_link = e
        .sources
        .iter()
        .find(|(_, s)| **s == Source::Named)
        .map(|(t, _)| Suspect::Taking(t.clone()))
        .unwrap();
    assert_eq!(mitra_lang::blame::kind(&e, &named_link), 4);
}

/// The links to review first are those the most verdicts assume.
#[test]
fn asks_come_by_what_a_review_lifts() {
    let m = model();
    let e = explain(&m, &ledger(&m, SOUND), &Settings::default());
    let order: Vec<String> = e
        .asks
        .iter()
        .map(|a| match &a.from {
            Some(from) => format!(
                "{} from {}",
                a.at,
                from.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            None => a.at.to_string(),
        })
        .collect();
    let at = |s: &str| order.iter().position(|o| o == s).unwrap();
    // `uses` lifts itself, its link and `top`; `top`'s own link lifts
    // two.
    assert!(
        at("high::uses from high::named") < at("high::top from high::uses,high::needs"),
        "{order:?}"
    );
    assert!(
        at("high::top from high::uses,high::needs") < at("high::words"),
        "{order:?}"
    );
    // The order is by what each lifts, most first.
    let lifts = |a: &mitra_lang::catalogue::Ask| {
        let Some(from) = &a.from else { return 0 };
        let link = Prop::Implies {
            pre: from.iter().map(|p| named(&p.to_string())).collect(),
            subject: a.at.to_string(),
            post: vec![named(&a.at.to_string())],
        };
        e.verdicts
            .iter()
            .filter(
                |v| matches!(&v.verdict, Verdict::Assumed { links, .. } if links.contains(&link)),
            )
            .count()
    };
    let counts: Vec<usize> = e.asks.iter().map(lifts).collect();
    let mut sorted = counts.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(counts, sorted);
    assert!(counts[0] > 1, "{counts:?}");
}
