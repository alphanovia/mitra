//! Uniformity claims: cases the red team found, kept as tests.

const MODEL: &str = "import std::claims::{uniform};\n\
    fn f(a: Int8) -> Int;\n\
    fn h(a: Int8) -> Int;\n\
    fn g(a: Int8) -> Int { a + a }\n\
    fn cls(a: Int8) -> {\"neg\", \"pos\"} { if a < 0 { \"neg\" } else { \"pos\" } }\n\
    claim f_spec(a) { f(a) == g(a) }\n\
    claim f_small(a: 0..=1) { f(a) == g(a) }\n\
    claim f_ok(a) { correct(f(a)) }\n\
    claim f_h(a) { f(a) == h(a) }\n\
    claim f_u = uniform(f, cls);\n";

/// Each line `target [a] says` is one piece of evidence.
fn eval(evidence: &str) -> String {
    let mut blocks = String::from("evidence r { for f_u holds }\n");
    for line in evidence.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let (given, says) = match words[1].parse::<i64>() {
            Ok(a) => (format!("given a = {a} "), words[2..].join(" ")),
            Err(_) => (String::new(), words[1..].join(" ")),
        };
        blocks += &format!("evidence e {{ for {} {given}{says} }}\n", words[0]);
    }
    mitra_core::run(MODEL, &blocks).unwrap().to_text()
}

fn logic(text: &str, claim: &str) -> String {
    let start = format!("{claim}: ");
    let line = text.lines().find(|l| l.starts_with(&start)).unwrap();
    line[start.len()..].to_string()
}

/// Two claims with the same statement share their judged inputs: a pass of
/// one and a failure of the other, in one class, is a conflict.
#[test]
fn claims_saying_the_same_share_their_inputs() {
    let text = eval("f_small holds\nf 5 gives 11");
    assert_eq!(logic(&text, "f_u"), "fails", "{text}");
    assert!(
        text.contains("error[M004]") && text.contains("claim f_u = uniform(f, cls);"),
        "{text}"
    );
}

/// A failure outside a claim's domain, in a class the domain meets, is
/// seen: here it conflicts with the pass inside.
#[test]
fn a_failure_outside_the_domain_counts_for_its_class() {
    let text = eval("f 1 gives 2\nf 5 gives 11");
    assert_eq!(logic(&text, "f_small"), "unverified", "{text}");
    assert_eq!(logic(&text, "f_u"), "fails", "{text}");
}

/// `correct` and `=` are different statements: a pass of one says nothing
/// of the other.
#[test]
fn different_statements_do_not_mix() {
    let text = eval("f 1 gives 2\nf -1 gives -2");
    assert_eq!(logic(&text, "f_spec"), "holds", "{text}");
    assert_eq!(logic(&text, "f_ok"), "unverified", "{text}");
}

/// Uniformity of `f` says nothing of another subject in the claim.
#[test]
fn a_claim_calling_another_subject_assumes_nothing() {
    let text = eval("f 1 gives 1\nh 1 gives 1\nf -1 gives 1\nh -1 gives 1");
    assert_eq!(logic(&text, "f_h"), "unverified", "{text}");
}

/// Two outputs for one input contradict even when neither matches the
/// claim: the input is not a failure, and does not stand for its class.
#[test]
fn two_outputs_are_a_contradiction_whatever_the_claim() {
    let text = eval("f 5 gives 11\nf 5 gives 12\nf 5 gives 11");
    assert_eq!(logic(&text, "f_spec"), "unverified", "{text}");
    assert_eq!(text.matches("error[M003]").count(), 1, "{text}");
    // On a subject without any claim too.
    let text = eval("h 1 gives 1\nh 1 gives 2");
    assert!(
        text.contains("On the inputs {1} of `h`, these two say opposite things"),
        "{text}"
    );
}

/// Evidence on a claim says nothing outside the claim's domain: a failure
/// elsewhere does not contradict it.
#[test]
fn evidence_on_a_claim_stays_in_its_domain() {
    let text = eval("f_small holds\nf_spec 5 fails");
    assert!(!text.contains("error[M003]"), "{text}");
}

/// The same pass and failure written several times is one contradiction.
#[test]
fn repeated_evidence_is_reported_once() {
    let pair = "f 3 holds\nf 3 fails\n";
    let text = eval(&pair.repeat(4));
    assert_eq!(text.matches("error[M003]").count(), 1, "{text}");
}
