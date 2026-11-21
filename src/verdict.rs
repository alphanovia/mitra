//! The state of each piece and how sure it is (`model/verdict.mitra`).
//!
//! The state is found first, by logic alone, from the evidence taken
//! as true. Confidence is counted after: how much evidence would have
//! to be wrong for a state to fall. It never changes a state.

use crate::model::{
    base, parts, reliable, written, Evidence, Model, Opinion, Piece, Relation, Voice,
};
use std::collections::BTreeMap;

/// What is known of a piece: nothing, true, false, or both at once.
// mitra: implements verdict::State
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unknown,
    True,
    False,
    Conflict,
}

/// A state and how sure it is. No confidence for a piece that is
/// unknown or in conflict.
// mitra: implements verdict::Verdict
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Verdict {
    pub state: State,
    pub confidence: Option<Opinion>,
}

/// Both said of one piece: the same if they agree, the other if one is
/// unknown, conflict otherwise.
// mitra: implements verdict::joined
pub fn joined(x: State, y: State) -> State {
    match (x, y) {
        (State::Unknown, other) | (other, State::Unknown) => other,
        (a, b) if a == b => a,
        _ => State::Conflict,
    }
}

/// What a piece of evidence says: yes, no, or neither.
// mitra: implements verdict::says
pub fn says(e: &Evidence) -> Option<bool> {
    match (e.value.r > e.value.s, e.value.s > e.value.r) {
        (true, _) => Some(true),
        (_, true) => Some(false),
        _ => None,
    }
}

/// How much a piece of evidence counts: what it says, as far as its
/// piece stands and its voice is believed before anything.
// mitra: implements verdict::weight
pub fn weight(model: &Model, e: &Evidence) -> f64 {
    counted(model, e, &mut Vec::new())
}

/// How far a piece of evidence stands: its base rate, moved by the
/// evidence that says yes about it, each voice once. A review of a
/// test is what makes the test count for more.
// mitra: implements verdict::standing
pub fn standing(model: &Model, x: &Piece) -> f64 {
    stands(model, x, &mut Vec::new())
}

fn counted(model: &Model, e: &Evidence, seen: &mut Vec<Piece>) -> f64 {
    let voice = base(model, &reliable(&e.by).piece);
    f64::max(e.value.r, e.value.s) * stands(model, &e.piece, seen) * voice
}

fn stands(model: &Model, x: &Piece, seen: &mut Vec<Piece>) -> f64 {
    let (a, w) = (base(model, x), model.settings.weight);
    if seen.contains(x) {
        return a;
    }
    seen.push(x.clone());
    let mut by_voice: Vec<(&Voice, f64)> = Vec::new();
    for e in model.evidence.iter().filter(|e| &e.about == x) {
        if says(e) != Some(true) {
            continue;
        }
        let count = counted(model, e, seen);
        match by_voice.iter_mut().find(|(by, _)| *by == &e.by) {
            Some(found) => found.1 = f64::max(found.1, count),
            None => by_voice.push((&e.by, count)),
        }
    }
    seen.pop();
    let r: f64 = by_voice.iter().map(|found| found.1).sum();
    match r + w > 0.0 {
        true => (r + a * w) / (r + w),
        false => a,
    }
}

/// Told true, told false.
type Told = (bool, bool);

fn state_of(told: Told) -> State {
    match told {
        (false, false) => State::Unknown,
        (true, false) => State::True,
        (false, true) => State::False,
        (true, true) => State::Conflict,
    }
}

/// The states once no rule gives anything new, with only the evidence
/// marked `live` taken as true. `model` has its ties written out.
fn spread_from(model: &Model, live: &[bool]) -> BTreeMap<Piece, Told> {
    let mut told: BTreeMap<Piece, Told> = BTreeMap::new();
    let get = |told: &BTreeMap<Piece, Told>, x: &Piece| told.get(x).copied().unwrap_or_default();
    let set = |told: &mut BTreeMap<Piece, Told>, x: &Piece, value: bool| -> bool {
        let entry = told.entry(x.clone()).or_default();
        let slot = if value { &mut entry.0 } else { &mut entry.1 };
        let new = !*slot;
        *slot = true;
        new
    };
    let wholes: Vec<(&Piece, Vec<Piece>)> = model
        .related
        .iter()
        .filter(|said| said.content.how == Relation::All && said.content.pieces.len() > 1)
        .map(|said| (&said.content.pieces[0], parts(model, &said.content)))
        .collect();
    for (e, _) in model.evidence.iter().zip(live).filter(|(_, live)| **live) {
        set(&mut told, &e.piece, true);
    }
    loop {
        let mut moved = false;
        // One: evidence that is true says what it says.
        for (e, _) in model.evidence.iter().zip(live).filter(|(_, live)| **live) {
            if let (true, Some(yes)) = (get(&told, &e.piece).0, says(e)) {
                moved |= set(&mut told, &e.about, yes);
            }
        }
        for (whole, parts) in &wholes {
            let w = get(&told, whole);
            // Two: a true whole makes each part true.
            if w.0 {
                for part in parts {
                    moved |= set(&mut told, part, true);
                }
            }
            // Three: all parts true, or one false.
            if parts.iter().all(|part| get(&told, part).0) {
                moved |= set(&mut told, whole, true);
            }
            if parts.iter().any(|part| get(&told, part).1) {
                moved |= set(&mut told, whole, false);
            }
            // Four: a false whole with every part true but one.
            if w.1 {
                for part in parts {
                    let others = parts.iter().filter(|other| *other != part);
                    if others.clone().all(|other| get(&told, other).0) {
                        moved |= set(&mut told, part, false);
                    }
                }
            }
        }
        for said in model
            .related
            .iter()
            .filter(|said| said.content.how == Relation::Instance)
        {
            let (claim, instance) = (&said.content.pieces[0], &said.content.pieces[1]);
            // Five: a claim holds at every input.
            if get(&told, claim).0 {
                moved |= set(&mut told, instance, true);
            }
            if get(&told, instance).1 {
                moved |= set(&mut told, claim, false);
            }
            // Six: with uniform, one input stands for all.
            if let Some(uniform) = said.content.pieces.get(2) {
                if get(&told, instance).0 && get(&told, uniform).0 {
                    moved |= set(&mut told, claim, true);
                }
            }
        }
        if !moved {
            return told;
        }
    }
}

/// What a set of evidence counts for together: two pieces of evidence
/// by one voice that say the same of the same piece count once, as the
/// stronger.
fn together(model: &Model, given_up: &[usize]) -> f64 {
    let mut said: Vec<(&Voice, &Piece, Option<bool>, f64)> = Vec::new();
    for e in given_up.iter().map(|i| &model.evidence[*i]) {
        let w = weight(model, e);
        let same = said
            .iter_mut()
            .find(|(by, about, sign, _)| (*by, *about, *sign) == (&e.by, &e.about, says(e)));
        match same {
            Some(found) => found.3 = f64::max(found.3, w),
            None => said.push((&e.by, &e.about, says(e), w)),
        }
    }
    said.iter().map(|found| found.3).sum()
}

/// Every set of evidence that may be given up: all of them for a small
/// model, and those of three pieces or fewer beyond.
fn sets(n: usize) -> Vec<Vec<usize>> {
    if n <= 16 {
        return (0..1usize << n)
            .map(|mask| (0..n).filter(|i| mask >> i & 1 == 1).collect())
            .collect();
    }
    let mut found = vec![Vec::new()];
    for i in 0..n {
        found.push(vec![i]);
        for j in i + 1..n {
            found.push(vec![i, j]);
            for k in j + 1..n {
                found.push(vec![i, j, k]);
            }
        }
    }
    found
}

/// Everything read of a model at once.
// mitra: implements verdict::Reading
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    /// The state of each piece something is said of, and how sure.
    pub pieces: Vec<(Piece, Verdict)>,
    /// What cannot all stand, the least sure first: sets of evidence
    /// to give up, then the parts of each false whole that are not
    /// known true.
    pub suspects: Vec<Vec<Piece>>,
}

/// The state of every piece once no rule gives anything new, how sure
/// each is, and the suspects.
///
/// - A state is as sure as the least evidence that would have to be
///   given up for it to fall: that much is its count, of yes for a
///   true piece and of no for a false one.
/// - A conflict is removed by giving up evidence: the smallest sets
///   that leave no conflict are listed, the lightest first.
// mitra: implements verdict::read
pub fn read(model: &Model) -> Reading {
    let model = &written(model);
    let n = model.evidence.len();
    let taken = taken(model);
    let full = spread_from(model, &taken);
    let in_conflict = |told: &BTreeMap<Piece, Told>| told.values().any(|t| *t == (true, true));
    let mut cut: BTreeMap<&Piece, f64> = BTreeMap::new();
    let mut clean: Vec<(Vec<usize>, f64)> = Vec::new();
    for given_up in sets(n) {
        if given_up.is_empty() {
            continue;
        }
        if given_up.iter().any(|i| !taken[*i]) {
            continue;
        }
        let mut live = taken.clone();
        for i in &given_up {
            live[*i] = false;
        }
        let left = spread_from(model, &live);
        let cost = together(model, &given_up);
        for (piece, told) in &full {
            let now = left.get(piece).copied().unwrap_or_default();
            if (told.0 && !now.0) || (told.1 && !now.1) {
                let least = cut.entry(piece).or_insert(f64::INFINITY);
                *least = f64::min(*least, cost);
            }
        }
        if in_conflict(&full) && !in_conflict(&left) {
            clean.push((given_up, cost));
        }
    }
    let pieces = full
        .iter()
        .map(|(piece, told)| {
            let a = base(model, piece);
            let count = cut.get(piece).copied().filter(|c| c.is_finite());
            // A piece of evidence is as sure as it stands: its base
            // rate and what says yes about it.
            if let Some(i) = model.evidence.iter().position(|e| &e.piece == piece) {
                let (w, at) = (model.settings.weight, standing(model, piece));
                let r = match at < 1.0 && at > a {
                    true => w * (at - a) / (1.0 - at),
                    false => 0.0,
                };
                let confidence = match (state_of(*told), taken[i]) {
                    (State::True, true) => Some(Opinion { r, s: 0.0, a }),
                    _ => None,
                };
                let state = state_of(*told);
                return (piece.clone(), Verdict { state, confidence });
            }
            let confidence = match (state_of(*told), count) {
                (State::True, Some(r)) => Some(Opinion { r, s: 0.0, a }),
                (State::False, Some(s)) => Some(Opinion { r: 0.0, s, a }),
                _ => None,
            };
            let state = state_of(*told);
            (piece.clone(), Verdict { state, confidence })
        })
        .collect();

    let smallest: Vec<&(Vec<usize>, f64)> = clean
        .iter()
        .filter(|(set, _)| {
            !clean
                .iter()
                .any(|(other, _)| other.len() < set.len() && other.iter().all(|i| set.contains(i)))
        })
        .collect();
    let mut ranked = smallest;
    ranked.sort_by(|x, y| x.1.total_cmp(&y.1));
    let mut suspects: Vec<Vec<Piece>> = ranked
        .iter()
        .map(|(set, _)| {
            set.iter()
                .map(|i| model.evidence[*i].piece.clone())
                .collect()
        })
        .collect();
    let wholes: Vec<&crate::model::Related> = model
        .related
        .iter()
        .map(|said| &said.content)
        .filter(|r| r.how == Relation::All && r.pieces.len() > 1)
        .collect();
    let known_true = |x: &Piece| full.get(x).copied().unwrap_or_default().0;
    for r in &wholes {
        let whole = full.get(&r.pieces[0]).copied().unwrap_or_default();
        let mut open: Vec<Piece> = Vec::new();
        let mut todo: Vec<Piece> = parts(model, r);
        // A suspect that is itself a whole stands for its parts.
        while let Some(part) = todo.pop() {
            if known_true(&part) || open.contains(&part) {
                continue;
            }
            match wholes.iter().find(|inner| inner.pieces[0] == part) {
                Some(inner) => todo.extend(parts(model, inner)),
                None => open.push(part),
            }
        }
        if whole == (false, true) && open.len() > 1 {
            open.sort_by(|x, y| base(model, x).total_cmp(&base(model, y)).then(x.cmp(y)));
            if !suspects.contains(&open) {
                suspects.push(open);
            }
        }
    }
    Reading { pieces, suspects }
}

/// The evidence that is taken as true: all of it but what counts for
/// nothing.
fn taken(model: &Model) -> Vec<bool> {
    let each = model.evidence.iter().map(|e| weight(model, e) > 0.0);
    each.collect()
}

/// The state of every piece something is said of.
// mitra: implements verdict::spread
pub fn spread(model: &Model) -> Vec<(Piece, State)> {
    let model = &written(model);
    let told = spread_from(model, &taken(model));
    told.into_iter()
        .map(|(piece, told)| (piece, state_of(told)))
        .collect()
}

/// The state of one piece.
// mitra: implements verdict::state
pub fn state(model: &Model, x: &Piece) -> State {
    let found = spread(model).into_iter().find(|(piece, _)| piece == x);
    found.map_or(State::Unknown, |(_, state)| state)
}

/// The pieces a verdict is read from: every piece a relation, a tie or
/// a piece of evidence speaks of, that is not itself evidence, a
/// relation, or the reliability of a voice.
// mitra: implements verdict::statements
pub fn statements(model: &Model) -> Vec<Piece> {
    let model = &written(model);
    let mut found: Vec<Piece> = Vec::new();
    let mut add = |x: &Piece| {
        let apart = model
            .evidence
            .iter()
            .any(|e| &e.piece == x || &reliable(&e.by).piece == x)
            || model.related.iter().any(|said| &said.piece == x);
        if !apart && !found.contains(x) {
            found.push(x.clone());
        }
    };
    for e in &model.evidence {
        add(&e.about);
    }
    for said in &model.related {
        for piece in parts(model, &said.content) {
            add(&piece);
        }
        add(&said.content.pieces[0]);
    }
    found
}

/// The verdict of a model: conflict if any piece is in conflict; else
/// false if a statement is false; else unknown if one is unknown; else
/// true.
// mitra: implements verdict::overall
pub fn overall(model: &Model) -> State {
    let states = spread(model);
    let of = |x: &Piece| {
        let found = states.iter().find(|(piece, _)| piece == x);
        found.map_or(State::Unknown, |(_, state)| *state)
    };
    if states.iter().any(|(_, state)| *state == State::Conflict) {
        return State::Conflict;
    }
    let all: Vec<State> = statements(model).iter().map(of).collect();
    for worst in [State::Conflict, State::False, State::Unknown] {
        if all.contains(&worst) {
            return worst;
        }
    }
    match all.is_empty() {
        true => State::Unknown,
        false => State::True,
    }
}

/// The state of one piece and how sure it is.
// mitra: implements verdict::verdict_of
pub fn verdict_of(model: &Model, x: &Piece) -> Verdict {
    let found = read(model).pieces.into_iter().find(|(piece, _)| piece == x);
    found.map_or(
        Verdict {
            state: State::Unknown,
            confidence: None,
        },
        |(_, verdict)| verdict,
    )
}

/// The suspects of a model, the least sure first.
// mitra: implements verdict::suspects
pub fn suspects(model: &Model) -> Vec<Vec<Piece>> {
    read(model).suspects
}

/// How sure a state is, as one number: the reading of its counts for
/// a true piece, and of the counts against for a false one. Nothing
/// for a piece that is unknown or in conflict.
// mitra: implements verdict::sure
pub fn sure(v: &Verdict, w: f64) -> Option<f64> {
    let o = v.confidence?;
    match v.state {
        State::True => Some((o.r + o.a * w) / (o.r + w)),
        State::False => Some((o.s + (1.0 - o.a) * w) / (o.s + w)),
        _ => None,
    }
}

/// A verdict in words: "true 0.91", "false 0.85", "unknown",
/// "conflict".
pub fn shown(v: &Verdict, w: f64) -> String {
    let state = format!("{:?}", v.state).to_lowercase();
    match sure(v, w) {
        Some(n) => format!("{state} {n:.2}"),
        None => state,
    }
}
