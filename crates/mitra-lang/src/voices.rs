//! On whose word a verdict rests: the acts behind a claim, the voices
//! that did them, how far apart those are, and how likely it is that
//! all of them are wrong. Nothing here changes a verdict.

use crate::ast::{apart, voice_parts, APART};
use crate::{Evidence, Judgment, Logic, Model, Report, Settings};
use std::collections::BTreeSet;

/// One word behind a claim: what was done and by whom.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// mitra: implements voices::Word
pub struct Word {
    pub act: String,
    /// Who wrote what was run, or who did it; `unknown` when nobody said.
    pub voice: String,
    pub cold: bool,
}

/// What a claim rests on.
#[derive(Debug, Clone, PartialEq, Default)]
// mitra: implements voices::Support
pub struct Support {
    pub words: Vec<Word>,
    /// How far apart its two furthest voices are, as a word of
    /// [`APART`]; `None` with fewer than two voices that are known.
    pub apart: Option<&'static str>,
    /// How likely every word behind it is wrong, from the rate of each
    /// kind of voice.
    pub doubt: f64,
    /// What the policy asks of it and it does not have.
    pub below: Vec<String>,
}

/// Whose word a piece of evidence is. A machine that ran a test is not
/// its author: without `written`, the author is unknown.
// mitra: implements voices::word
pub fn word(e: &Evidence) -> Word {
    let act = e.act().to_string();
    let ran = ["test", "sample", "exhaustive", "measure"].contains(&act.as_str());
    let machine = |v: &&str| v.starts_with("machine:");
    let voice = match (&e.written, e.by.as_deref()) {
        (Some(w), _) => w.clone(),
        (None, Some(by)) if !(ran && machine(&by)) => by.to_string(),
        _ => "unknown".to_string(),
    };
    Word {
        act,
        voice,
        cold: e.cold,
    }
}

/// The evidence a claim rests on: what speaks for it or for its
/// function, and for a promise what speaks for its claims, for what it
/// requires and for its capture. Evidence that denies is not support.
fn behind<'m>(
    model: &'m Model,
    name: &str,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<&'m Evidence>,
) {
    if !seen.insert(name.to_string()) {
        return;
    }
    // Evidence on a function speaks for the claim that the function is
    // right, and for a claim on single calls of it. A claim on every
    // input is held by what is said of the claim itself.
    let mut funcs: Vec<String> = Vec::new();
    if let Some(c) = model.claim(name) {
        let own = matches!(c.target, crate::Target::Correct | crate::Target::Matches(_));
        let point = c.pattern.iter().all(|p| p.var.is_none());
        if own {
            funcs.push(c.func.clone());
        } else if point {
            funcs.extend(model.subjects_of(c));
        }
    }
    let denies = |e: &Evidence| {
        matches!(
            e.judgment,
            Judgment::Holds(false) | Judgment::Correct(false) | Judgment::Sampled { ok: false, .. }
        )
    };
    for e in &model.evidence {
        let on = e.about == name || funcs.contains(&e.about);
        let own = matches!(e.source.as_deref(), Some("declared" | "assumed"));
        if on && !own && !denies(e) && !out.iter().any(|o| o.line == e.line) {
            out.push(e);
        }
    }
    for n in parts(model, name) {
        behind(model, &n, seen, out);
    }
}

const CAPTURED: &str = "::captured";

/// What a promise rests on: its claims, what it requires, its capture.
fn parts(model: &Model, name: &str) -> Vec<String> {
    let Some(e) = model.ensure(name) else {
        return Vec::new();
    };
    let rests = e.claims.iter().chain(&e.requires).chain(&e.needs).cloned();
    rests.chain([format!("{name}::captured")]).collect()
}

/// The rate at which a word of this kind is wrong, from the settings.
// mitra: implements voices::error_rate
pub fn error_rate(settings: &Settings, w: &Word) -> f64 {
    let kind = match (w.act.as_str(), voice_parts(&w.voice)) {
        ("proof", _) => "proof",
        (_, Some(("human", _))) => "human",
        (_, Some(("machine", _))) => "machine",
        (_, Some(("agent", _))) if w.cold => "cold",
        (_, Some(("agent", _))) => "agent",
        _ => "unknown",
    };
    settings.doubt(kind)
}

/// What a claim rests on, and what the policy asks that it lacks.
// mitra: implements voices::support
pub fn support(model: &Model, name: &str) -> Support {
    let mut found = Vec::new();
    behind(model, name, &mut BTreeSet::new(), &mut found);
    let mut words: BTreeSet<Word> = BTreeSet::new();
    for e in &found {
        words.insert(word(e));
        // A review of this evidence is a second voice on it.
        let source = e.source.as_deref();
        for v in model.vouches.iter().filter(|v| v.of.as_deref() == source) {
            if v.judgment == Judgment::Holds(true) || v.judgment == Judgment::Correct(true) {
                words.insert(word(v));
            }
        }
    }
    let words: Vec<Word> = words.into_iter().collect();
    let known: Vec<String> = words
        .iter()
        .map(|w| w.voice.clone())
        .filter(|v| v != "unknown")
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut far = None;
    for (i, a) in known.iter().enumerate() {
        for b in &known[i + 1..] {
            far = far.max(Some(apart(a, b)));
        }
    }
    // One factor per kind of voice that is its own: a model of a vendor,
    // cold or not; a person; a machine; a proof; the unknown.
    let mut classes: BTreeSet<(String, bool, bool)> = BTreeSet::new();
    let mut doubt = 1.0;
    for w in &words {
        let class = match voice_parts(&w.voice) {
            Some(("agent", path)) => format!("agent:{}", path[..path.len().min(2)].join("/")),
            _ => w.voice.clone(),
        };
        if classes.insert((class, w.cold, w.act == "proof")) {
            doubt *= error_rate(&model.settings, w);
        }
    }
    // A promise is doubted as its most doubted part: the claims under
    // it do not vouch for each other.
    let under = parts(model, name);
    if !under.is_empty() {
        // A capture, and what an opaque function does, hold on a word
        // only: with none, nothing is known of them. Another part with
        // no word may hold by computation, and is left out.
        let worded = [CAPTURED, "::correct", "::deterministic"];
        let worst = under.iter().filter_map(|p| {
            let s = support(model, p);
            match s.words.is_empty() {
                false => Some(s.doubt),
                true => worded.iter().any(|w| p.ends_with(w)).then_some(1.0),
            }
        });
        // Nothing said of any part: nothing to believe it on.
        doubt = worst.reduce(f64::max).unwrap_or(1.0);
    }
    let mut s = Support {
        words,
        apart: far.map(|i| APART[i]),
        doubt,
        below: Vec::new(),
    };
    let kind = match model.ensure(name) {
        Some(_) => "ensure",
        None => "claim",
    };
    if let Some(asked) = model.settings.policy.get(kind) {
        s.below = missing(asked, &s, known.len());
    }
    s
}

/// The requirements of a policy a support does not meet. A policy is
/// words separated by commas: an act (`review`), `cold`, `human`,
/// `voices 2`, `apart model`, `doubt 0.1`.
// mitra: implements voices::missing
pub fn missing(asked: &str, s: &Support, voices: usize) -> Vec<String> {
    let mut out = Vec::new();
    for want in asked.split(',').map(str::trim).filter(|w| !w.is_empty()) {
        let (word, arg) = want.split_once(' ').unwrap_or((want, ""));
        let met = match word {
            "any" => true,
            "cold" => s
                .words
                .iter()
                .any(|w| w.cold || w.voice.starts_with("human:")),
            "human" => s.words.iter().any(|w| w.voice.starts_with("human:")),
            "voices" => arg.parse::<usize>().is_ok_and(|n| voices >= n),
            "apart" => {
                let level = APART.iter().position(|a| *a == arg);
                let has = s.apart.and_then(|a| APART.iter().position(|x| *x == a));
                matches!((level, has), (Some(l), Some(h)) if h >= l && h < 5)
            }
            "doubt" => arg.parse::<f64>().is_ok_and(|d| s.doubt <= d),
            act => s.words.iter().any(|w| w.act == act),
        };
        if !met {
            out.push(want.to_string());
        }
    }
    out
}

/// Whether a policy reads: every word is a requirement.
// mitra: implements voices::policy_reads
pub fn policy_reads(asked: &str) -> Result<(), String> {
    for want in asked.split(',').map(str::trim).filter(|w| !w.is_empty()) {
        let (word, arg) = want.split_once(' ').unwrap_or((want, ""));
        let ok = match word {
            "any" | "cold" | "human" => arg.is_empty(),
            "voices" => arg.parse::<usize>().is_ok(),
            "apart" => APART[1..5].contains(&arg),
            "doubt" => arg.parse::<f64>().is_ok_and(|d| (0.0..=1.0).contains(&d)),
            act => crate::ast::ACTS.contains(&act) && arg.is_empty(),
        };
        if !ok {
            return Err(format!(
                "`{want}` is no requirement: an act, `cold`, `human`, `voices 2`, \
                 `apart session|model|vendor|kind`, `doubt 0.1`"
            ));
        }
    }
    Ok(())
}

fn rounded(x: f64) -> f64 {
    let places = if x < 0.01 { 10000.0 } else { 100.0 };
    (x * places).round() / places
}

/// The line under a verdict that says on whose word it rests. Nothing
/// when no evidence behind it names a voice.
pub(crate) fn by_line(model: &Model, name: &str) -> String {
    let s = support(model, name);
    let named = s.words.iter().any(|w| w.voice != "unknown");
    if s.words.is_empty() || !(named || !s.below.is_empty()) {
        return String::new();
    }
    let mut acts: Vec<&str> = s.words.iter().map(|w| w.act.as_str()).collect();
    acts.dedup();
    acts.sort();
    acts.dedup();
    let mut parts = Vec::new();
    for act in acts {
        let who: Vec<String> = s
            .words
            .iter()
            .filter(|w| w.act == act)
            .map(|w| format!("{}{}", w.voice, if w.cold { " cold" } else { "" }))
            .collect();
        parts.push(format!("{act} ({})", who.join(", ")));
    }
    let mut out = format!("  by              {}", parts.join(", "));
    if let Some(a) = s.apart {
        out += &format!("; voices a {a} apart");
    }
    out += &format!("; doubt {}\n", rounded(s.doubt));
    if !s.below.is_empty() {
        out += &format!("  below policy    lacks {}\n", s.below.join(", "));
    }
    out
}

/// What the promises that hold rest on, counted by the strongest kind
/// of word behind each, and what a review by a person would change.
// mitra: implements voices::summary
pub fn summary(model: &Model, report: &Report) -> String {
    let holds = |v: &&&crate::Verdict| matches!(v.logic, Logic::Holds | Logic::Assuming);
    let promises: Vec<&crate::Verdict> = report
        .verdicts
        .iter()
        .filter(|v| {
            model
                .ensure(&v.claim)
                .is_some_and(|e| !e.boundary && !e.world)
        })
        .filter(|v| !v.claim.ends_with("::captured"))
        .collect();
    let held: Vec<&&crate::Verdict> = promises.iter().filter(holds).collect();
    let (mut proof, mut human, mut apart, mut agent, mut unknown, mut below) = (0, 0, 0, 0, 0, 0);
    let mut machine = 0;
    let mut lifts: Vec<(f64, f64, String)> = Vec::new();
    let person = Word {
        act: "review".to_string(),
        voice: "human:you".to_string(),
        cold: false,
    };
    for v in &held {
        let s = support(model, &v.claim);
        let has = |f: &dyn Fn(&Word) -> bool| s.words.iter().any(f);
        if has(&|w| w.act == "proof") {
            proof += 1;
        } else if has(&|w| w.voice.starts_with("human:")) {
            human += 1;
        } else if has(&|w| w.cold) {
            apart += 1;
        } else if has(&|w| w.voice.starts_with("agent:")) {
            agent += 1;
        } else if has(&|w| w.voice.starts_with("machine:")) {
            machine += 1;
        } else {
            unknown += 1;
        }
        if !s.below.is_empty() {
            below += 1;
        }
        if !has(&|w| w.voice.starts_with("human:")) {
            let after = s.doubt * error_rate(&model.settings, &person);
            lifts.push((s.doubt, after, v.claim.clone()));
        }
    }
    let mut out = format!(
        "{} of {} promises hold: {proof} with a proof, {human} with a person's review, \
         {apart} with a cold agent's word, {agent} on agent work alone, {machine} on a machine's word, {unknown} on words nobody signed\n",
        held.len(),
        promises.len()
    );
    if model.settings.policy.values().any(|p| p != "any") {
        out += &format!("  below policy: {below}\n");
    }
    lifts.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    if !lifts.is_empty() {
        out += "  a person's review would lift the most:\n";
        for (now, after, name) in lifts.iter().take(5) {
            out += &format!(
                "    {name}: doubt {} to {}\n",
                rounded(*now),
                rounded(*after)
            );
        }
    }
    out
}
