//! Red team of text, `[~char]` (finding 3, step 3; your "go"): `"abc"`,
//! `s.len()` and `s[i]` in bytes, `++`, `starts_with`, `==` and `<` byte
//! by byte, `.decoded()` waiting for the grapheme decoder. Each guard has
//! a test that fails when it is taken out.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::parse::parse;
use mitra_lang::print::print;

fn refused(text: &str) -> String {
    parse(text)
        .map(|_| "parsed".to_string())
        .unwrap_or_else(|e| e.to_string())
}

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap_or_else(|e| panic!("{e}")))
        .iter()
        .map(|e| e.to_string())
        .collect()
}

fn agreed(text: &str) -> Vec<String> {
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![], "{text}");
    let (agreed, _) = agree(&model, &Settings::default());
    agreed
        .iter()
        .map(|a| {
            let outcome = match &a.agreement {
                Agreement::Holds => "holds".to_string(),
                Agreement::Fails { at, .. } => {
                    let at: Vec<String> = at
                        .iter()
                        .map(|(n, v)| format!("{n} = {}", mitra_lang::print::value(v)))
                        .collect();
                    format!("fails at {}", at.join(", "))
                }
                Agreement::Unknown { why } => format!("unknown: {why}"),
            };
            format!("{} {}: {outcome}", a.at, a.against)
        })
        .collect()
}

const GREET: &str = "subject greet {
    fn hello(name: [~char]) -> g: [~char] { \"hello, \" ++ name } where {
        claim starts: g.starts_with(\"hello, \");
        claim longer: g.len() == name.len() + 7;
        examples { (\"Ada\") -> \"hello, Ada\", (\"\") -> \"hello, \" }
    }
    fn first(s: [~char]) -> b: bool { s.len() > 0 && s[0] == s[0] } where {
        claim some: b == (s.len() > 0);
    }
}";

#[test]
fn claims_on_text_are_decided() {
    // No point breaks a claim; the examples hold.
    let out = agreed(GREET);
    assert!(out.iter().all(|o| !o.contains("fails")), "{out:?}");
    assert!(
        out.iter()
            .any(|o| o.contains("example 0 against claim `starts`: holds")),
        "{out:?}"
    );
    // Joined the other way round: it no longer starts with the greeting.
    let wrong = GREET.replace("\"hello, \" ++ name", "name ++ \"hello, \"");
    let out = agreed(&wrong.replace(
        "examples { (\"Ada\") -> \"hello, Ada\", (\"\") -> \"hello, \" }",
        "",
    ));
    assert!(
        out.iter()
            .any(|o| o.contains("claim `starts`: fails at name = \"")),
        "{out:?}"
    );
}

#[test]
fn length_and_places_are_bytes() {
    // `é` is two bytes in UTF-8, as Rust's `str::len` counts it.
    let text =
        "subject s { fn f(x: integer) -> n: integer { \"é\".len() } where { claim two: n == 2; } }";
    let out = agreed(text);
    assert!(out.iter().all(|o| o.ends_with("holds")), "{out:?}");
    let wrong = text.replace("n == 2", "n == 1");
    let out = agreed(&wrong);
    assert!(out.iter().any(|o| o.contains("fails")), "{out:?}");
    // A place past the bytes has no value.
    let text =
        "subject s { fn f(x: integer) -> b: bool { \"ab\"[2] == \"ab\"[0] } where { claim c: b; } }";
    let out = agreed(text);
    assert!(out.iter().any(|o| o.contains("fails at x = ")), "{out:?}");
    // `s[i]` is a byte, not a text, nor a letter.
    let text = "subject s { fn f(s: [~char]) -> b: bool { s[0] == \"a\" } }";
    let out = errors(text);
    assert!(out.iter().any(|e| e.starts_with("error[M005]")), "{out:?}");
}

#[test]
fn texts_are_equal_and_ordered_byte_by_byte() {
    let text = "subject s {
    fn order(x: integer) -> b: bool { \"a\" < \"b\" && \"ab\" < \"b\" && \"\" < \"a\" && \"a\" <= \"a\" && !(\"é\" < \"z\") } where {
        claim c: b;
    }
    fn same(x: integer) -> b: bool { \"ab\" == \"a\" ++ \"b\" } where {
        claim c: b;
    }
}";
    let out = agreed(text);
    assert!(out.iter().all(|o| o.ends_with("holds")), "{out:?}");
    let wrong = text.replace("!(\"é\" < \"z\")", "\"é\" < \"z\"");
    let out = agreed(&wrong);
    assert!(out.iter().any(|o| o.contains("fails")), "{out:?}");
}

#[test]
fn every_text_starts_with_the_empty_one() {
    // By bounds, on a text not known.
    let text =
        "subject s { fn f(s: [~char]) -> b: bool { s.starts_with(\"\") } where { claim c: b; } }";
    let out = agreed(text);
    assert!(
        out.iter().any(|o| o.contains("claim `c`: holds")),
        "{out:?}"
    );
}

#[test]
fn decoding_text_waits_for_the_decoder() {
    let text = "subject s { fn f(s: [~char]) -> n: integer { s.decoded().len() } }";
    let out = errors(text);
    assert!(
        out.iter()
            .any(|e| e.contains("waits for the grapheme decoder")),
        "{out:?}"
    );
    // A plain sequence decodes to itself.
    let text =
        "subject s { fn f(xs: [integer]) -> b: bool { xs.decoded() == xs } where { claim c: b; } }";
    let out = agreed(text);
    assert!(out.iter().all(|o| !o.contains("fails")), "{out:?}");
    let wrong = text.replace("xs.decoded() == xs", "xs.decoded() == [1]");
    let out = agreed(&wrong);
    assert!(
        out.iter().any(|o| o.contains("fails at xs = []")),
        "{out:?}"
    );
}

#[test]
fn sequences_join_and_start_alike() {
    let text = "subject s {
    fn f(xs: [integer], ys: [integer]) -> zs: [integer] { xs ++ ys } where {
        claim long: zs.len() == xs.len() + ys.len();
        claim start: zs.starts_with(xs);
        claim empty: zs.starts_with([]);
    }
}";
    // `empty` by bounds; the others at every point tried.
    let out = agreed(text);
    assert!(out.iter().all(|o| !o.contains("fails")), "{out:?}");
    assert!(
        out.iter().any(|o| o.contains("claim `empty`: holds")),
        "{out:?}"
    );
    let wrong = text.replace("zs.starts_with(xs)", "zs.starts_with(ys)");
    let out = agreed(&wrong);
    assert!(
        out.iter().any(|o| o.contains("claim `start`: fails")),
        "{out:?}"
    );
}

#[test]
fn only_text_has_an_encoding() {
    let out = refused("subject s { fn f(x: [~integer]) -> y: integer { 0 } }");
    assert!(
        out.contains("error[P008]: only `[~char]` has an encoding today, not `[~integer]`"),
        "{out}"
    );
    // A text and a sequence are not joined, nor a letter compared to text.
    let out = errors("subject s { fn f(s: [~char], xs: [integer]) -> y: bool { s ++ xs == s } }");
    assert!(out.iter().any(|e| e.contains("`++` joins")), "{out:?}");
    let out = errors("subject s { fn f(s: [~char]) -> y: [~char] { s + s } }");
    assert!(
        out.iter()
            .any(|e| e.contains("help: two texts or two sequences are joined with `++`")),
        "{out:?}"
    );
    let out = errors("subject s { fn f(s: [~char]) -> y: integer { s.len } }");
    assert!(
        out.iter()
            .any(|e| e.contains("the length of a sequence is `.len()`")),
        "{out:?}"
    );
    let out = errors("subject s { fn f(c: char, s: [~char]) -> y: bool { c < s } }");
    assert!(out.iter().any(|e| e.starts_with("error[M005]")), "{out:?}");
    // A byte is not ordered.
    let out = errors("subject s { fn f(s: [~char]) -> y: bool { s[0] < s[1] } }");
    assert!(out.iter().any(|e| e.starts_with("error[M005]")), "{out:?}");
    // Letters compare.
    assert_eq!(
        errors("subject s { fn f(a: char, b: char) -> y: bool { a < b || a == b } }"),
        Vec::<String>::new()
    );
}

#[test]
fn a_model_of_text_reads_back_as_printed() {
    let text = "subject record Entry { title: [~char], bytes: [byte], tags: [[~char]] }
subject s {
    fn f(e: Entry, c: char) -> b: bool {
        e.title ++ (e.title ++ \"\\n\\\"q\\\"\") == \"x\" && e.title.starts_with(\"a\") && e.bytes[0] == e.title[0] && e.tags[0] < \"é\" && c == c && e.tags.decoded() == e.tags
    } where {
        examples { (Entry { title: \"a\", bytes: [], tags: [\"t\"] }, c) -> true }
    }
}";
    let model = parse(
        &text
            .replace(", c) -> true", ") -> true")
            .replace("e: Entry, c: char", "e: Entry")
            .replace(" && c == c", ""),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    let again = parse(&printed).unwrap_or_else(|e| panic!("{e}\n{printed}"));
    assert_eq!(model, again, "{printed}");
    assert_eq!(print(&again), printed);
}

mod bounds {
    //! Bounds of text that the fuzz reaches seldom: each exact value at
    //! a point lies in them.
    use mitra_lang::expression::{eval, Arg, Expression};
    use mitra_lang::simulation::{
        lies_in, Bounds, Exact, Numbers, Op, Ranges, Rational, Span, Value,
    };
    use std::collections::BTreeMap;

    type E = Expression<Op>;

    fn apply(op: Op, args: Vec<E>) -> E {
        Expression::Apply(
            op,
            args.into_iter()
                .map(|body| Arg {
                    binds: vec![],
                    body,
                })
                .collect(),
        )
    }
    fn text(s: &str) -> E {
        apply(Op::Text(s.to_string()), vec![])
    }
    fn name(n: &str) -> E {
        Expression::Name(n.to_string())
    }
    /// `e` at `exact`, and by `bounds`: the first lies in the second.
    fn lies(e: &E, exact: Value, bounds: Span) {
        let v = eval(
            &Exact::default(),
            &BTreeMap::from([("x".to_string(), exact)]),
            e,
        )
        .unwrap_or_else(|f| panic!("{f:?}"));
        let b = eval(
            &Ranges::default(),
            &BTreeMap::from([(
                "x".to_string(),
                Bounds {
                    span: bounds,
                    may_fault: false,
                },
            )]),
            e,
        )
        .unwrap_or_else(|f| panic!("{f:?}"));
        assert!(lies_in(&v, &b.span), "{v:?} not in {b:?}");
    }
    fn two() -> Span {
        Span::Number(
            Numbers::Int,
            Some(Rational::integer(2)),
            Some(Rational::integer(2)),
        )
    }

    #[test]
    fn a_prefix_as_long_as_the_whole_may_start_it() {
        // `"ab".starts_with(x)`, `x` of two bytes not known: `"ab"` is one.
        let e = apply(Op::StartsWith, vec![text("ab"), name("x")]);
        lies(
            &e,
            Value::Text("ab".into()),
            Span::Text(Box::new(two()), None),
        );
    }

    #[test]
    fn two_bytes_joined_are_either() {
        // `(if x == "a" { "a"[0] } else { "b"[0] }) == "b"[0]`.
        let byte = |s| apply(Op::Index, vec![text(s), apply(Op::Int(0.into()), vec![])]);
        let test = apply(Op::Equal, vec![name("x"), text("a")]);
        let e = apply(
            Op::Equal,
            vec![apply(Op::If, vec![test, byte("a"), byte("b")]), byte("b")],
        );
        let one = Span::Number(
            Numbers::Int,
            Some(Rational::integer(1)),
            Some(Rational::integer(1)),
        );
        lies(&e, Value::Text("b".into()), Span::Text(Box::new(one), None));
    }
}
