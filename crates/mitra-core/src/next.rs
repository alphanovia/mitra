//! `mitra next`: for each uncertain claim, what is missing and the evidence
//! that would settle it.

use crate::ast::{Model, Target};
use crate::eval::{Logic, Report};
use crate::print::call;

/// Up to this many missing inputs are listed one by one.
const LIST: usize = 5;

pub fn next(model: &Model, report: &Report) -> String {
    let mut out = String::new();
    let mut seen = Vec::new();
    for v in &report.verdicts {
        if seen.contains(&&v.claim) || v.logic != Logic::Uncertain {
            continue;
        }
        seen.push(&v.claim);
        let c = model.claim(&v.claim).expect("verdicts name claims");
        let missing = v.domain.minus(&v.covered);
        out += &format!("{} : missing {}\n", c.name, missing.show(&v.domain));
        let waiting: Vec<&str> = c
            .from
            .iter()
            .filter(|p| {
                report
                    .verdicts
                    .iter()
                    .any(|w| &&w.claim == p && w.logic != Logic::Holds)
            })
            .map(|p| p.as_str())
            .collect();
        if !waiting.is_empty() {
            out += &format!("  derived: waits on {}\n", waiting.join(", "));
        }
        let subject = model.function(&c.func).is_some_and(|f| !f.is_spec());
        match missing.values() {
            Some(inputs) if subject && inputs.len() <= LIST => {
                for input in inputs {
                    let call = call(&c.func, &input);
                    out += &match c.target {
                        Target::Correct => format!("  evidence correct ({call}) at 1\n"),
                        Target::Matches(_) => format!("  evidence {call} = <output> at 1\n"),
                    };
                }
            }
            Some(inputs) => {
                out += &format!(
                    "  {} inputs: test them, or bring evidence over the whole set (a proof)\n",
                    inputs.len()
                )
            }
            None => out += "  infinite: tests can't cover it, only evidence over a set (a proof)\n",
        }
    }
    if out.is_empty() {
        out += "nothing uncertain\n";
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn lists_what_is_missing() {
        let model = "def neg (a : Bool) : Bool := !a\n\
                     opaque not (a : Bool) : Bool\n\
                     opaque id (x : Value) : Value\n\
                     claim not_spec : ∀ a, not a = neg a\n\
                     claim not_correct : ∀ a, correct (not a)\n\
                     claim id_correct : ∀ x, correct (id x)\n\
                     claim big : ∀ x ∈ Icc 0 99, correct (id x)\n\
                     claim derived : ∀ a, not a = neg a := by derive [not_spec]";
        let evidence = "evidence not true = false at 1\nevidence correct (not true) at 1";
        let m = load(model, evidence).unwrap();
        let r = evaluate(&m).unwrap();
        assert_eq!(
            next(&m, &r),
            "not_spec : missing {false}\n\
             \x20 evidence not false = <output> at 1\n\
             not_correct : missing {false}\n\
             \x20 evidence correct (not false) at 1\n\
             id_correct : missing Value\n\
             \x20 infinite: tests can't cover it, only evidence over a set (a proof)\n\
             big : missing Icc 0 99\n\
             \x20 100 inputs: test them, or bring evidence over the whole set (a proof)\n\
             derived : missing {false}\n\
             \x20 derived: waits on not_spec\n\
             \x20 evidence not false = <output> at 1\n"
        );
    }
}
