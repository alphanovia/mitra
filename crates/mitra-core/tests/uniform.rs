//! Uniformity claims: cases the red team found, kept as tests.

const MODEL: &str = "opaque f (a : Int8) : Int\n\
    opaque h (a : Int8) : Int\n\
    def g (a : Int8) : Int := a + a\n\
    def cls (a : Int8) : {\"neg\", \"pos\"} := if a < 0 then \"neg\" else \"pos\"\n\
    claim f_spec : forall a, f a = g a\n\
    claim f_small : forall a in 0..=1, f a = g a\n\
    claim f_ok : forall a, correct (f a)\n\
    claim f_h : forall a, f a = h a\n\
    claim f_u : uniform f by cls\n";

fn eval(evidence: &str) -> String {
    mitra_core::run(MODEL, &format!("evidence f_u at 1\n{evidence}"))
        .unwrap()
        .to_text()
}

fn logic(text: &str, claim: &str) -> String {
    let line = text.lines().find(|l| l.starts_with(claim)).unwrap();
    let logic = line.split("logic := .").nth(1).unwrap();
    logic.split(',').next().unwrap().to_string()
}

/// Two claims with the same statement share their judged inputs: a pass of
/// one and a failure of the other, in one class, is a conflict.
#[test]
fn claims_saying_the_same_share_their_inputs() {
    let text = eval("evidence f_small at 1\nevidence f 5 = 11 at 1");
    assert_eq!(logic(&text, "f_u"), "fails", "{text}");
    assert!(text.contains("conflict: `f_u`"), "{text}");
}

/// A failure outside a claim's domain, in a class the domain meets, is
/// seen: here it conflicts with the pass inside.
#[test]
fn a_failure_outside_the_domain_counts_for_its_class() {
    let text = eval("evidence f 1 = 2 at 1\nevidence f 5 = 11 at 1");
    assert_eq!(logic(&text, "f_small"), "uncertain", "{text}");
    assert_eq!(logic(&text, "f_u"), "fails", "{text}");
}

/// `correct` and `=` are different statements: a pass of one says nothing
/// of the other.
#[test]
fn different_statements_do_not_mix() {
    let text = eval("evidence f 1 = 2 at 1\nevidence f (-1) = -2 at 1");
    assert_eq!(logic(&text, "f_spec"), "holds", "{text}");
    assert_eq!(logic(&text, "f_ok"), "uncertain", "{text}");
}

/// Uniformity of `f` says nothing of another subject in the claim.
#[test]
fn a_claim_calling_another_subject_assumes_nothing() {
    let text = eval("evidence f 1 = 1 at 1\nevidence h 1 = 1 at 1\nevidence f (-1) = 1 at 1\nevidence h (-1) = 1 at 1");
    assert_eq!(logic(&text, "f_h"), "uncertain", "{text}");
}

#[test]
fn structural_claims_are_not_premises() {
    let model = format!("{MODEL}claim d : forall a, correct (f a) := by derive [f_u]");
    let err = mitra_core::run(&model, "").unwrap_err().1;
    assert!(err.message.contains("can't be a premise"), "{err}");
}

/// Two outputs for one input contradict even when neither matches the
/// claim: the input is not a failure, and does not stand for its class.
#[test]
fn two_outputs_are_a_contradiction_whatever_the_claim() {
    let text = eval("evidence f 5 = 11 at 1\nevidence f 5 = 12 at 1\nevidence f 5 = 11 at 1");
    assert_eq!(logic(&text, "f_spec"), "uncertain", "{text}");
    assert_eq!(text.matches("can't both be true").count(), 1, "{text}");
    // On a subject without any claim too.
    let text = eval("evidence h 1 = 1 at 1\nevidence h 1 = 2 at 1");
    assert!(text.contains("can't both be true, on {1} of `h`"), "{text}");
}

/// Evidence on a claim says nothing outside the claim's domain: a failure
/// elsewhere does not contradict it.
#[test]
fn evidence_on_a_claim_stays_in_its_domain() {
    let text = eval("evidence f_small at 1\nevidence not f_spec 5 at 1");
    assert!(!text.contains("can't both be true"), "{text}");
}

/// The same pass and failure written several times is one contradiction.
#[test]
fn repeated_evidence_is_reported_once() {
    let pair = "evidence correct (f 3) at 1\nevidence not correct (f 3) at 1\n";
    let text = eval(&pair.repeat(4));
    assert_eq!(text.matches("can't both be true").count(), 1, "{text}");
}
