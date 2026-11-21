//! The state of each piece and how sure it is (`model/verdict.mitra`).
//!
//! The state is found first, by logic alone, from the evidence taken
//! as true, and each state keeps what it was drawn from. Confidence is
//! counted after: how much would have to be given up for a state to
//! fall. It never changes a state.

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

/// A set of assumptions, each by its place in the list, in order.
type Env = Vec<usize>;
/// A piece told true, or told false.
type Fact = (Piece, bool);

/// What starts true: a piece of evidence that counts for something, or
/// a relation that is doubted. `said` is what a piece of evidence
/// says, for a voice to count once.
struct Assumed<'a> {
    piece: &'a Piece,
    weight: f64,
    said: Option<(&'a Voice, &'a Piece, Option<bool>)>,
}

/// One step of a rule: when all of `needs` are so, `gives` is so,
/// under the relation that says it when that relation is doubted.
struct Step {
    needs: Vec<Fact>,
    gives: Fact,
    under: Option<usize>,
}

/// The states with what each was drawn from.
///
/// - `labels`: for each piece told true or told false, the smallest
///   sets of assumptions it follows from, none of which is in
///   contradiction with itself.
/// - `nogoods`: the smallest sets of assumptions that cannot all be
///   true: from each, some piece follows both true and false.
struct Drawn<'a> {
    assumed: Vec<Assumed<'a>>,
    labels: BTreeMap<Fact, Vec<Env>>,
    nogoods: Vec<Env>,
}

fn within(small: &Env, large: &Env) -> bool {
    small.iter().all(|i| large.binary_search(i).is_ok())
}

fn union(x: &Env, y: &Env) -> Env {
    let mut both: Env = x.iter().chain(y).copied().collect();
    both.sort_unstable();
    both.dedup();
    both
}

/// Adds a set to a list of smallest sets. False when one already there
/// is within it.
fn add_smallest(sets: &mut Vec<Env>, new: Env) -> bool {
    if sets.iter().any(|old| within(old, &new)) {
        return false;
    }
    sets.retain(|old| !within(&new, old));
    sets.push(new);
    true
}

/// The steps the relations of a model allow, and what starts true.
/// `model` has its ties written out.
fn steps<'a>(model: &'a Model) -> (Vec<Assumed<'a>>, Vec<Step>, Vec<(Fact, Env)>) {
    let mut assumed: Vec<Assumed> = Vec::new();
    let mut steps: Vec<Step> = Vec::new();
    let mut start: Vec<(Fact, Env)> = Vec::new();
    for e in &model.evidence {
        let weight = weight(model, e);
        if weight <= 0.0 {
            continue;
        }
        start.push(((e.piece.clone(), true), vec![assumed.len()]));
        assumed.push(Assumed {
            piece: &e.piece,
            weight,
            said: Some((&e.by, &e.about, says(e))),
        });
        // One: evidence that is taken says what it says. A review
        // that says yes about it does not say it in its place.
        if let Some(yes) = says(e) {
            start.push(((e.about.clone(), yes), vec![assumed.len() - 1]));
        }
    }
    for said in &model.related {
        let r = &said.content;
        // A relation that is doubted is itself assumed.
        let under = match base(model, &said.piece) < 1.0 {
            true => {
                assumed.push(Assumed {
                    piece: &said.piece,
                    weight: standing(model, &said.piece),
                    said: None,
                });
                Some(assumed.len() - 1)
            }
            false => None,
        };
        let mut step = |needs: Vec<Fact>, gives: Fact| {
            steps.push(Step {
                needs,
                gives,
                under,
            })
        };
        let so = |x: &Piece, value: bool| (x.clone(), value);
        match r.how {
            Relation::All if r.pieces.len() > 1 => {
                let (whole, parts) = (&r.pieces[0], parts(model, r));
                // Two: a true whole makes each part true.
                for part in &parts {
                    step(vec![so(whole, true)], so(part, true));
                }
                // Three: all parts true, or one false.
                step(parts.iter().map(|x| so(x, true)).collect(), so(whole, true));
                for part in &parts {
                    step(vec![so(part, false)], so(whole, false));
                }
                // Four: a false whole with every part true but one.
                for part in &parts {
                    let mut needs = vec![so(whole, false)];
                    let others = parts.iter().filter(|other| *other != part);
                    needs.extend(others.map(|x| so(x, true)));
                    step(needs, so(part, false));
                }
            }
            Relation::Instance if r.pieces.len() > 1 => {
                let (claim, instance) = (&r.pieces[0], &r.pieces[1]);
                // Five: a claim holds at every input.
                step(vec![so(claim, true)], so(instance, true));
                step(vec![so(instance, false)], so(claim, false));
                // Six: with uniform, one input stands for all.
                if let Some(uniform) = r.pieces.get(2) {
                    step(vec![so(instance, true), so(uniform, true)], so(claim, true));
                }
            }
            _ => {}
        }
    }
    (assumed, steps, start)
}

/// Every state with what it was drawn from, once no rule gives
/// anything new.
///
/// After de Kleer, 1986, "An Assumption-based TMS": each state keeps
/// the smallest sets of assumptions it follows from, and a set from
/// which a piece follows both true and false is set aside with
/// everything that only follows from it.
fn drawn(model: &Model) -> Drawn<'_> {
    let (assumed, steps, start) = steps(model);
    let mut labels: BTreeMap<Fact, Vec<Env>> = BTreeMap::new();
    for (fact, env) in start {
        labels.entry(fact).or_default().push(env);
    }
    let mut nogoods: Vec<Env> = Vec::new();
    loop {
        let mut moved = false;
        for step in &steps {
            // Every way to have all that the step needs.
            let mut ways: Vec<Env> = vec![step.under.into_iter().collect()];
            for need in &step.needs {
                let known = labels.get(need).map(Vec::as_slice).unwrap_or(&[]);
                let mut next: Vec<Env> = Vec::new();
                for way in &ways {
                    for env in known {
                        let both = union(way, env);
                        if !nogoods.iter().any(|no| within(no, &both)) {
                            add_smallest(&mut next, both);
                        }
                    }
                }
                ways = next;
            }
            for way in ways {
                moved |= add_smallest(labels.entry(step.gives.clone()).or_default(), way);
            }
        }
        // A set from which a piece follows both true and false.
        let mut found: Vec<Env> = Vec::new();
        for ((piece, value), envs) in &labels {
            if !*value {
                continue;
            }
            let against = labels.get(&(piece.clone(), false));
            for no in against.map(Vec::as_slice).unwrap_or(&[]) {
                for yes in envs {
                    found.push(union(yes, no));
                }
            }
        }
        for new in found {
            if add_smallest(&mut nogoods, new) {
                moved = true;
            }
        }
        for envs in labels.values_mut() {
            envs.retain(|env| !nogoods.iter().any(|no| within(no, env)));
        }
        labels.retain(|_, envs| !envs.is_empty());
        if !moved {
            return Drawn {
                assumed,
                labels,
                nogoods,
            };
        }
    }
}

impl Drawn<'_> {
    fn state(&self, x: &Piece) -> State {
        let told = |value: bool| self.labels.contains_key(&(x.clone(), value));
        match (told(true), told(false)) {
            (false, false) => State::Unknown,
            (true, false) => State::True,
            (false, true) => State::False,
            (true, true) => State::Conflict,
        }
    }

    /// What a set of assumptions counts for together: two pieces of
    /// evidence by one voice that say the same of the same piece count
    /// once, as the stronger.
    fn together(&self, given_up: &[usize]) -> f64 {
        let mut said: Vec<(Option<(&Voice, &Piece, Option<bool>)>, f64)> = Vec::new();
        for a in given_up.iter().map(|i| &self.assumed[*i]) {
            match said
                .iter_mut()
                .find(|(what, _)| a.said.is_some() && *what == a.said)
            {
                Some(found) => found.1 = f64::max(found.1, a.weight),
                None => said.push((a.said, a.weight)),
            }
        }
        said.iter().map(|found| found.1).sum()
    }

    /// The least that must be given up so that none of these sets is
    /// left whole, and what it counts for. Each branch gives up one
    /// member of the first set still whole.
    fn least(&self, sets: &[Env], given_up: &mut Vec<usize>, best: &mut f64) {
        let cost = self.together(given_up);
        if cost >= *best {
            return;
        }
        let whole = sets
            .iter()
            .find(|env| !env.iter().any(|i| given_up.contains(i)));
        match whole {
            None => *best = cost,
            Some(env) => {
                for i in env {
                    given_up.push(*i);
                    self.least(sets, given_up, best);
                    given_up.pop();
                }
            }
        }
    }

    /// Every smallest set that leaves none of these sets whole.
    fn hitting(&self, sets: &[Env]) -> Vec<Env> {
        let mut found: Vec<Env> = Vec::new();
        let mut todo: Vec<Env> = vec![Vec::new()];
        while let Some(given_up) = todo.pop() {
            if found.iter().any(|old| within(old, &given_up)) || found.len() >= 64 {
                continue;
            }
            match sets
                .iter()
                .find(|env| !env.iter().any(|i| given_up.contains(i)))
            {
                None => {
                    add_smallest(&mut found, given_up);
                }
                Some(env) => {
                    for i in env {
                        todo.push(union(&given_up, &vec![*i]));
                    }
                }
            }
        }
        found
    }
}

/// Everything read of a model at once.
// mitra: implements verdict::Reading
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    /// The state of each piece something is said of, and how sure.
    pub pieces: Vec<(Piece, Verdict)>,
    /// What cannot all stand, the least sure first: sets of
    /// assumptions to give up, then the parts of each false whole that
    /// are not known true.
    pub suspects: Vec<Vec<Piece>>,
}

/// The state of every piece once no rule gives anything new, how sure
/// each is, and the suspects.
///
/// - A state is as sure as the least that would have to be given up
///   for it to fall: that much is its count, of yes for a true piece
///   and of no for a false one.
/// - A piece is in conflict when it follows true from some of what is
///   assumed and false from the rest, each without contradiction.
/// - What to give up: the smallest sets that leave no contradiction,
///   the lightest first.
// mitra: implements verdict::read
pub fn read(model: &Model) -> Reading {
    let model = &written(model);
    let drawn = drawn(model);
    let mut names: Vec<&Piece> = drawn.labels.keys().map(|(piece, _)| piece).collect();
    names.dedup();
    let mut pieces: Vec<(Piece, Verdict)> = Vec::new();
    for piece in names {
        let (state, a) = (drawn.state(piece), base(model, piece));
        let count = |value: bool| {
            let mut best = f64::INFINITY;
            let sets = &drawn.labels[&(piece.clone(), value)];
            drawn.least(sets, &mut Vec::new(), &mut best);
            best
        };
        let is_evidence = model.evidence.iter().any(|e| &e.piece == piece);
        let confidence = match state {
            // A piece of evidence is as sure as it stands: its base
            // rate and what says yes about it.
            State::True if is_evidence => {
                let (w, at) = (model.settings.weight, standing(model, piece));
                let r = match at < 1.0 && at > a {
                    true => w * (at - a) / (1.0 - at),
                    false => 0.0,
                };
                Some(Opinion { r, s: 0.0, a })
            }
            State::True => Some(Opinion {
                r: count(true),
                s: 0.0,
                a,
            }),
            State::False => Some(Opinion {
                r: 0.0,
                s: count(false),
                a,
            }),
            _ => None,
        };
        pieces.push((piece.clone(), Verdict { state, confidence }));
    }

    let mut given_up: Vec<(Env, f64)> = match drawn.nogoods.is_empty() {
        true => Vec::new(),
        false => {
            let sets = drawn.hitting(&drawn.nogoods);
            sets.into_iter()
                .map(|set| {
                    let cost = drawn.together(&set);
                    (set, cost)
                })
                .collect()
        }
    };
    given_up.sort_by(|x, y| x.1.total_cmp(&y.1).then(x.0.cmp(&y.0)));
    let mut suspects: Vec<Vec<Piece>> = given_up
        .iter()
        .map(|(set, _)| {
            set.iter()
                .map(|i| drawn.assumed[*i].piece.clone())
                .collect()
        })
        .collect();
    let wholes: Vec<&crate::model::Related> = model
        .related
        .iter()
        .map(|said| &said.content)
        .filter(|r| r.how == Relation::All && r.pieces.len() > 1)
        .collect();
    let known_true = |x: &Piece| drawn.labels.contains_key(&(x.clone(), true));
    for r in &wholes {
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
        if drawn.state(&r.pieces[0]) == State::False && open.len() > 1 {
            open.sort_by(|x, y| base(model, x).total_cmp(&base(model, y)).then(x.cmp(y)));
            if !suspects.contains(&open) {
                suspects.push(open);
            }
        }
    }
    Reading { pieces, suspects }
}

/// The state of every piece something is said of.
// mitra: implements verdict::spread
pub fn spread(model: &Model) -> Vec<(Piece, State)> {
    let model = &written(model);
    let drawn = drawn(model);
    let mut names: Vec<&Piece> = drawn.labels.keys().map(|(piece, _)| piece).collect();
    names.dedup();
    names
        .into_iter()
        .map(|piece| (piece.clone(), drawn.state(piece)))
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

/// How sure a verdict that holds is: its least sure statement, and
/// which one it is. Nothing unless every statement is true.
// mitra: implements verdict::least_sure
pub fn least_sure(model: &Model) -> Option<(Piece, f64)> {
    if overall(model) != State::True {
        return None;
    }
    let reading = read(model);
    let w = model.settings.weight;
    let mut least: Option<(Piece, f64)> = None;
    for x in statements(model) {
        let found = reading.pieces.iter().find(|(piece, _)| piece == &x)?;
        let n = sure(&found.1, w)?;
        if least.as_ref().is_none_or(|(_, m)| n < *m) {
            least = Some((x, n));
        }
    }
    least
}
