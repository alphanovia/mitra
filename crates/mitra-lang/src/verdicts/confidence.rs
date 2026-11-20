//! How far a verdict is believed. A test owes two things: that it
//! passes, which a machine says, and that it is correct, which a review
//! says. A test nobody reviewed is believed at the bias of the settings;
//! a reviewed one as far as its reviewer said. A claim is believed as
//! the product of the tests it stands on and of the claims it follows
//! from, and holds when that reaches what the policy asks.
//!
//! Only tests are weighed here: any other evidence counts as before.

use super::*;

impl<'a> Judge<'a> {
    /// How far a test is believed to be correct, and who reviewed it.
    fn believed(&self, e: &Evidence, bias: f64) -> (f64, Option<String>) {
        let source = e.source.as_deref();
        let mut best: Option<(f64, Option<String>)> = None;
        for r in self.model.vouches.iter().filter(|r| r.of.as_deref() == source) {
            let who = r.voice().unwrap_or("unknown");
            let (said, who) = match r.judgment {
                Judgment::Holds(true) | Judgment::Correct(true) => (r.confidence, who.to_string()),
                _ => (0.0, format!("{who}, who says it is wrong")),
            };
            if best.as_ref().is_none_or(|b| said > b.0) {
                best = Some((said, Some(who)));
            }
        }
        best.unwrap_or((bias, None))
    }

    /// Step 3, after the verdict of a claim: its confidence, and the
    /// word `unverified` where the policy asks more than it has.
    pub(super) fn weigh(&mut self, i: usize, c: &'a Claim, v: &mut Verdict) {
        let model = self.model;
        let Some(&bias) = model.settings.bias.get("test") else {
            return;
        };
        if v.logic == Logic::Unverified {
            return;
        }
        let holds = v.logic != Logic::Fails;
        // Each way the evidence gives the verdict on some inputs is an
        // argument: its tests are all needed, so it is believed as their
        // product. Two arguments on the same inputs stand for each
        // other: the better one counts.
        type Test = (String, f64, Option<String>, String);
        let mut best: Vec<(&Tuples, f64, Vec<Test>)> = Vec::new();
        for (set, ok, lines) in &self.direct[i].arguments {
            if *ok != holds {
                continue;
            }
            let mut tests: Vec<Test> = Vec::new();
            for line in lines {
                let e = model.evidence.iter().find(|e| e.line == *line);
                let Some(e) = e.filter(|e| e.act() == "test") else {
                    continue;
                };
                let (said, who) = self.believed(e, bias);
                let source = e.source.clone().unwrap_or_else(|| "a test".to_string());
                if !tests.iter().any(|t| t.0 == source) {
                    tests.push((source, said, who, e.about.clone()));
                }
            }
            let value: f64 = tests.iter().map(|t| t.1).product();
            match best.iter_mut().find(|b| b.0.same(set)) {
                Some(b) if value > b.1 => *b = (set, value, tests),
                Some(_) => {}
                None => best.push((set, value, tests)),
            }
        }
        // One test that pins several inputs is one test.
        let mut tests: Vec<Test> = Vec::new();
        for t in best.into_iter().flat_map(|b| b.2) {
            if !tests.iter().any(|u| u.0 == t.0) {
                tests.push(t);
            }
        }
        // What it follows from: its premises, and for a promise its
        // claims, what it requires and its capture.
        let mut parts: Vec<String> = c.from.clone();
        if let Some(e) = model.ensure(&c.name) {
            parts.extend(e.claims.iter().chain(&e.requires).chain(&e.needs).cloned());
            parts.push(format!("{}::captured", c.name));
        }
        let mut from: Vec<(String, f64)> = Vec::new();
        for p in parts.iter().filter(|_| holds) {
            if let Some(n) = self.confidence_of.get(p.as_str()) {
                if !from.iter().any(|f| &f.0 == p) {
                    from.push((p.clone(), *n));
                }
            }
        }
        if tests.is_empty() && from.is_empty() {
            return;
        }
        let value = tests.iter().map(|t| t.1).chain(from.iter().map(|f| f.1)).product();
        let asked = model.settings.holds;
        // Evidence believed at 0 is no evidence, whatever the policy
        // asks: a claim neither holds nor fails on it.
        if value == 0.0 || (holds && asked.is_some_and(|a| value < a)) {
            v.logic = Logic::Unverified;
            v.assuming.clear();
        }
        self.confidence_of.insert(&c.name, value);
        v.confidence = Some(Confidence {
            value,
            tests,
            from,
            asked,
            passes: holds,
        });
    }
}
