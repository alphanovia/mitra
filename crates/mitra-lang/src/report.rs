//! The report as text: each claim's verdict, then the errors, warnings and
//! notes, written like a compiler's diagnostics. Each has a stable code
//! (`mitra explain <code>`), a headline, a location, a description in
//! plain words and, where there is one, the next thing to do.

mod codes;
pub use codes::explain;
use codes::CODES;
mod errors;
pub mod next;
pub mod plan;
pub mod print;

use crate::model::{Claim, Judgment, Model};
use crate::report::print::{call, print_claim, print_evidence};
use crate::types::tuples::Tuples;
use crate::verdicts::{
    brief, Candidate, Display, Implication, Kind, Logic, Piece, Problem, Report, Unranked, Verdict,
    QUOTED, WIDE,
};
use crate::Why;

/// Where a line of the model or of the evidence is, as `file:line`.
pub trait Places {
    fn model(&self, line: usize) -> String;
    fn evidence(&self, line: usize) -> String;
}

/// The model and the evidence as one file each.
pub struct Files<'a> {
    pub model: &'a str,
    pub evidence: &'a str,
}

impl Places for Files<'_> {
    fn model(&self, line: usize) -> String {
        format!("{}:{line}", self.model)
    }
    fn evidence(&self, line: usize) -> String {
        format!("{}:{line}", self.evidence)
    }
}

fn headline(code: &str) -> &'static str {
    CODES.iter().find(|c| c.0 == code).map_or("", |c| c.1)
}

/// How likely a piece is the wrong one, in a word.
// mitra: implements report::word
fn word(c: &Candidate) -> &'static str {
    if c.belief >= 0.995 {
        "certain"
    } else if c.belief >= 0.5 {
        "likely"
    } else if c.plausibility >= 0.5 {
        "possible"
    } else {
        "unlikely"
    }
}

struct Writer<'a> {
    report: &'a Report,
    model: &'a Model,
    display: &'a Display,
    places: &'a dyn Places,
}

impl Writer<'_> {
    fn claim(&self, name: &str) -> &Claim {
        self.model.claim(name).expect("reports name claims")
    }

    fn verdict(&self, name: &str) -> &Verdict {
        let found = self.report.verdicts.iter().find(|v| v.claim == name);
        found.expect("every claim has a verdict")
    }

    /// `  --> file:line  claim …`
    fn at_claim(&self, name: &str) -> String {
        let c = self.claim(name);
        let place = self.places.model(c.line);
        let written = self.model.written.as_ref();
        let text = written.and_then(|w| crate::parse::written(&w.0, c.line));
        let text = text.unwrap_or_else(|| print_claim(self.model, c));
        format!("  --> {place}  {text}\n")
    }

    fn evidence_text(&self, line: usize) -> String {
        if let Some(c) = self.declared(line) {
            let written = self.model.written.as_ref();
            let text = written.and_then(|w| crate::parse::written(&w.0, c.line));
            return text.unwrap_or_default();
        }
        let written = self.model.written.as_ref();
        if let Some(text) = written.and_then(|w| crate::parse::written(&w.1, line)) {
            return text;
        }
        let e = self.entry(line);
        e.map(|e| print_evidence(std::slice::from_ref(e), Some(self.model)))
            .unwrap_or_default()
            .trim_end()
            .to_string()
    }

    /// What a piece of evidence says, in a word.
    /// A piece of evidence by its line, also when it was set aside.
    fn entry(&self, line: usize) -> Option<&crate::Evidence> {
        let aside = self.model.misfits.iter().map(|m| &m.0);
        self.model
            .evidence
            .iter()
            .chain(aside)
            .find(|e| e.line == line)
    }

    /// The claim that a piece of evidence was declared with, in the model:
    /// an `assume`, a `uses` clause.
    fn declared(&self, line: usize) -> Option<&Claim> {
        let written = self.model.written.as_ref()?;
        let e = self.entry(line).filter(|_| line > written.1.len())?;
        self.model.claim(&e.about)
    }

    /// The verdict of a claim.
    fn logic(&self, name: &str) -> Option<Logic> {
        let v = self.report.verdicts.iter().find(|v| v.claim == name);
        v.map(|v| v.logic)
    }

    /// The evidence the model declares for an `assume` or a `uses` clause.
    fn stated(&self, name: &str) -> Option<&crate::Evidence> {
        let files = self.model.written.as_ref().map_or(0, |w| w.1.len());
        let aside = self.model.misfits.iter().map(|m| &m.0);
        let mut all = self.model.evidence.iter().chain(aside);
        all.find(|e| e.about == name && e.line > files)
    }

    /// Where a piece of evidence is.
    fn evidence_at(&self, line: usize) -> String {
        match self.declared(line) {
            Some(c) => self.places.model(c.line),
            None => self.places.evidence(line),
        }
    }

    fn role(&self, line: usize) -> &'static str {
        if self.declared(line).is_some() {
            return "declared";
        }
        let e = self.entry(line);
        match e.map(|e| &e.judgment) {
            Some(Judgment::Correct(true) | Judgment::Passed) => "passes",
            Some(Judgment::Sampled { ok: true, .. }) => "sampled",
            Some(Judgment::Correct(false) | Judgment::Sampled { ok: false, .. }) => "fails",
            Some(Judgment::Output(_)) => "output",
            Some(Judgment::Holds(true)) => "vouches",
            Some(Judgment::Holds(false)) => "denies",
            Some(Judgment::Measured(..)) => "measured",
            None => "",
        }
    }

    /// Inputs of `func`: as calls when there are few, as a set otherwise.
    fn inputs(&self, func: &str, set: &Tuples, domain: &Tuples) -> String {
        // A claim on determinism is about a function, not about inputs.
        let about = |d: &&(String, String, bool)| format!("{}::prose", d.0) == func;
        if let Some((_, f, same)) = self.model.determinism.iter().find(about) {
            return match same {
                true => format!("`{f}` giving one output per input"),
                false => format!("`{f}` giving several outputs on one input"),
            };
        }
        if set.size_bound().is_some_and(|n| n <= 3.0) {
            let values = set.values().unwrap_or_default();
            // A property has no call of its own: `a = 5, b = 2`.
            let property = self.model.properties.iter().find(|f| f.name == func);
            let named = |v: &Vec<crate::types::set::Value>| -> String {
                let names = property.unwrap().params.iter().map(|p| &p.0);
                let each: Vec<String> = names.zip(v).map(|(x, v)| format!("{x} = {v}")).collect();
                each.join(", ")
            };
            if property.is_some_and(|f| f.params.is_empty()) {
                return "the statement itself".to_string();
            }
            if !values.is_empty() {
                let calls: Vec<String> = match property {
                    Some(_) => values.iter().map(named).collect(),
                    None => values.iter().map(|v| call(func, v)).collect(),
                };
                return format!("`{}`", calls.join("`, `"));
            }
        }
        let text = set.show(domain);
        if text.len() > WIDE {
            return brief(set, domain);
        }
        let f = self.model.function(func);
        let names: Vec<&str> = f.map_or(Vec::new(), |f| {
            f.params.iter().map(|p| p.0.as_str()).collect()
        });
        match names.as_slice() {
            [] => text,
            [one] => format!("{one} in {text}"),
            many => format!("({}) in {text}", many.join(", ")),
        }
    }

    /// What no evidence covers, for a claim on inputs.
    fn missed(&self, v: &Verdict, func: &str) -> Option<String> {
        if v.missed.is_empty() {
            return None;
        }
        let listed = v
            .missed
            .size_bound()
            .is_some_and(|n| n <= crate::verdicts::POINTS);
        // Classes without a passing input, when the set can't be listed; a
        // failing class is not missed.
        let assumed = v
            .assumed
            .as_ref()
            .filter(|a| a.usable && (a.spec.is_some() || !a.calls.is_empty()));
        if let (Some(a), false) = (assumed, listed) {
            let mut open = a.untested.clone();
            open.retain(|k| !(a.vouched && a.failing.contains(k)));
            if !open.is_empty() {
                return Some(crate::verdicts::class_text(&open, &a.spec, &a.calls));
            }
        }
        Some(self.inputs(func, &v.missed, &v.domain))
    }

    fn verdicts(&self, out: &mut String) {
        for v in &self.report.verdicts {
            let c = self.claim(&v.claim);
            *out += &format!("{}: {}", v.claim, v.logic.as_str());
            if v.own_word() {
                *out += " [itself]";
            } else if v.logic == Logic::Assuming {
                *out += &format!(" [{}]", v.assuming.join(", "));
            }
            *out += "\n";
            if matches!(v.logic, Logic::Holds | Logic::Assuming) {
                *out += &crate::ledger::voices::by_line(self.model, &v.claim);
            }
            if c.target.is_structural() {
                continue;
            }
            if self.model.captured(&v.claim).is_some() {
                if v.logic == Logic::Unverified {
                    *out += "  nobody vouched that the claims say what the text says\n";
                }
                continue;
            }
            if let Some(e) = self.model.ensure(&v.claim) {
                if let Some(n) = &e.needs {
                    *out += &format!("  needs           `{n}`\n");
                    if self.logic(n) == Some(Logic::Fails) {
                        *out += &format!("  fails with      `{n}`\n");
                    }
                    continue;
                }
                *out += &format!("  says            \"{}\"\n", e.prose);
                let names = |all: &[String]| -> String {
                    let all: Vec<String> = all.iter().map(|n| format!("`{n}`")).collect();
                    all.join(", ")
                };
                let failing: Vec<String> = e
                    .claims
                    .iter()
                    .chain(&e.requires)
                    .filter(|p| self.logic(p) == Some(Logic::Fails))
                    .cloned()
                    .collect();
                if !v.refuted.is_empty() {
                    *out += "  fails: evidence says it is not met\n";
                }
                if !failing.is_empty() {
                    *out += &format!("  fails with      {}\n", names(&failing));
                }
                if !e.boundary && e.claims.is_empty() {
                    *out += "  no claim in its subject: nothing says it is met\n";
                } else if !e.boundary {
                    *out += &format!("  rests on        {}\n", names(&e.claims));
                }
                if !e.requires.is_empty() {
                    *out += &format!("  requires        {}\n", names(&e.requires));
                }
                continue;
            }
            let inputs = |set: &Tuples| self.inputs(&c.func, set, &v.domain);
            // Tests on inputs drawn from a stated domain.
            let whole = c.target == crate::Target::Correct;
            let drawn = |want: bool| {
                let mut lines = Vec::new();
                for e in &self.model.evidence {
                    let Judgment::Sampled {
                        count, ok, domain, ..
                    } = &e.judgment
                    else {
                        continue;
                    };
                    if *ok == want && (e.about == c.name || (whole && e.about == c.func)) {
                        let from = match domain.is_empty() {
                            true => "its inputs".to_string(),
                            false => format!("({domain})"),
                        };
                        let by = e.source.as_deref().unwrap_or("a test");
                        lines.push(match want {
                            true => {
                                let s = &self.model.settings;
                                let trust = (s.sample(*count) * 100.0).round() / 100.0;
                                let eps = s.tolerance;
                                format!(
                                    "{count} drawn from {from} ({by}), trust {trust} at tolerance {eps}"
                                )
                            }
                            false => format!("one of {count} drawn from {from}, not stated ({by})"),
                        });
                    }
                }
                lines
            };
            if v.logic == Logic::Fails && !v.refuted.is_empty() {
                *out += &format!("  fails on        {}\n", inputs(&v.refuted));
            }
            for l in drawn(false) {
                *out += &format!("  fails on        {l}\n");
            }
            if !v.tested.is_empty() && !v.tested.same(&v.domain) {
                *out += &format!("  passes on       {}\n", inputs(&v.tested));
            }
            for l in drawn(true).iter().filter(|_| v.logic != Logic::Holds) {
                *out += &format!("  sampled on      {l}\n");
            }
            // Tests that passed on inputs they do not state.
            let on = |e: &&crate::Evidence| {
                e.judgment == Judgment::Passed
                    && (e.about == c.name || (whole && e.about == c.func))
            };
            let by: Vec<&str> = (self.model.evidence.iter().filter(on))
                .map(|e| e.source.as_deref().unwrap_or("a test"))
                .collect();
            if !by.is_empty() && v.logic != Logic::Holds {
                *out += &format!("  passes on       inputs not stated ({})\n", by.join(", "));
            }
            if let Some(i) = v.implication {
                if !v.derived.is_empty() {
                    let names: Vec<String> = c.from.iter().map(|p| format!("`{p}`")).collect();
                    let how = match (i, c.auto) {
                        _ if c.proved => format!("it follows from {}", names.join(", ")),
                        (_, true) => "the specs of what it calls imply it".to_string(),
                        (Implication::Checked, _) => "its premises imply it".to_string(),
                        _ => "from its premises, the implication not checked".to_string(),
                    };
                    *out += &format!("  assumed on      {}: {how}\n", inputs(&v.derived));
                } else if i == Implication::Refuted && c.auto {
                    *out += "  not derived: the specs of what it calls contradict it\n";
                } else if i == Implication::Refuted {
                    *out += "  not derived: its premises do not imply it\n";
                } else if !v.follows && !c.auto {
                    *out += "  not derived: a premise fails\n";
                }
            }
            for (by, set) in v.stood.iter().filter(|_| v.logic != Logic::Fails) {
                *out += &format!("  assumed on      {}, by `{by}`\n", inputs(set));
            }
            let assumed = v.assumed.as_ref().filter(|a| a.usable);
            if let Some(a) = assumed.filter(|a| !a.classes.is_empty() && v.logic != Logic::Fails) {
                let by: Vec<String> = std::iter::once(&a.by)
                    .chain(&a.with)
                    .map(|u| format!("`{u}`"))
                    .collect();
                *out += &format!(
                    "  assumed on      {}, by {}{}\n",
                    crate::verdicts::class_text(&a.classes, &a.spec, &a.calls),
                    by.join(", "),
                    match (&a.unvouched[..], a.with.is_empty()) {
                        ([], _) => String::new(),
                        (_, true) => ", which is not vouched for".to_string(),
                        (names, false) => format!(
                            "; not vouched for: {}",
                            names
                                .iter()
                                .map(|u| format!("`{u}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    }
                );
            }
            // A function with type parameters: the instances it was
            // seen at. Each is a point, and stands for no other.
            let last = c.name.rsplit("::").next().unwrap_or_default();
            let generic = self.model.function(&c.func).filter(|_| {
                c.name.strip_suffix(last) == Some(&format!("{}::", c.func))
                    && ["correct", "spec"].contains(&last)
            });
            let at = generic.map_or(Vec::new(), |f| {
                crate::contracts::instances::seen(self.model, f)
            });
            if !at.is_empty() {
                *out += &format!("  seen at         {}\n", at.join("; "));
            }
            if let Some(missed) = self.missed(v, &c.func) {
                *out += &format!("  no evidence on  {missed}\n");
            }
            if !at.is_empty() && v.logic != Logic::Holds {
                *out += "  no evidence at  any other type\n";
            }
        }
    }

    /// The warnings the model gives by itself, before any evidence.
    fn lints(&self, out: &mut String) {
        for l in &self.model.lints {
            let written = self.model.written.as_ref();
            let text = written.and_then(|w| crate::parse::written(&w.0, l.line));
            *out += &format!("\nwarning[{}]: {}\n", l.code, headline(l.code));
            *out += &format!(
                "  --> {}  {}\n",
                self.places.model(l.line),
                text.unwrap_or_default()
            );
            *out += &format!("\n  {}\n  help: {}\n", l.message, l.help);
        }
    }

    fn warnings(&self, out: &mut String) {
        self.lints(out);
        for a in &self.model.aside {
            let (code, said) = match &a.why {
                Why::Orphaned => ("M107", format!("The model has no `{}`.", a.about)),
                Why::Stale { seen, now } => (
                    "M108",
                    format!(
                        "`{}` was {seen} when this was said; it is {now} now.",
                        a.about
                    ),
                ),
            };
            let help = match (&a.why, crate::model::machine(a.source.as_deref())) {
                (Why::Orphaned, true) => "the next run of its machine removes it".to_string(),
                (Why::Orphaned, false) => {
                    "say it again on the piece as it is named now, or `mitra remove --orphaned`"
                        .to_string()
                }
                (Why::Stale { .. }, _) => format!(
                    "look at `{0}` again; if this still stands say it again, then `mitra remove --stale {0}`",
                    a.about
                ),
            };
            *out += &format!(
                "\nwarning[{code}]: {}\n  --> {}  {}\n\n  {said} It judges nothing.\n  help: {help}\n",
                headline(code),
                self.evidence_at(a.line),
                self.evidence_text(a.line),
            );
        }
        for f in &self.report.flags {
            *out += &format!(
                "\nwarning[M104]: evidence that can't be used\n  --> {}  {}\n\n  {}\n",
                self.evidence_at(f.line),
                self.evidence_text(f.line),
                f.message
            );
        }
        for v in &self.report.verdicts {
            let c = self.claim(&v.claim);
            let name = &v.claim;
            match v.logic {
                Logic::Unverified if self.model.ensure(name).is_some() => {
                    *out += &format!("\nwarning[M101]: {}\n", headline("M101"));
                    *out += &self.at_claim(name);
                    *out += "\n  No claim in its subject says it, or a claim it rests on is not settled.\n  \
                             help: write the claims that say it, in the same subject\n";
                }
                Logic::Unverified
                    if c.target.is_structural() || self.model.captured(name).is_some() =>
                {
                    *out += &format!("\nwarning[M103]: {}\n", headline("M103"));
                    *out += &self.at_claim(name);
                    *out += &if let Some(e) = self.model.captured(name) {
                        format!(
                            "\n  Nobody vouched that the claims `{}` rests on say what its text says.\n  \
                             help: review them against the text, then record: \
                             evidence <name> {{ for {name} holds source \"review:<who>\" }}\n",
                            e.name
                        )
                    } else if v.tested.is_empty() {
                        format!(
                            "\n  Nothing says that `{name}` holds, and Mitra can't check it.\n  \
                             help: review it against the code, then record: {}",
                            crate::report::next::vouch(self.model, name).trim_start()
                        )
                    } else {
                        format!(
                            "\n  `{name}` is vouched for in part only, or both vouched for and denied.\n  \
                             help: settle the evidence on it\n"
                        )
                    };
                }
                Logic::Unverified => {
                    *out += &format!("\nwarning[M101]: {}\n", headline("M101"));
                    *out += &self.at_claim(name);
                    let missed = self.missed(v, &c.func).unwrap_or_default();
                    let help = if missed.starts_with("class") {
                        "one passing test in such a class stands for the class"
                    } else {
                        "`mitra next` lists the evidence that would settle it"
                    };
                    *out += &format!("\n  No evidence on {missed}.\n");
                    if let Some(why) = &v.unsorted {
                        *out += &format!("  note: {why}\n");
                    }
                    *out += &format!("  help: {help}\n");
                }
                Logic::Assuming if v.own_word() => {}
                // A named need says what its promise says: one warning, there.
                Logic::Assuming if self.model.ensure(name).is_some_and(|e| e.needs.is_some()) => {}
                Logic::Assuming => {
                    *out += &format!("\nwarning[M102]: {}\n", headline("M102"));
                    *out += &self.at_claim(name);
                    // A sample is trusted for its draws: nothing to review.
                    if v.assuming == [format!("{name}::sample")] {
                        *out += &format!(
                            "\n  `{name}` holds as far as its sample is trusted: no input it \
                             was not drawn on has evidence.\n  \
                             help: more draws raise its trust; evidence on the inputs settles it\n"
                        );
                        continue;
                    }
                    let on: Vec<String> = v.assuming.iter().map(|a| format!("`{a}`")).collect();
                    *out += &format!(
                        "\n  `{name}` holds if {} hold{}, and no evidence supports {}.\n  \
                         help: review {}, then record it as evidence, or test until it is not needed\n",
                        on.join(", "),
                        if on.len() == 1 { "s" } else { "" },
                        if on.len() == 1 { "it" } else { "them" },
                        if on.len() == 1 { "it" } else { "them" },
                    );
                }
                _ => {}
            }
        }
    }
}

impl Report {
    /// The result with the default display settings.
    pub fn to_text(&self) -> String {
        self.to_text_with(&Display::default())
    }

    /// The result, naming the two files `model.mitra` and `evidence.mitra`.
    pub fn to_text_with(&self, display: &Display) -> String {
        let files = Files {
            model: "model.mitra",
            evidence: "evidence.mitra",
        };
        self.to_text_at(display, &files)
    }

    /// What can be said of the model alone: the errors found with no
    /// evidence, the warnings on its `require` and `uses`, and each
    /// promise of a subject that no claim says. Nothing on what is owed.
    pub fn model_text_at(&self, display: &Display, places: &dyn Places) -> String {
        let w = Writer {
            report: self,
            model: &self.model,
            display,
            places,
        };
        let (mut out, mut attached) = (String::new(), Vec::new());
        for p in &self.problems {
            w.error(p, &mut attached, &mut out);
        }
        w.lints(&mut out);
        for e in self.model.bare_ensures() {
            out += &format!("\nwarning[M101]: {}\n", headline("M101"));
            out += &w.at_claim(&e.name);
            out += "\n  No claim in its subject says it.\n  \
                    help: write the claims that say it, in the same subject\n";
        }
        out
    }

    /// Each error, as the report prints it.
    pub fn problem_texts(&self, display: &Display, places: &dyn Places) -> Vec<String> {
        let w = Writer {
            report: self,
            model: &self.model,
            display,
            places,
        };
        let mut attached = Vec::new();
        let mut each = Vec::new();
        for p in &self.problems {
            let mut out = String::new();
            w.error(p, &mut attached, &mut out);
            each.push(out);
        }
        each
    }

    /// How many warnings [`Report::model_text_at`] prints.
    pub fn model_warnings(&self) -> usize {
        self.model.lints.len() + self.model.bare_ensures().len()
    }

    /// The result: each claim's verdict, then the errors, the warnings and
    /// the notes. `places` says where each line of the model and of the
    /// evidence is.
    pub fn to_text_at(&self, display: &Display, places: &dyn Places) -> String {
        let w = Writer {
            report: self,
            model: &self.model,
            display,
            places,
        };
        let mut out = String::new();
        w.verdicts(&mut out);
        let start = out.len();
        let mut attached = Vec::new();
        for p in &self.problems {
            w.error(p, &mut attached, &mut out);
        }
        w.warnings(&mut out);
        let mut notes = String::new();
        for s in &self.suspects {
            if !attached.contains(&format!("suspect {} {}", s.subject, s.uses)) {
                notes += &format!(
                    "note: `{}` uses `{}`, where {} fail{}: look there first\n",
                    s.subject,
                    s.uses,
                    s.failing.join(", "),
                    if s.failing.len() == 1 { "s" } else { "" }
                );
            }
        }
        for r in &self.roots {
            if !attached.contains(&format!("root {}", r.subject)) {
                notes += &format!("note: {}\n", r.text());
            }
        }
        if !notes.is_empty() {
            out += "\n";
            out += &notes;
        }
        if out.len() > start {
            let mut codes: Vec<&str> = CODES.iter().map(|c| c.0).collect();
            codes.retain(|c| out[start..].contains(&format!("[{c}]")));
            if let Some(code) = codes.first() {
                out += &format!(
                    "\n`mitra explain <code>` describes a kind of error or warning, such as `mitra explain {code}`.\n"
                );
            }
        }
        self.model.named(&out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verdicts::Piece;

    fn candidate(belief: f64, plausibility: f64) -> Candidate {
        Candidate {
            piece: Piece::Code("f".into()),
            belief,
            plausibility,
        }
    }

    // One input per test, for Mitra: each `mitra: test` line gives the
    // output the test checks.

    // mitra: evidence for t_word_certain
    #[test]
    fn t_word_certain() {
        assert_eq!(word(&candidate(1.0, 1.0)), "certain");
    }

    // mitra: evidence for t_word_likely
    #[test]
    fn t_word_likely() {
        assert_eq!(word(&candidate(0.5, 1.0)), "likely");
    }

    // mitra: evidence for t_word_possible
    #[test]
    fn t_word_possible() {
        assert_eq!(word(&candidate(0.0, 0.5)), "possible");
    }

    // mitra: evidence for t_word_unlikely
    #[test]
    fn t_word_unlikely() {
        assert_eq!(word(&candidate(0.0, 0.25)), "unlikely");
    }
}
