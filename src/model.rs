//! The piece system (`model/model.mitra`): a piece, its opinion, and
//! the calculus that judges every piece of a model from what was
//! taken. It knows nothing of values.

use std::collections::BTreeMap;

/// An address in the real system: a file, then what is in it. It is
/// not right or wrong.
// mitra: implements location::Location
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub path: Vec<String>,
}

impl Location {
    pub fn at(path: &[&str]) -> Location {
        Location {
            path: path.iter().map(|p| p.to_string()).collect(),
        }
    }
}

/// `a` is within `b`: its path goes on from the other's.
// mitra: implements location::within
pub fn within(a: &Location, b: &Location) -> bool {
    a.path.starts_with(&b.path)
}

/// The step of logic a piece is: a deduction holds by what it is, an
/// induction may be wrong.
// mitra: implements piece::Inference
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inference {
    Deduction,
    Induction,
}

/// What a piece can be about: its type gives the keyword, the word
/// the language names that kind of piece by.
pub trait Keyed {
    fn keyword(&self) -> String;
}

/// A keyword is its own.
impl Keyed for String {
    fn keyword(&self) -> String {
        self.clone()
    }
}

/// A proposed property, known by its name: given the pieces before
/// it, what it is about gives the pieces after it. `T` is what it is
/// about, and the type of `T` gives its keyword. The pieces before
/// and after are of any kind.
// mitra: implements piece::Piece
#[derive(Clone, Debug, PartialEq)]
pub struct Piece<T> {
    pub name: String,
    pub pre: Vec<SomePiece>,
    pub about: T,
    pub post: Vec<SomePiece>,
}

/// Some piece, of any kind: what it is about is forgotten, and its
/// keyword is kept. The calculus reads pieces in this form only.
pub type SomePiece = Piece<String>;

/// The piece with what it is about forgotten.
// mitra: implements piece::erased
pub fn erased<T: Keyed>(x: &Piece<T>) -> SomePiece {
    Piece {
        name: x.name.clone(),
        pre: x.pre.clone(),
        about: x.about.keyword(),
        post: x.post.clone(),
    }
}

/// A piece with nothing before and nothing after.
// mitra: implements piece::leaf
pub fn leaf<T>(name: &str, about: T) -> Piece<T> {
    Piece {
        name: name.to_string(),
        pre: Vec::new(),
        about,
        post: Vec::new(),
    }
}

/// What a part needs: a piece about a part that cannot be evaluated.
/// It is taken, or it holds, or it is refuted.
// mitra: implements piece::Require
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Require {
    pub part: String,
    pub text: String,
}

/// What a part promises: a piece about a part that cannot be
/// evaluated.
// mitra: implements piece::Ensure
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ensure {
    pub part: String,
    pub text: String,
}

/// A believed link: `of`, a part, a record or a function of the
/// model, is at `location` in the real system, and the model says all
/// that matters of it there. Identification and completeness are one
/// belief.
// mitra: implements piece::Tie
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tie {
    pub of: String,
    pub location: Location,
}

impl Keyed for Require {
    fn keyword(&self) -> String {
        "require".to_string()
    }
}
impl Keyed for Ensure {
    fn keyword(&self) -> String {
        "ensure".to_string()
    }
}
impl Keyed for Tie {
    fn keyword(&self) -> String {
        "tie".to_string()
    }
}

/// What is known of a piece: nothing, true, false, or both at once.
// mitra: implements piece::State
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unknown,
    True,
    False,
    Conflict,
}

/// Both said of one piece.
// mitra: implements piece::joined
pub fn joined(x: State, y: State) -> State {
    match (x, y) {
        (State::Unknown, other) | (other, State::Unknown) => other,
        (a, b) if a == b => a,
        _ => State::Conflict,
    }
}

fn of(value: bool) -> State {
    match value {
        true => State::True,
        false => State::False,
    }
}

/// What is believed of a piece. Read it through its getters: the
/// inside is free to change.
///
/// Inside, nothing is a state. There is what counts for the piece and
/// what counts against it, which never cancel; the sets of pieces
/// each of those rests on; and the bias. The state is read from
/// which sets are empty, never from the numbers.
// mitra: implements piece::Opinion
#[derive(Clone, Debug, PartialEq)]
pub struct Opinion {
    evaluated: State,
    taken: State,
    /// What counts for it, and against it.
    yes: f64,
    no: f64,
    /// The smallest sets of pieces it follows true from, and false
    /// from.
    rests_for: Vec<Vec<SomePiece>>,
    rests_against: Vec<Vec<SomePiece>>,
    /// The ways out of a conflict elsewhere that leave it with
    /// nothing to rest on.
    falls: Vec<Vec<SomePiece>>,
    bias: f64,
    /// How many counts the bias weighs.
    weight: f64,
}

impl Opinion {
    fn of(bias: f64, weight: f64) -> Opinion {
        Opinion {
            evaluated: State::Unknown,
            taken: State::Unknown,
            yes: 0.0,
            no: 0.0,
            rests_for: Vec::new(),
            rests_against: Vec::new(),
            falls: Vec::new(),
            bias,
            weight,
        }
    }
    /// What computing it gave.
    // mitra: implements piece::evaluated
    pub fn evaluated(&self) -> State {
        self.evaluated
    }
    /// What voices took it as.
    // mitra: implements piece::taken
    pub fn taken(&self) -> State {
        self.taken
    }
    /// What the calculus judges it to be, read from what it rests on:
    /// unknown with nothing for and nothing against; true, it holds,
    /// with something for and nothing against; false, it is refuted,
    /// the other way; conflict with both, or when a way out of a
    /// conflict elsewhere leaves it with nothing to rest on.
    // mitra: implements piece::judged
    pub fn judged(&self) -> State {
        let contested = !self.falls.is_empty();
        match (self.rests_for.is_empty(), self.rests_against.is_empty()) {
            (true, true) => State::Unknown,
            (false, true) if !contested => State::True,
            (true, false) if !contested => State::False,
            _ => State::Conflict,
        }
    }
    /// How sure the judged state is. None when unknown or in conflict.
    // mitra: implements piece::confidence
    pub fn confidence(&self) -> Option<f64> {
        let (a, w) = (self.bias, self.weight);
        match self.judged() {
            State::True => Some((self.yes + a * w) / (self.yes + w)),
            State::False => Some((self.no + (1.0 - a) * w) / (self.no + w)),
            _ => None,
        }
    }
    /// What to expect of it before anything.
    // mitra: implements piece::bias
    pub fn bias(&self) -> f64 {
        self.bias
    }
    /// The smallest sets of pieces the judged state rests on. Nothing
    /// for a piece that is unknown or in conflict.
    // mitra: implements piece::rests
    pub fn rests(&self) -> &[Vec<SomePiece>] {
        match self.judged() {
            State::True => &self.rests_for,
            State::False => &self.rests_against,
            _ => &[],
        }
    }
    /// A trial: how far the piece is believed, as an interval, for
    /// any state. The low end is what says it is true, the high end
    /// is all that does not say it is false. Unknown is 0 to 1.
    // mitra: implements piece::interval
    pub fn interval(&self) -> (f64, f64) {
        let all = self.yes + self.no + self.weight;
        (self.yes / all, (self.yes + self.weight) / all)
    }
}

/// Who takes.
// mitra: implements model::Voice
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Voice {
    pub name: String,
}

impl Voice {
    pub fn named(name: &str) -> Voice {
        Voice {
            name: name.to_string(),
        }
    }
}

/// How a voice came to take a piece. The calculus reads the three
/// the same way.
///
/// - `Review`: it read the model, the code, or both.
/// - `Measurement`: it ran the system at a point.
/// - `Evaluation`: it read a claim over a set, with no run.
// mitra: implements model::Form
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Review,
    Measurement,
    Evaluation,
}

/// A voice takes a piece as true or as false. `count` is how firmly:
/// one test is 1.
// mitra: implements model::Taking
#[derive(Clone, Debug, PartialEq)]
pub struct Taking {
    pub piece: SomePiece,
    pub value: bool,
    pub count: f64,
    pub by: Voice,
    pub form: Form,
}

/// - `weight`: how many counts a bias weighs.
/// - `bias`: what a piece nobody gave a bias is expected to be.
/// - `reliability`: how far a voice nobody judged is believed.
// mitra: implements model::Settings
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub weight: f64,
    pub bias: f64,
    pub reliability: f64,
}

/// - `pieces`: what is proposed. The pieces before and after a piece
///   are in the model with it.
/// - `evaluated`: what computing a piece gave.
/// - `taken`: what voices took.
// mitra: implements model::Model
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub pieces: Vec<SomePiece>,
    pub evaluated: Vec<(SomePiece, bool)>,
    pub taken: Vec<Taking>,
    pub bias: Vec<(SomePiece, f64)>,
    pub voices: Vec<(Voice, f64)>,
    pub settings: Settings,
}

impl Model {
    pub fn empty(settings: Settings) -> Model {
        Model {
            pieces: Vec::new(),
            evaluated: Vec::new(),
            taken: Vec::new(),
            bias: Vec::new(),
            voices: Vec::new(),
            settings,
        }
    }
}

/// Every piece of the model, each once: those listed, those taken or
/// evaluated, and the pieces before and after each.
// mitra: implements model::pieces
pub fn pieces(model: &Model) -> Vec<SomePiece> {
    fn collect(found: &mut Vec<SomePiece>, x: &SomePiece) {
        if found.iter().any(|old| old.name == x.name) {
            return;
        }
        found.push(x.clone());
        for inner in x.pre.iter().chain(&x.post) {
            collect(found, inner);
        }
    }
    let mut found = Vec::new();
    let taken = model.taken.iter().map(|t| &t.piece);
    let evaluated = model.evaluated.iter().map(|(x, _)| x);
    for x in &model.pieces {
        collect(&mut found, x);
    }
    for x in taken.chain(evaluated) {
        collect(&mut found, x);
    }
    found
}

/// What to expect of a piece before anything.
// mitra: implements model::bias
pub fn bias<T>(model: &Model, x: &Piece<T>) -> f64 {
    let found = model.bias.iter().find(|(piece, _)| piece.name == x.name);
    found.map_or(model.settings.bias, |(_, a)| *a)
}

/// How much a taking weighs: its count, as far as its voice is
/// believed.
// mitra: implements model::weight
pub fn weight(model: &Model, t: &Taking) -> f64 {
    let voice = model.voices.iter().find(|(by, _)| by == &t.by);
    t.count * voice.map_or(model.settings.reliability, |(_, a)| *a)
}

/// The step of logic a piece is, read from its bias: with a bias of 1
/// it is a deduction, under 1 an induction. None for a leaf.
// mitra: implements model::step
pub fn step<T>(model: &Model, x: &Piece<T>) -> Option<Inference> {
    match (x.post.is_empty(), bias(model, x) >= 1.0) {
        (true, _) => None,
        (false, true) => Some(Inference::Deduction),
        (false, false) => Some(Inference::Induction),
    }
}

/// A deduction holds by what it is: it needs no state.
fn given<T>(model: &Model, x: &Piece<T>) -> bool {
    step(model, x) == Some(Inference::Deduction)
}

/// What can be given up: a taking, or the doubt of an induction.
struct Assumed {
    piece: SomePiece,
    weight: f64,
    /// For a taking: who took, and as what.
    said: Option<(Voice, bool)>,
}

/// A set of takings, each by its place in the list, in order.
type Env = Vec<usize>;
/// A piece told true, or told false, by its name.
type Fact = (String, bool);

/// When all of `needs` are so, `gives` is so.
struct Step {
    needs: Vec<Fact>,
    gives: Fact,
    /// The doubt of the induction this step applies.
    under: Option<usize>,
}

/// The states with what each was drawn from.
///
/// - `labels`: for each piece told true or told false, the smallest
///   sets of what is assumed that it follows from, none in
///   contradiction with itself.
/// - `nogoods`: the smallest sets that cannot all be true.
struct Drawn {
    assumed: Vec<Assumed>,
    labels: BTreeMap<Fact, Vec<Env>>,
    nogoods: Vec<Env>,
    /// The smallest sets of takings to give up so that what is left
    /// can all be true. One empty set when nothing is contested.
    ways: Vec<Env>,
}

fn inside(small: &Env, large: &Env) -> bool {
    small.iter().all(|i| large.binary_search(i).is_ok())
}

fn union(x: &Env, y: &Env) -> Env {
    let mut both: Env = x.iter().chain(y).copied().collect();
    both.sort_unstable();
    both.dedup();
    both
}

/// Adds a set to a list of smallest sets. False when one already there
/// is inside it.
fn add_smallest(sets: &mut Vec<Env>, new: Env) -> bool {
    if sets.iter().any(|old| inside(old, &new)) {
        return false;
    }
    sets.retain(|old| !inside(&new, old));
    sets.push(new);
    true
}

/// The steps a piece allows. Forward: the piece and all before it
/// true, each piece after is true. Backward: a piece after false, and
/// all the others true, the one left is false. And the piece itself
/// is true when every piece after it is. A piece that is given needs
/// no state.
fn steps(model: &Model, all: &[SomePiece], assumed: &mut Vec<Assumed>) -> Vec<Step> {
    let mut steps = Vec::new();
    for t in all.iter().filter(|t| !t.post.is_empty()) {
        let so = |x: &SomePiece, value: bool| (x.name.clone(), value);
        let mut holds: Vec<&SomePiece> = t.pre.iter().collect();
        // An induction is applied under its own doubt, which weighs
        // its bias.
        let mut doubt = None;
        if !given(model, t) {
            holds.push(t);
            doubt = Some(assumed.len());
            assumed.push(Assumed {
                piece: t.clone(),
                weight: bias(model, t),
                said: None,
            });
            // A piece whose every piece after is true holds, whatever
            // is before it.
            steps.push(Step {
                needs: t.post.iter().map(|q| so(q, true)).collect(),
                gives: so(t, true),
                under: None,
            });
        }
        for q in &t.post {
            let needs = holds.iter().map(|x| so(x, true)).collect();
            steps.push(Step {
                needs,
                gives: so(q, true),
                under: doubt,
            });
            for one in &holds {
                let mut needs = vec![so(q, false)];
                let others = holds.iter().filter(|x| x.name != one.name);
                needs.extend(others.map(|x| so(x, true)));
                // Refuting the piece itself does not apply it.
                let under = doubt.filter(|_| one.name != t.name);
                steps.push(Step {
                    needs,
                    gives: so(one, false),
                    under,
                });
            }
        }
    }
    steps
}

/// Every state with what it was drawn from, once no step gives
/// anything new.
///
/// After de Kleer, 1986, "An Assumption-based TMS".
fn drawn(model: &Model, all: &[SomePiece]) -> Drawn {
    let mut assumed: Vec<Assumed> = Vec::new();
    let mut labels: BTreeMap<Fact, Vec<Env>> = BTreeMap::new();
    for t in &model.taken {
        // A taking that weighs nothing is not taken at all.
        if weight(model, t) > 0.0 {
            let fact = (t.piece.name.clone(), t.value);
            labels.entry(fact).or_default().push(vec![assumed.len()]);
            assumed.push(Assumed {
                piece: t.piece.clone(),
                weight: weight(model, t),
                said: Some((t.by.clone(), t.value)),
            });
        }
    }
    let steps = steps(model, all, &mut assumed);
    let mut nogoods: Vec<Env> = Vec::new();
    loop {
        let mut moved = false;
        for step in &steps {
            let mut ways: Vec<Env> = vec![step.under.into_iter().collect()];
            for need in &step.needs {
                let known = labels.get(need).map(Vec::as_slice).unwrap_or(&[]);
                let mut next: Vec<Env> = Vec::new();
                for way in &ways {
                    for env in known {
                        let both = union(way, env);
                        if !nogoods.iter().any(|no| inside(no, &both)) {
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
        let mut found: Vec<Env> = Vec::new();
        for ((name, value), envs) in &labels {
            if !*value {
                continue;
            }
            let against = labels.get(&(name.clone(), false));
            for no in against.map(Vec::as_slice).unwrap_or(&[]) {
                for yes in envs {
                    found.push(union(yes, no));
                }
            }
        }
        for new in found {
            moved |= add_smallest(&mut nogoods, new);
        }
        for envs in labels.values_mut() {
            envs.retain(|env| !nogoods.iter().any(|no| inside(no, env)));
        }
        labels.retain(|_, envs| !envs.is_empty());
        if !moved {
            let mut all = Drawn {
                assumed,
                labels,
                nogoods,
                ways: vec![Vec::new()],
            };
            if !all.nogoods.is_empty() {
                all.ways = all.hitting(&all.nogoods);
            }
            return all;
        }
    }
}

impl Drawn {
    fn told(&self, x: &SomePiece, value: bool) -> bool {
        self.labels.contains_key(&(x.name.clone(), value))
    }

    /// The state of a piece. Where some takings cannot all be true,
    /// there are several ways to give up the least: a piece is true
    /// only when it is told true whichever is chosen and never told
    /// false, and a piece that some of the ways leave without its
    /// state is in conflict with the pieces it rests on.
    fn state(&self, x: &SomePiece) -> State {
        let sets = |value: bool| {
            let found = self.labels.get(&(x.name.clone(), value));
            found.map(Vec::as_slice).unwrap_or(&[])
        };
        let always = |value: bool| {
            let stands = |given_up: &Env| {
                let mut left = sets(value).iter();
                left.any(|env| !env.iter().any(|i| given_up.contains(i)))
            };
            !sets(value).is_empty() && self.ways.iter().all(stands)
        };
        match (!sets(true).is_empty(), !sets(false).is_empty()) {
            (false, false) => State::Unknown,
            (true, false) if always(true) => State::True,
            (false, true) if always(false) => State::False,
            _ => State::Conflict,
        }
    }

    /// The ways out of a conflict that leave this side of a piece
    /// with nothing to rest on. None where nothing is contested.
    fn falls(&self, x: &SomePiece, value: bool) -> Vec<Env> {
        let found = self.labels.get(&(x.name.clone(), value));
        let sets = found.map(Vec::as_slice).unwrap_or(&[]);
        let stands = |given_up: &Env| {
            let mut left = sets.iter();
            left.any(|env| !env.iter().any(|i| given_up.contains(i)))
        };
        self.ways
            .iter()
            .filter(|way| !stands(way))
            .cloned()
            .collect()
    }

    /// What a set weighs together: one voice that takes one piece
    /// the same way twice counts once, as the firmer.
    fn together(&self, given_up: &[usize]) -> f64 {
        let mut said: Vec<(&str, &Option<(Voice, bool)>, f64)> = Vec::new();
        for a in given_up.iter().map(|i| &self.assumed[*i]) {
            let same = said
                .iter_mut()
                .find(|old| a.said.is_some() && old.0 == a.piece.name && old.1 == &a.said);
            match same {
                Some(found) => found.2 = f64::max(found.2, a.weight),
                None => said.push((&a.piece.name, &a.said, a.weight)),
            }
        }
        said.iter().map(|found| found.2).sum()
    }

    /// The least that must be given up so that none of these sets is
    /// left whole.
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
            if found.iter().any(|old| inside(old, &given_up)) || found.len() >= 64 {
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

/// Everything judged of a model at once.
// mitra: implements calculus::Reading
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    /// Every piece that is not a deduction, with its opinion.
    pub pieces: Vec<(SomePiece, Opinion)>,
    /// What cannot all be true, the lightest first: sets of taken
    /// pieces to give up, then for each piece that came out false the
    /// pieces it rests on that are not known true.
    pub suspects: Vec<Vec<SomePiece>>,
}

/// The pieces a set names, each once.
fn named(drawn: &Drawn, set: &Env) -> Vec<SomePiece> {
    let mut group: Vec<SomePiece> = Vec::new();
    for x in set.iter().map(|i| &drawn.assumed[*i].piece) {
        if !group.iter().any(|old| old.name == x.name) {
            group.push(x.clone());
        }
    }
    group
}

/// Judges every piece of the model.
///
/// - What was taken starts as it was taken. States spread by the
///   steps until none gives anything new.
/// - A state is as sure as the least that would have to be given up
///   for it to fall.
/// - A piece is in conflict when it follows true from some takings
///   and false from others.
// mitra: implements calculus::judge
pub fn judge(model: &Model) -> Reading {
    let all = pieces(model);
    let drawn = drawn(model, &all);
    let w = model.settings.weight;
    let mut out: Vec<(SomePiece, Opinion)> = Vec::new();
    for x in all.iter().filter(|x| !given(model, x)) {
        let said = |values: &mut dyn Iterator<Item = bool>| {
            values.fold(State::Unknown, |state, value| joined(state, of(value)))
        };
        let evaluated = model.evaluated.iter().filter(|(p, _)| p.name == x.name);
        let taken = model.taken.iter().filter(|t| t.piece.name == x.name);
        let count = |value: bool| {
            let mut best = f64::INFINITY;
            match drawn.labels.get(&(x.name.clone(), value)) {
                None => 0.0,
                Some(sets) => {
                    drawn.least(sets, &mut Vec::new(), &mut best);
                    best
                }
            }
        };
        let rests = |value: bool| -> Vec<Vec<SomePiece>> {
            let found = drawn.labels.get(&(x.name.clone(), value));
            let sets = found.map(Vec::as_slice).unwrap_or(&[]);
            sets.iter().map(|env| named(&drawn, env)).collect()
        };
        let (rests_for, rests_against) = (rests(true), rests(false));
        // Only a piece with one side can be left with nothing.
        let side = match (rests_for.is_empty(), rests_against.is_empty()) {
            (false, true) => Some(true),
            (true, false) => Some(false),
            _ => None,
        };
        let falls = side.map_or(Vec::new(), |value| drawn.falls(x, value));
        let opinion = Opinion {
            evaluated: said(&mut evaluated.map(|(_, value)| *value)),
            taken: said(&mut taken.map(|t| t.value)),
            yes: count(true),
            no: count(false),
            rests_for,
            rests_against,
            falls: falls.iter().map(|way| named(&drawn, way)).collect(),
            bias: bias(model, x),
            weight: w,
        };
        out.push((x.clone(), opinion));
    }

    let mut given_up: Vec<(Env, f64)> = match drawn.nogoods.is_empty() {
        true => Vec::new(),
        false => {
            let sets = drawn.ways.clone();
            sets.into_iter()
                .map(|set| {
                    let cost = drawn.together(&set);
                    (set, cost)
                })
                .collect()
        }
    };
    given_up.sort_by(|x, y| x.1.total_cmp(&y.1).then(x.0.cmp(&y.0)));
    let mut suspects: Vec<Vec<SomePiece>> = Vec::new();
    for (set, _) in &given_up {
        let group = named(&drawn, set);
        if !suspects.contains(&group) {
            suspects.push(group);
        }
    }
    for t in all.iter().filter(|t| !t.post.is_empty()) {
        if !t.post.iter().any(|q| drawn.state(q) == State::False) {
            continue;
        }
        let mut open: Vec<SomePiece> = t.pre.clone();
        if !given(model, t) {
            open.push(t.clone());
        }
        open.retain(|x| !drawn.told(x, true));
        open.sort_by(|x, y| {
            let by_bias = bias(model, x).total_cmp(&bias(model, y));
            by_bias.then(x.name.cmp(&y.name))
        });
        if open.len() > 1 && !suspects.contains(&open) {
            suspects.push(open);
        }
    }
    Reading {
        pieces: out,
        suspects,
    }
}

/// The opinion of one piece.
// mitra: implements calculus::opinion
pub fn opinion<T>(model: &Model, x: &Piece<T>) -> Opinion {
    let found = judge(model)
        .pieces
        .into_iter()
        .find(|(p, _)| p.name == x.name);
    found.map_or(
        Opinion::of(bias(model, x), model.settings.weight),
        |(_, opinion)| opinion,
    )
}

/// What the judged state of a piece rests on: the smallest sets of
/// pieces it follows from. Nothing for a piece that is unknown or in
/// conflict.
// mitra: implements calculus::rests
pub fn rests<T>(model: &Model, x: &Piece<T>) -> Vec<Vec<SomePiece>> {
    opinion(model, x).rests().to_vec()
}

/// The verdict of a model: conflict if any piece is in conflict; else
/// false if one is false; else unknown if one is unknown; else true.
// mitra: implements calculus::verdict
pub fn verdict(model: &Model) -> State {
    let states: Vec<State> = judge(model)
        .pieces
        .iter()
        .map(|(_, o)| o.judged())
        .collect();
    for worst in [State::Conflict, State::False, State::Unknown] {
        if states.contains(&worst) {
            return worst;
        }
    }
    match states.is_empty() {
        true => State::Unknown,
        false => State::True,
    }
}

/// The suspects of a model, the lightest first.
// mitra: implements calculus::suspects
pub fn suspects(model: &Model) -> Vec<Vec<SomePiece>> {
    judge(model).suspects
}

/// How sure a verdict that holds is: its least sure piece, and which
/// one it is. Nothing unless every piece is true.
// mitra: implements calculus::least_sure
pub fn least_sure(model: &Model) -> Option<(SomePiece, f64)> {
    if verdict(model) != State::True {
        return None;
    }
    let mut least: Option<(SomePiece, f64)> = None;
    for (x, o) in judge(model).pieces {
        let n = o.confidence()?;
        if least.as_ref().is_none_or(|(_, m)| n < *m) {
            least = Some((x, n));
        }
    }
    least
}

/// An opinion in words: "true 0.91", "false 0.85", "unknown",
/// "conflict".
pub fn shown(o: &Opinion) -> String {
    let state = format!("{:?}", o.judged()).to_lowercase();
    match o.confidence() {
        Some(n) => format!("{state} {n:.2}"),
        None => state,
    }
}
