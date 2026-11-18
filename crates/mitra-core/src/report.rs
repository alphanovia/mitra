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
use crate::set::Value;
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
const CODES: [(&str, &str, &str); 8] = [
    (
        "M001",
        "evidence refutes a claim",
        "A claim says a function is correct, or equal to a definition, on some\n\
         inputs, and a piece of evidence says otherwise on one of them. The\n\
         evidence may refute the claim directly, or through a class: with a\n\
         `uniform` claim that is vouched for, one failing input stands for its\n\
         whole class.\n\n\
         Either the code is wrong, or the evidence is. Mitra lists the code of\n\
         the function, the code of what it uses, and each piece of evidence,\n\
         with how plausible it is that each is the wrong one. Evidence trusted\n\
         at 1 can't be the wrong one, so the code is.\n\n\
         To fix: correct the code and record new evidence, or correct the\n\
         evidence if it is the one that is wrong.",
    ),
    (
        "M002",
        "evidence denies an assumption",
        "A `uniform` or `uses` claim can't be checked by Mitra: evidence\n\
         vouches for it or denies it. Here a piece of evidence denies it, so\n\
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
        "M101",
        "a claim is not verified",
        "No evidence covers some inputs of the claim: nothing was tested\n\
         there, and no assumption lets a tested input stand for them.\n\n\
         To fix: add a test on the missing inputs, or state a `uniform` claim\n\
         so that the tests you have stand for their classes. `mitra next`\n\
         lists what would settle each claim.",
    ),
    (
        "M102",
        "a claim holds only under assumptions",
        "Every input of the claim is covered, but part of the cover rests on\n\
         claims that no evidence supports: a `uniform` claim nobody vouched\n\
         for, or the premise of a derivation that is not verified.\n\n\
         To fix: review each assumption and record the review as evidence\n\
         (`evidence <claim> at 1`), or test until it is not needed.",
    ),
    (
        "M103",
        "an assumption is not vouched for",
        "A `uniform` or `uses` claim can't be checked by Mitra. Until a piece\n\
         of evidence vouches for it, what rests on it is only assumed.\n\n\
         To fix: have someone review it against the code, then record it:\n\
         `evidence <claim> at 1`, with a `source` naming the reviewer.",
    ),
    (
        "M104",
        "evidence that can't be used",
        "A piece of evidence is about a definition (`def`). A definition is\n\
         what Mitra computes with: it is true by construction, and evidence\n\
         can neither support nor refute it. The entry is ignored.\n\n\
         To fix: record the evidence on the function under review (an\n\
         `opaque`), or on a claim.",
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
        format!("  --> {place}  {}\n", print_claim(self.model, c))
    }

    fn evidence_text(&self, line: usize) -> String {
        let e = self.model.evidence.iter().find(|e| e.line == line);
        e.map(|e| print_evidence(std::slice::from_ref(e), Some(self.model)))
            .unwrap_or_default()
            .trim_end()
            .to_string()
    }

    /// What a piece of evidence says, in a word.
    fn role(&self, line: usize) -> &'static str {
        let e = self.model.evidence.iter().find(|e| e.line == line);
        match e.map(|e| &e.judgment) {
            Some(Judgment::Correct(true)) => "passes",
            Some(Judgment::Correct(false)) => "fails",
            Some(Judgment::Output(_)) => "output",
            Some(Judgment::Holds(true)) => "vouches",
            Some(Judgment::Holds(false)) => "denies",
            None => "",
        }
    }

    /// Inputs of `func`: as calls when there are few, as a set otherwise.
    fn inputs(&self, func: &str, set: &Tuples, domain: &Tuples) -> String {
        if set.size_bound().is_some_and(|n| n <= 3.0) {
            let values = set.values().unwrap_or_default();
            if !values.is_empty() {
                let calls: Vec<String> = values.iter().map(|v| call(func, v)).collect();
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

    fn classes(list: &[Value], spec: &str) -> String {
        let names: Vec<String> = list.iter().map(|c| c.to_string()).collect();
        let word = if names.len() == 1 { "class" } else { "classes" };
        format!("{word} {} of `{spec}`", names.join(", "))
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
        let assumed = v.assumed.as_ref().filter(|a| a.usable);
        if let (Some(a), false) = (assumed, listed) {
            let mut open = a.untested.clone();
            open.retain(|k| !(a.vouched && a.failing.contains(k)));
            if !open.is_empty() {
                return Some(Self::classes(&open, &a.spec));
            }
        }
        Some(self.inputs(func, &v.missed, &v.domain))
    }

    fn verdicts(&self, out: &mut String) {
        for v in &self.report.verdicts {
            let c = self.claim(&v.claim);
            *out += &format!("{}: {}", v.claim, v.logic.as_str());
            if v.logic == Logic::Assuming {
                *out += &format!(" [{}]", v.assuming.join(", "));
            }
            *out += "\n";
            if c.target.is_structural() {
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
                    let how = match i {
                        Implication::Checked => "its premises imply it",
                        _ => "from its premises, the implication not checked",
                    };
                    *out += &format!("  assumed on      {}: {how}\n", inputs(&v.derived));
                } else if i == Implication::Refuted {
                    *out += "  not derived: its premises do not imply it\n";
                } else if !v.follows {
                    *out += "  not derived: a premise fails\n";
                }
            }
            let assumed = v.assumed.as_ref().filter(|a| a.usable);
            if let Some(a) = assumed.filter(|a| !a.classes.is_empty() && v.logic != Logic::Fails) {
                *out += &format!(
                    "  assumed on      {}, by `{}`{}\n",
                    Self::classes(&a.classes, &a.spec),
                    a.by,
                    if a.vouched {
                        ""
                    } else {
                        ", which is not vouched for"
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
                self.places.evidence(*line),
                self.evidence_text(*line)
            );
        }
        if p.evidence.len() > QUOTED {
            *out += &format!("    and {} more\n", p.evidence.len() - QUOTED);
        }
    }

    fn piece(&self, piece: &Piece) -> String {
        match piece {
            Piece::Evidence(line) => {
                let e = self.model.evidence.iter().find(|e| e.line == *line);
                let source = e.and_then(|e| e.source.as_ref());
                let source = source.map_or(String::new(), |s| format!(" ({s})"));
                let trust = e.map_or(0.0, |e| e.confidence);
                format!(
                    "{}{source}, trusted at {trust}",
                    self.places.evidence(*line)
                )
            }
            Piece::Code(f) => {
                let line = self.model.function(f).map_or(0, |f| f.line);
                format!("the code of `{f}`, {}", self.places.model(line))
            }
            Piece::Claim(c) => {
                let line = self.claim(c).line;
                format!("the claim `{c}` itself, {}", self.places.model(line))
            }
        }
    }

    /// Which piece is most plausibly wrong.
    fn ranking(&self, p: &Problem, out: &mut String) {
        let candidates = match &p.candidates {
            Ok(c) => c,
            Err(Unranked::AllTrusted) => {
                *out +=
                    "\n  These can't all be right, and the ones trusted at 1 are enough for\n  \
                         that: nothing says which one is wrong.\n";
                return;
            }
            Err(Unranked::TooMany) => {
                *out += "\n  These can't all be right. More than 20 of them have a trust\n  \
                         between 0 and 1: too many to rank.\n";
                return;
            }
        };
        *out += match p.kind {
            Kind::Fails { .. } => {
                "\n  Either the code is wrong or the evidence is. Which one, from the trust\n  \
                 given to each piece of evidence:\n\n"
            }
            _ => {
                "\n  These can't all be right. Which one is wrong, from the trust given to\n  \
                 each:\n\n"
            }
        };
        let shown = |c: &&Candidate| c.plausibility >= self.display.threshold;
        for c in candidates.iter().filter(shown).take(self.display.max) {
            *out += &format!("    {:<9} {}", word(c), self.piece(&c.piece));
            if self.display.numbers {
                *out += &format!(
                    "  [belief {:.2}, plausibility {:.2}]",
                    c.belief, c.plausibility
                );
            }
            *out += "\n";
        }
        let listed = candidates
            .iter()
            .filter(shown)
            .take(self.display.max)
            .count();
        let possible = candidates.iter().filter(|c| c.plausibility > 0.0).count();
        if possible > listed {
            *out += &format!(
                "    {} less plausible not shown (`--all` shows them)\n",
                possible - listed
            );
        }
        if self.display.numbers {
            *out += "\n  belief: how sure it is that the piece is wrong. plausibility: how\n  \
                     far it could be wrong. Both go from 0 to 1.\n";
        }
        if p.conflict >= 0.5 && !matches!(p.kind, Kind::Fails { .. }) {
            *out += "\n  note: the trust given says these are probably all right";
            if self.display.numbers {
                *out += &format!(" (at {:.2})", p.conflict);
            }
            *out += ", which can't\n  be: at least one is trusted more than it deserves\n";
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
                let f = &self.claim(&claims[0]).func;
                let subject = self.model.function(f).is_some_and(|f| !f.is_spec());
                *out += "\n";
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
                *out += &if subject {
                    format!(
                        "  help: if the evidence is right, fix `{f}`; if not, correct the evidence\n"
                    )
                } else {
                    "  help: correct the claim or the definitions it compares\n".to_string()
                };
            }
            Kind::Denied { claim } => {
                *out += &self.at_claim(claim);
                *out += &format!(
                    "\n  The model states `{claim}`, and evidence says it does not hold:\n\n"
                );
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  help: correct or remove the claim, or correct the evidence\n";
            }
            Kind::Contradiction { subject, inputs } => {
                if let Some((line, _)) = p.evidence.first() {
                    *out += &format!("  --> {}\n", self.places.evidence(*line));
                }
                *out += &format!(
                    "\n  On the inputs {inputs} of `{subject}`, these two say opposite things:\n\n"
                );
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  note: until this is settled, neither counts for these inputs\n  \
                         help: find out which one is right, and correct the other\n";
            }
            Kind::Conflict { claim, spec, class } => {
                let func = &self.claim(claim).func;
                *out += &self.at_claim(claim);
                *out += &format!(
                    "\n  `{claim}` says `{func}` behaves the same for every input of class {class}\n  \
                     of `{spec}`. But in that class some inputs pass and others fail:\n\n"
                );
                self.quoted(p, out);
                self.ranking(p, out);
                let found = self.report.conflicts.iter();
                let mut found = found.filter(|k| &k.claim == claim && &k.class == class);
                let fails = found.next().map(|k| call(func, &k.fail.0));
                let fails = fails.unwrap_or_else(|| "the failing input".to_string());
                *out += &format!(
                    "\n  help: check that `{fails}` really fails; if it does, class {class} is\n  \
                     too wide: split it in `{spec}`\n"
                );
            }
        }
    }

    fn warnings(&self, out: &mut String) {
        for f in &self.report.flags {
            *out += &format!(
                "\nwarning[M104]: evidence that can't be used\n  --> {}  {}\n\n  {}\n",
                self.places.evidence(f.line),
                self.evidence_text(f.line),
                f.message
            );
        }
        for v in &self.report.verdicts {
            let c = self.claim(&v.claim);
            let name = &v.claim;
            match v.logic {
                Logic::Unverified if c.target.is_structural() => {
                    *out += &format!("\nwarning[M103]: {}\n", headline("M103"));
                    *out += &self.at_claim(name);
                    *out += &if v.tested.is_empty() {
                        format!(
                            "\n  Nothing says that `{name}` holds, and Mitra can't check it.\n  \
                             help: review it against the code, then record: evidence {name} at 1\n"
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
                    *out += &format!("\n  No evidence on {missed}.\n  help: {help}\n");
                }
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
        out
    }
}
