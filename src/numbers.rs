//! Numbers (`docs/numbers.md`): how much the evidence says, read after
//! judging, never in it. Subjective Logic (Jøsang, 2016) on counts.
//!
//! The takings of one proposition, each voice weighted, add up to `r`
//! runs for it and `s` against: belief `b = r/(r+s+W)`, disbelief
//! `d = s/(r+s+W)`, uncertainty `u = W/(r+s+W)`, expected `b + a·u`,
//! with `W` the prior weight and `a` the base rate (settings). What the
//! model alone says counts nothing: `u = 1`, expected `a`.
//!
//! A proposition judgement gave is read from its rests: a rest is as
//! strong as the belief that all its takings hold, by Subjective
//! Logic's multiplication (each taking its proposition's opinion, or
//! its negation's when taken false), and the proposition as its
//! strongest rest for, and against. So one operation everywhere: a
//! product along a rest, the largest across rests, and `!p` swaps them,
//! as judgement already does. The product reads takings as independent.
//!
//! Blame ranks by doubt: a taking is wrong with what is left of the
//! chance its proposition is as it says; a measurement or a review with
//! its most doubtful taking; a diagnosis with the product of its
//! suspects'.

use crate::blame::{Blame, Diagnosis, Suspect};
use crate::explain::{Explanation, Source};
use crate::judgement::{Prop, Taking};
use std::collections::{BTreeMap, BTreeSet};

/// The numbers' settings: each is a choice, none is fixed in the code.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// What a proposition is expected to be with no evidence: 0.5.
    pub base_rate: f64,
    /// How many runs uncertainty weighs as: 2.
    pub prior_weight: f64,
    /// How many runs one count of a voice is, by voice: 1 when not said.
    pub voices: BTreeMap<String, f64>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            base_rate: 0.5,
            prior_weight: 2.0,
            voices: BTreeMap::new(),
        }
    }
}

/// An opinion in numbers: belief, disbelief and uncertainty add to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Belief {
    pub belief: f64,
    pub disbelief: f64,
    pub uncertainty: f64,
    pub expected: f64,
}

impl Belief {
    /// From a belief and a disbelief: when they add to more than 1 (a
    /// conflict), each in proportion, with no uncertainty left.
    fn of(belief: f64, disbelief: f64, settings: &Settings) -> Belief {
        let total = belief + disbelief;
        let (belief, disbelief) = match total > 1.0 {
            true => (belief / total, disbelief / total),
            false => (belief, disbelief),
        };
        let uncertainty = (1.0 - belief - disbelief).max(0.0);
        Belief {
            belief,
            disbelief,
            uncertainty,
            expected: belief + settings.base_rate * uncertainty,
        }
    }
}

/// An opinion with its base rate, as multiplication reads it.
#[derive(Clone, Copy, Debug)]
struct Said {
    b: f64,
    d: f64,
    u: f64,
    a: f64,
}

/// `x` and `y` both hold: Subjective Logic's binomial multiplication.
fn times(x: Said, y: Said) -> Said {
    let k = 1.0 - x.a * y.a;
    let (b, u) = match k > 0.0 {
        true => (
            x.b * y.b + ((1.0 - x.a) * y.a * x.b * y.u + x.a * (1.0 - y.a) * x.u * y.b) / k,
            x.u * y.u + ((1.0 - y.a) * x.b * y.u + (1.0 - x.a) * x.u * y.b) / k,
        ),
        // Both certain to hold with no evidence: the plain product.
        false => (x.b * y.b, x.u * y.u + x.b * y.u + x.u * y.b),
    };
    Said {
        b,
        d: x.d + y.d - x.d * y.d,
        u,
        a: x.a * y.a,
    }
}

/// Counts read by voice.
fn weight(t: &Taking, settings: &Settings) -> f64 {
    let w = settings.voices.get(&t.by).copied().unwrap_or(1.0);
    w * t.count as f64
}

/// What the takings of one proposition say, fused: their weighted
/// counts added, for and against.
pub fn fused<'t>(takings: impl IntoIterator<Item = &'t Taking>, settings: &Settings) -> Belief {
    let (mut r, mut s) = (0.0, 0.0);
    for t in takings {
        match t.value {
            true => r += weight(t, settings),
            false => s += weight(t, settings),
        }
    }
    let all = r + s + settings.prior_weight;
    match all > 0.0 {
        true => Belief::of(r / all, s / all, settings),
        false => Belief::of(0.0, 0.0, settings),
    }
}

/// The numbers of an explanation: each proposition taken, fused.
pub struct Numbers<'e> {
    explanation: &'e Explanation,
    taken: BTreeMap<&'e Prop, Belief>,
    settings: Settings,
}

impl<'e> Numbers<'e> {
    pub fn of(explanation: &'e Explanation, settings: &Settings) -> Numbers<'e> {
        let mut by_prop: BTreeMap<&Prop, Vec<&Taking>> = BTreeMap::new();
        for t in explanation.sources.keys() {
            by_prop.entry(&t.prop).or_default().push(t);
        }
        let taken = by_prop
            .into_iter()
            .map(|(p, ts)| (p, fused(ts, settings)))
            .collect();
        Numbers {
            explanation,
            taken,
            settings: settings.clone(),
        }
    }

    /// What a taking says, in numbers: its proposition's opinion, all
    /// its takings fused; or that of its negation when it is taken false.
    fn said(&self, t: &Taking) -> Said {
        let b = self
            .taken
            .get(&t.prop)
            .copied()
            .unwrap_or_else(|| fused([t], &self.settings));
        let a = self.settings.base_rate;
        match t.value {
            true => Said {
                b: b.belief,
                d: b.disbelief,
                u: b.uncertainty,
                a,
            },
            false => Said {
                b: b.disbelief,
                d: b.belief,
                u: b.uncertainty,
                a: 1.0 - a,
            },
        }
    }

    /// How strong a rest is: the belief that all its takings hold, by
    /// Subjective Logic's multiplication (Jøsang and McAnally, 2005),
    /// which keeps what uncertainty says where a plain product of
    /// beliefs would give 0 for anything only the model says.
    pub fn rest(&self, rest: &BTreeSet<Taking>) -> f64 {
        let one = Said {
            b: 1.0,
            d: 0.0,
            u: 0.0,
            a: 1.0,
        };
        rest.iter().map(|t| self.said(t)).fold(one, times).b
    }

    /// The strongest of these rests, and how strong.
    fn strongest<'r>(
        &self,
        rests: impl IntoIterator<Item = &'r BTreeSet<Taking>>,
    ) -> Option<(&'r BTreeSet<Taking>, f64)> {
        rests
            .into_iter()
            .map(|r| (r, self.rest(r)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// The numbers of `p`, from what judging gave of it: its strongest
    /// rest for, and against. Nothing known: uncertainty 1.
    pub fn prop(&self, p: &Prop) -> Belief {
        let Some(o) = self.explanation.opinions.get(p) else {
            return Belief::of(0.0, 0.0, &self.settings);
        };
        let belief = self.strongest(o.rests_for()).map_or(0.0, |(_, s)| s);
        let disbelief = self.strongest(o.rests_against()).map_or(0.0, |(_, s)| s);
        Belief::of(belief, disbelief, &self.settings)
    }

    /// The strongest rest for `p`, and against.
    pub fn strongest_rests(
        &self,
        p: &Prop,
    ) -> (Option<&'e BTreeSet<Taking>>, Option<&'e BTreeSet<Taking>>) {
        let Some(o) = self.explanation.opinions.get(p) else {
            return (None, None);
        };
        (
            self.strongest(o.rests_for()).map(|(r, _)| r),
            self.strongest(o.rests_against()).map(|(r, _)| r),
        )
    }

    /// The chance a taking is wrong: what is left of the chance its
    /// proposition is as it says, all takings of it fused.
    pub fn taking_doubt(&self, t: &Taking) -> f64 {
        let b = self
            .taken
            .get(&t.prop)
            .copied()
            .unwrap_or_else(|| fused([t], &self.settings));
        let right = match t.value {
            true => b.belief + self.settings.base_rate * b.uncertainty,
            false => b.disbelief + (1.0 - self.settings.base_rate) * b.uncertainty,
        };
        (1.0 - right).clamp(0.0, 1.0)
    }

    /// The chance a suspect is wrong: of a measurement or a review, its
    /// most doubtful taking (what one run says is not independent, so
    /// its takings are not multiplied).
    pub fn doubt(&self, s: &Suspect) -> f64 {
        let takings: Vec<&Taking> = match s {
            Suspect::Taking(t) => vec![t],
            Suspect::Measurement(i) => self.of_source(Source::Measurement(*i)),
            Suspect::Review(i) => self.of_source(Source::Review(*i)),
        };
        takings
            .iter()
            .map(|t| self.taking_doubt(t))
            .fold(0.0, f64::max)
    }

    fn of_source(&self, source: Source) -> Vec<&'e Taking> {
        self.explanation
            .sources
            .iter()
            .filter(|(_, s)| **s == source)
            .map(|(t, _)| t)
            .collect()
    }

    /// The chance every suspect of a diagnosis is wrong.
    pub fn diagnosis_doubt(&self, d: &Diagnosis) -> f64 {
        d.suspects.iter().map(|s| self.doubt(s)).product()
    }

    /// Blame's diagnoses, the most doubtful first; as blame ranked them
    /// where two are as doubtful.
    pub fn rank(&self, blame: &Blame) -> Vec<(Diagnosis, f64)> {
        let mut ranked: Vec<(usize, Diagnosis, f64)> = blame
            .diagnoses
            .iter()
            .enumerate()
            .map(|(i, d)| (i, d.clone(), self.diagnosis_doubt(d)))
            .collect();
        ranked.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
        ranked.into_iter().map(|(_, d, x)| (d, x)).collect()
    }
}
