//! The report as text: each claim's verdict, then the errors, warnings and
//! notes, written like a compiler's diagnostics. Each has a stable code
//! (`mitra explain <code>`), a headline, a location, a description in
//! plain words and, where there is one, the next thing to do.

use crate::ast::{Claim, Judgment, Model};
use crate::eval::{
    brief, Candidate, Display, Implication, Kind, Logic, Piece, Problem, Report, Unranked, Verdict,
    QUOTED, WIDE,
};
use crate::print::{call, print_claim, print_evidence};
use crate::tuples::Tuples;

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

/// Every code, with its headline and its long explanation.
const CODES: [(&str, &str, &str); 13] = [
    (
        "M001",
        "evidence refutes a claim",
        "A claim says a function is correct, or equal to a definition, on some\n\
         inputs, and a piece of evidence says otherwise on one of them. The\n\
         evidence may refute the claim directly, or through a class: with a\n\
         `uniform` claim that is vouched for, one failing input stands for its\n\
         whole class.\n\n\
         One piece is wrong. Mitra lists the code of the function, the code\n\
         of what it uses, what the model states (the claim, an assumption, a\n\
         `uses` clause) and each piece of evidence, with how plausible it is\n\
         that each is the wrong one. A piece trusted at 1 can't be the wrong\n\
         one.\n\n\
         On a requirement (`ensure`), evidence says the promise was not met\n\
         while its claims hold. The candidates are then its text, the word\n\
         that the claims say what the text says, what it requires and the\n\
         evidence, not the code: write the claim that would have caught it.\n\n\
         To fix: correct the code and record new evidence, or correct the\n\
         evidence if it is the one that is wrong.",
    ),
    (
        "M002",
        "evidence denies an assumption",
        "A `uniform`, `represented` or `uses` claim can't be checked by Mitra:\n\
         evidence vouches for it or denies it. Here a piece of evidence denies it, so\n\
         nothing rests on it any more.\n\n\
         To fix: correct or remove the claim, or correct the evidence.",
    ),
    (
        "M003",
        "two pieces of evidence contradict each other",
        "On the same input, one piece of evidence says the function passes and\n\
         another says it fails, or two pieces record two different outputs.\n\
         They can't both be true. Until this is settled, neither counts for\n\
         that input, and its class is not relied on.\n\n\
         A test that passes only sometimes shows up this way.\n\n\
         To fix: find out which one is right, and correct the other.",
    ),
    (
        "M004",
        "an assumption is contradicted by the evidence",
        "A `uniform` claim says a function behaves the same for every input of\n\
         a class: one passing input stands for the class. Here one input of a\n\
         class passes and another fails, so the claim can't hold together\n\
         with both results.\n\n\
         One of them is wrong: the claim (the class is too wide), the pass, or\n\
         the failure. Mitra ranks them from the trust given to each.\n\n\
         To fix: check the failing result. If it is real, split the class so\n\
         that the two inputs fall in different classes; then fix the code.",
    ),
    (
        "M005",
        "a measurement does not fit the other evidence",
        "A measure such as `rate(x in S, cond)` has a value either from a\n\
         measurement (`evidence m { for <claim> gives rate = 0.4 }`) or from\n\
         counting the observed outputs. Here a measurement disagrees with another\n\
         measurement of the same measure, or with what the observed outputs\n\
         leave possible.\n\n\
         To fix: find out which one is right, and correct the other.",
    ),
    (
        "M006",
        "evidence does not fit the model",
        "A piece of evidence gives a function inputs, or an output, that its\n\
         signature in the model does not allow. Either the observation is\n\
         wrong, or the model is: the code does take or return such a value.\n\
         The evidence judges nothing until this is settled.\n\n\
         To fix: if the observation is real, widen the type in the model;\n\
         if not, correct the evidence.",
    ),
    (
        "M007",
        "a claim contradicts the specs of what it calls",
        "A claim calls functions that have a body in the model. Computed from\n\
         those bodies, the claim is false on an input. The model says two\n\
         things that can't both be true, before any evidence.\n\n\
         To fix: correct the claim, or the body it contradicts.",
    ),
    (
        "M101",
        "a claim is not verified",
        "No evidence covers some inputs of the claim: nothing was tested\n\
         there, and no assumption lets a tested input stand for them.\n\n\
         A requirement (`ensure`) is not verified while its subject has no\n\
         claim.\n\n\
         To fix: add a test on the missing inputs, or state a `represented`\n\
         or `uniform` claim so that the tests you have stand for the others. `mitra next`\n\
         lists what would settle each claim.",
    ),
    (
        "M102",
        "a claim holds only under assumptions",
        "Every input of the claim is covered, but part of the cover rests on\n\
         claims that no evidence supports: a `uniform` or `represented` claim\n\
         nobody vouched for, an `assume`, the spec of a function it calls\n\
         that is not verified, or, for a requirement (`ensure`), a claim of\n\
         its subject, its capture or a requirement it names.\n\n\
         To fix: review each assumption and record the review as evidence\n\
         (`evidence r { for <claim> holds }`), or test until it is not needed.",
    ),
    (
        "M103",
        "an assumption is not vouched for",
        "A `uniform`, `represented` or `uses` claim can't be checked by Mitra.\n\
         Neither can the capture of a requirement (`<ensure>::captured`):\n\
         that the claims of a subject say what the text of its `ensure`\n\
         says. Until a piece of evidence vouches for it, what rests on it is\n\
         only assumed.\n\n\
         To fix: have someone review it against the code, or the claims\n\
         against the text, then record it:\n\
         `evidence r { for <claim> holds source \"review:<who>\" }`.",
    ),
    (
        "M104",
        "evidence that can't be used",
        "A piece of evidence is about a function Mitra computes: one with a\n\
         body, outside any subject. It is true by construction, and evidence\n\
         can neither support nor refute it. The entry is ignored.\n\n\
         To fix: record the evidence on the function under review (one of a\n\
         subject, or one without a body), or on a claim.",
    ),
    (
        "M105",
        "a requirement that nothing leans on",
        "`require need = other::promise;` says this subject leans on a promise\n\
         made elsewhere, and nothing declared beside the `require` shows it:\n\
         no function `uses` one declared beside that promise, directly or\n\
         through other functions, no body or claim calls one, and no\n\
         signature or record field names one of its types. The need is\n\
         stale, or the design is not built yet.\n\n\
         To fix: add the `uses` clause to the function that calls it, or\n\
         remove the `require`.",
    ),
    (
        "M106",
        "a function uses what its subject does not require",
        "A function `uses` a function of another subject or boundary, which\n\
         makes promises (`ensure`), and no `require` beside the function names\n\
         one of them: the code relies on something the intent does not say.\n\
         Only a subject or boundary that states an `ensure` or a `require`\n\
         is held to this.\n\n\
         To fix: write `require <name> = other::promise;` beside the function.",
    ),
];

/// The explanation of a code, for `mitra explain`.
pub fn explain(code: &str) -> Option<String> {
    let found = CODES.iter().find(|c| c.0.eq_ignore_ascii_case(code));
    found.map(|(code, headline, text)| format!("{code}: {headline}\n\n{text}\n"))
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
            Some(Judgment::Correct(true)) => "passes",
            Some(Judgment::Correct(false)) => "fails",
            Some(Judgment::Output(_)) => "output",
            Some(Judgment::Holds(true)) => "vouches",
            Some(Judgment::Holds(false)) => "denies",
            Some(Judgment::Measured(..)) => "measured",
            None => "",
        }
    }

    /// Inputs of `func`: as calls when there are few, as a set otherwise.
    fn inputs(&self, func: &str, set: &Tuples, domain: &Tuples) -> String {
        if set.size_bound().is_some_and(|n| n <= 3.0) {
            let values = set.values().unwrap_or_default();
            // A property has no call of its own: `a = 5, b = 2`.
            let property = self.model.properties.iter().find(|f| f.name == func);
            let named = |v: &Vec<crate::set::Value>| -> String {
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
            .is_some_and(|n| n <= crate::eval::POINTS);
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
                return Some(crate::eval::class_text(&open, &a.spec, &a.calls));
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
            if v.logic == Logic::Fails {
                *out += &format!("  fails on        {}\n", inputs(&v.refuted));
            }
            if !v.tested.is_empty() && !v.tested.same(&v.domain) {
                *out += &format!("  passes on       {}\n", inputs(&v.tested));
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
                    crate::eval::class_text(&a.classes, &a.spec, &a.calls),
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
            if let Some(missed) = self.missed(v, &c.func) {
                *out += &format!("  no evidence on  {missed}\n");
            }
        }
    }

    /// The evidence of an error, one per line, with what each says.
    fn quoted(&self, p: &Problem, out: &mut String) {
        for (line, _) in p.evidence.iter().take(QUOTED) {
            *out += &format!(
                "    {:<9} {}  {}\n",
                self.role(*line),
                self.evidence_at(*line),
                self.evidence_text(*line)
            );
        }
        if p.evidence.len() > QUOTED {
            *out += &format!("    and {} more\n", p.evidence.len() - QUOTED);
        }
    }

    /// A piece, and the trust given to it.
    fn piece(&self, piece: &Piece) -> (String, String) {
        match piece {
            Piece::Evidence(line) => {
                let e = self.entry(*line);
                let source = e.and_then(|e| e.source.as_ref());
                let source = source.map_or(String::new(), |s| format!(" ({s})"));
                let trust = e.map_or(0.0, |e| e.confidence);
                let text = match self.declared(*line) {
                    Some(c) => format!("`{}` as{source}, {}", c.name, self.evidence_at(*line))
                        .replace("as (", "as ")
                        .replace("),", ","),
                    None => format!("{}{source}", self.evidence_at(*line)),
                };
                (text, format!("{trust}"))
            }
            Piece::Code(f) => {
                if let Some(r) = self
                    .model
                    .record(f)
                    .filter(|_| self.model.function(f).is_none())
                {
                    let place = self.places.model(r.line);
                    return (format!("the struct of `{f}`, {place}"), "none".to_string());
                }
                let line = self.model.function(f).map_or(0, |f| f.line);
                let place = self.places.model(line);
                (format!("the code of `{f}`, {place}"), "none".to_string())
            }
            Piece::Claim(c) => {
                let line = self.claim(c).line;
                let place = self.places.model(line);
                let trust = self.claim(c).trust;
                (
                    format!("the claim `{c}` itself, {place}"),
                    format!("{}", trust.unwrap_or(1.0)),
                )
            }
            Piece::Signature(f) => {
                if let Some(r) = self
                    .model
                    .record(f)
                    .filter(|_| self.model.function(f).is_none())
                {
                    let place = self.places.model(r.line);
                    return (
                        format!("the record `{f}` as written, {place}"),
                        "1".to_string(),
                    );
                }
                let line = self.model.function(f).map_or(0, |f| f.line);
                let place = self.places.model(line);
                (format!("the signature of `{f}`, {place}"), "1".to_string())
            }
            Piece::Requirement(c) => {
                let place = self.places.model(self.claim(c).line);
                let needs = self.model.ensure(c).and_then(|e| e.needs.clone());
                let text = match needs {
                    Some(n) => format!("that `{n}` is what `{c}` needs, {place}"),
                    None => format!("the text of `{c}`, {place}"),
                };
                let trust = self.claim(c).trust;
                (text, trust.map_or("none".to_string(), |t| format!("{t}")))
            }
            Piece::Capture(c) => {
                let place = self.places.model(self.claim(c).line);
                let text = format!("that the claims say what `{c}` says, {place}");
                let about = format!("{c}::captured");
                let vouches = self.model.evidence.iter().filter(|e| e.about == about);
                let trust = vouches.map(|e| e.confidence).reduce(f64::max);
                (text, trust.map_or("none".to_string(), |t| format!("{t}")))
            }
            Piece::Assumption(_) | Piece::Uses(_) => {
                let name = piece.declared_claim().expect("declared");
                let how = match piece {
                    Piece::Uses(_) => "declared",
                    _ => "assumed",
                };
                let place = self.places.model(self.claim(&name).line);
                let trust = self.stated(&name).map_or(1.0, |e| e.confidence);
                (format!("`{name}` as {how}, {place}"), format!("{trust}"))
            }
        }
    }

    /// Which piece is most plausibly wrong: the trust given to each, and
    /// the trust left once they are weighed against each other.
    fn ranking(&self, p: &Problem, out: &mut String) {
        let candidates = match &p.candidates {
            Ok(c) => c,
            Err(Unranked::TooMany) => {
                *out += "\n  These can't all be right. More than 20 of them would have to be\n  \
                         weighed against each other: too many to rank.\n";
                return;
            }
        };
        let prose = |c: &String| self.model.ensure(c).is_some();
        *out += match &p.kind {
            Kind::Fails { claims } if claims.iter().any(prose) => {
                "\n  The text, the word that the claims say it, what it rests on or requires,\n  or the evidence is wrong."
            }
            Kind::Fails { .. } => "\n  The code, the claim or the evidence is wrong.",
            Kind::Misfit { .. } => "\n  Either the model is wrong or the evidence is.",
            Kind::Against { .. } => "\n  The claim or a spec is wrong.",
            _ => "\n  These can't all be right.",
        };
        *out += " Below, the trust given to each\n  piece, and what is left of it once they are weighed against each other:\n\n";
        let shown = |c: &&Candidate| c.plausibility >= self.display.threshold;
        let listed: Vec<&Candidate> = candidates
            .iter()
            .filter(shown)
            .take(self.display.max)
            .collect();
        let rows: Vec<(String, String, String)> = listed
            .iter()
            .map(|c| {
                let (piece, trust) = self.piece(&c.piece);
                // What is left: from "it can be wrong this much" to "it
                // must be wrong this much".
                let (low, high) = (1.0 - c.plausibility, 1.0 - c.belief);
                let left = if (high - low).abs() < 0.005 {
                    format!("{low:.2}")
                } else {
                    format!("{low:.2} to {high:.2}")
                };
                (piece, trust, left)
            })
            .collect();
        let wide = |f: fn(&(String, String, String)) -> &String, head: &str| {
            let cells = rows.iter().map(|r| f(r).chars().count());
            cells.chain([head.len()]).max().unwrap_or(0)
        };
        let (w0, w1) = (wide(|r| &r.0, ""), wide(|r| &r.1, "trust"));
        *out += &format!("    {:<9} {:<w0$}  {:<w1$}  left\n", "", "", "trust");
        for (c, (piece, trust, left)) in listed.iter().zip(&rows) {
            *out += &format!("    {:<9} {piece:<w0$}  {trust:<w1$}  {left}\n", word(c));
        }
        let possible = candidates.iter().filter(|c| c.plausibility > 0.0).count();
        if possible > listed.len() {
            *out += &format!(
                "    {} less plausible not shown (`--all` shows them)\n",
                possible - listed.len()
            );
        }
        *out += "\n  The lower what is left, the more plausibly the piece is the wrong one.\n";
        if listed.iter().any(|c| c.belief + 0.005 < c.plausibility) {
            *out += "  A range means the evidence does not settle it.\n";
        }
        // Pieces trusted at 1 that are doubted all the same.
        let sure = |c: &&&Candidate| match &c.piece {
            Piece::Evidence(line) => {
                let e = self.model.evidence.iter().find(|e| e.line == *line);
                e.is_some_and(|e| e.confidence >= 1.0) && c.plausibility > 0.0
            }
            Piece::Assumption(_) | Piece::Uses(_) => {
                let name = c.piece.declared_claim().expect("declared");
                self.stated(&name).is_some_and(|e| e.confidence >= 1.0) && c.plausibility > 0.0
            }
            _ => false,
        };
        if listed.iter().filter(sure).count() > 0 {
            *out += "\n  note: the pieces trusted at 1 can't all be right, so their trust says\n  \
                     nothing here: Mitra shares the doubt equally between them. Lower the\n  \
                     trust of the one you doubt most to set them apart.\n";
        } else if p.conflict >= 0.5 && !matches!(p.kind, Kind::Fails { .. }) {
            *out += "\n  note: the trust given says these are probably all right, which can't\n  \
                     be: at least one is trusted more than it deserves\n";
        }
    }

    fn error(&self, p: &Problem, attached: &mut Vec<String>, out: &mut String) {
        let code = p.kind.code();
        *out += &format!("\nerror[{code}]: {}\n", headline(code));
        match &p.kind {
            Kind::Fails { claims } => {
                for name in claims {
                    *out += &self.at_claim(name);
                }
                *out += "\n";
                for name in claims {
                    let (c, v) = (self.claim(name), self.verdict(name));
                    if self.model.ensure(name).is_some() {
                        *out += &format!("  `{name}` is not met.\n");
                        continue;
                    }
                    *out += &format!(
                        "  `{name}` fails on {}.\n",
                        self.inputs(&c.func, &v.refuted, &v.domain)
                    );
                }
                if p.evidence.is_empty() {
                    *out += "  Its two sides are definitions, and they differ there.\n";
                } else {
                    *out += "  The evidence that says so:\n\n";
                    self.quoted(p, out);
                }
                self.ranking(p, out);
                // The subjects the claim is about: one, or for a property
                // those it calls.
                let mut subjects = self.model.subjects_of(self.claim(&claims[0]));
                subjects.retain(|f| !self.model.boundaries.contains(f));
                subjects.retain(|f| !f.ends_with("::prose"));
                subjects.retain(|f| {
                    self.model
                        .declared_function(f)
                        .is_some_and(|f| !f.is_spec())
                });
                *out += "\n";
                for f in &subjects {
                    let suspects = self.report.suspects.iter().filter(|s| &s.subject == f);
                    for s in suspects {
                        *out += &format!(
                            "  help: look first at `{}`, which `{f}` uses: {} fail{} there\n",
                            s.uses,
                            s.failing.join(", "),
                            if s.failing.len() == 1 { "s" } else { "" }
                        );
                        attached.push(format!("suspect {f} {}", s.uses));
                    }
                    for r in self.report.roots.iter().filter(|r| &r.subject == f) {
                        *out += &format!("  note: {}\n", r.text());
                        attached.push(format!("root {f}"));
                    }
                }
                *out += &if self.model.ensure(&claims[0]).is_some_and(|e| e.world) {
                    "  help: if the evidence is right, the world did not keep its promise, or the
  \
                     assumptions read more into it than it says; if not, correct the evidence\n"
                        .to_string()
                } else if self.model.ensure(&claims[0]).is_some() {
                    "  help: if the evidence is right, the claims miss what the text says: write the\n  \
                     claim that would have caught it; if not, correct the evidence\n"
                        .to_string()
                } else if !subjects.is_empty() {
                    let f = subjects.join("` or `");
                    format!(
                        "  help: if the evidence is right, fix `{f}`; if not, correct the evidence\n"
                    )
                } else {
                    "  help: correct the claim or the definitions it compares\n".to_string()
                };
            }
            Kind::Denied { claim } if self.claim(claim).target == crate::Target::Tied => {
                let f = &self.claim(claim).func;
                *out += &self.at_claim(claim);
                *out += &match f.strip_suffix("::tied::prose") {
                    Some(r) => format!(
                        "\n  The record `{r}` and its struct do not agree, says this evidence:\n\n"
                    ),
                    None => format!(
                        "\n  The model and the code of `{f}` do not agree, says this evidence:\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  help: change the model or the code so that they say the same\n";
            }
            Kind::Denied { claim } => {
                *out += &self.at_claim(claim);
                let split = matches!(self.claim(claim).target, crate::Target::Represented(_));
                *out += &match split {
                    true => format!(
                        "\n  `{claim}` says one input stands for the others, and two of them\n  \
                         do not behave the same:\n\n"
                    ),
                    false => format!(
                        "\n  The model states `{claim}`, and evidence says it does not hold:\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  help: correct or remove the claim, or correct the evidence\n";
            }
            Kind::Contradiction { subject, inputs } => {
                if let Some((line, _)) = p.evidence.first() {
                    *out += &format!("  --> {}\n", self.evidence_at(*line));
                }
                *out += &match subject.strip_suffix("::prose") {
                    Some(name) => format!("\n  On `{name}`, these two say opposite things:\n\n"),
                    None => format!(
                        "\n  On the inputs {inputs} of `{subject}`, these two say opposite things:\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  note: until this is settled, neither counts for these inputs\n  \
                         help: find out which one is right, and correct the other\n";
            }
            Kind::Against { claim, specs, .. } => {
                *out += &self.at_claim(claim);
                let (c, v) = (self.claim(claim), self.verdict(claim));
                let at = v.against.as_ref().map(|x| Tuples::point(x));
                let input = at.map_or(String::new(), |x| self.inputs(&c.func, &x, &v.domain));
                let specs: Vec<String> = specs.iter().map(|s| format!("`{s}`")).collect();
                *out += &format!(
                    "\n  On {input}, the specs of the functions `{claim}` calls say it is false\n  \
                     ({}). No evidence is needed to see it.\n",
                    specs.join(", ")
                );
                self.ranking(p, out);
                *out += "\n  help: correct the claim, or the body of the function it contradicts\n";
            }
            Kind::Misfit { function, why } => {
                if let Some((line, _)) = p.evidence.first() {
                    *out += &format!("  --> {}\n", self.evidence_at(*line));
                }
                *out += &format!(
                    "\n  This evidence does not fit the signature of `{function}`: {why}.\n\n"
                );
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  note: until this is settled, the evidence judges nothing\n";
                *out += &format!(
                    "  help: if the observation is real, change the signature of \
                     `{function}`; if not, correct the evidence\n"
                );
            }
            Kind::Measure { measure, counted } => {
                if let Some((line, _)) = p.evidence.last() {
                    *out += &format!("  --> {}\n", self.evidence_at(*line));
                }
                *out += &match counted.is_empty() {
                    true => format!("\n  `{measure}` was measured with two values:\n\n"),
                    false => format!(
                        "\n  `{measure}` as measured does not fit what was observed \
                         ({counted}):\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  help: find out which one is right, and correct the other\n";
            }
            Kind::Conflict {
                claim,
                spec,
                class,
                calls,
            } => {
                let found = self.report.conflicts.iter();
                let on = found
                    .filter(|k| &k.claim == claim && &k.class == class && &k.calls == calls)
                    .find_map(|k| k.on.clone());
                let func = &self.claim(claim).func;
                *out += &self.at_claim(claim);
                *out += &match (spec, &on) {
                    (_, Some(on)) => format!(
                        "\n  `{claim}` says `{func}` behaves the same for every input{}.\n  \
                         So `{on}`, which calls `{func}`, should pass or fail the same way{}.\n  \
                         But some inputs pass and others fail:\n\n",
                        if calls.is_empty() { "" } else { " of a class" },
                        match calls.is_empty() {
                            true => String::new(),
                            false => format!(
                                " for\n  {}",
                                crate::eval::class_text(std::slice::from_ref(class), spec, calls)
                            ),
                        }
                    ),
                    (Some(spec), None) => format!(
                        "\n  `{claim}` says `{func}` behaves the same for every input of class {class}\n  \
                         of `{spec}`. But in that class some inputs pass and others fail:\n\n"
                    ),
                    (None, None) => format!(
                        "\n  `{claim}` says `{func}` behaves the same for every input. But some inputs\n  \
                         pass and others fail:\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                let found = self.report.conflicts.iter();
                let mut found = found.filter(|k| &k.claim == claim && &k.class == class);
                let fails = found
                    .next()
                    .map(|k| call(on.as_deref().unwrap_or(func), &k.fail.0));
                let fails = fails.unwrap_or_else(|| "the failing input".to_string());
                *out += &match spec {
                    _ if !calls.is_empty() => format!(
                        "\n  help: check that `{fails}` really fails; if it does, a class\n  \
                         is too wide: split it\n"
                    ),
                    Some(spec) => format!(
                        "\n  help: check that `{fails}` really fails; if it does, class {class} is\n  \
                         too wide: split it in `{spec}`\n"
                    ),
                    None => format!(
                        "\n  help: check that `{fails}` really fails; if it does, sort the\n  \
                         inputs of `{func}` into classes: claim {claim} = uniform({func}, <classes>);\n"
                    ),
                };
            }
        }
    }

    fn warnings(&self, out: &mut String) {
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
                            crate::next::vouch(self.model, name).trim_start()
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
    use crate::eval::Piece;

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
