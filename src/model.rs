//! The piece system (`model/model.mitra`): a piece, its opinion, and
//! the calculus that judges every piece of a model from what was
//! taken. It knows nothing of values.

use std::collections::BTreeMap;

/// A location in the system. It is not right or wrong.
// mitra: implements place::Place
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place {
    pub path: Vec<String>,
}

impl Place {
    pub fn at(path: &[&str]) -> Place {
        Place {
            path: path.iter().map(|p| p.to_string()).collect(),
        }
    }
}

/// `a` is within `b`: its path goes on from the other's.
// mitra: implements place::within
pub fn within(a: &Place, b: &Place) -> bool {
    a.path.starts_with(&b.path)
}

/// The step of logic a piece is.
// mitra: implements piece::Inference
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inference {
    Deduction,
    Induction,
    Abduction,
}

/// A proposed property of the system: given `pre`, `subject` gives
/// `post`. Known by its name. With no `pre` and no `post` it is a
/// leaf: something said of the subject, with nothing drawn from it.
// mitra: implements piece::Piece
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub name: String,
    pub pre: Vec<Piece>,
    pub subject: Place,
    pub post: Vec<Piece>,
    pub by: Inference,
}

/// A piece with nothing before and nothing after.
// mitra: implements piece::leaf
pub fn leaf(name: &str, subject: &Place) -> Piece {
    Piece {
        name: name.to_string(),
        pre: Vec::new(),
        subject: subject.clone(),
        post: Vec::new(),
        by: Inference::Deduction,
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

/// What is believed of a piece. Read it through its five getters: the
/// inside is free to change.
// mitra: implements piece::Opinion
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opinion {
    evaluated: State,
    taken: State,
    judged: State,
    sure: Option<f64>,
    bias: f64,
}

impl Opinion {
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
    /// What the calculus judges it to be.
    // mitra: implements piece::judged
    pub fn judged(&self) -> State {
        self.judged
    }
    /// How sure the judged state is. None when unknown or in conflict.
    // mitra: implements piece::confidence
    pub fn confidence(&self) -> Option<f64> {
        self.sure
    }
    /// What to expect of it before anything.
    // mitra: implements piece::bias
    pub fn bias(&self) -> f64 {
        self.bias
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

/// A voice takes a piece as true or as false. `count` is how firmly:
/// one test is 1.
// mitra: implements model::Taking
#[derive(Clone, Debug, PartialEq)]
pub struct Taking {
    pub piece: Piece,
    pub value: bool,
    pub count: f64,
    pub by: Voice,
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
/// - `given`: pieces that hold by what they are: steps of logic.
/// - `evaluated`: what computing a piece gave.
/// - `taken`: what voices took.
// mitra: implements model::Model
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub pieces: Vec<Piece>,
    pub given: Vec<Piece>,
    pub evaluated: Vec<(Piece, bool)>,
    pub taken: Vec<Taking>,
    pub bias: Vec<(Piece, f64)>,
    pub voices: Vec<(Voice, f64)>,
    pub settings: Settings,
}

impl Model {
    pub fn empty(settings: Settings) -> Model {
        Model {
            pieces: Vec::new(),
            given: Vec::new(),
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
pub fn pieces(model: &Model) -> Vec<Piece> {
    fn collect(found: &mut Vec<Piece>, x: &Piece) {
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
    for x in model.pieces.iter().chain(&model.given) {
        collect(&mut found, x);
    }
    for x in taken.chain(evaluated) {
        collect(&mut found, x);
    }
    found
}

/// What to expect of a piece before anything.
// mitra: implements model::bias
pub fn bias(model: &Model, x: &Piece) -> f64 {
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

fn given(model: &Model, x: &Piece) -> bool {
    model.given.iter().any(|g| g.name == x.name)
}

/// A set of takings, each by its place in the list, in order.
type Env = Vec<usize>;
/// A piece told true, or told false, by its name.
type Fact = (String, bool);

/// When all of `needs` are so, `gives` is so.
struct Step {
    needs: Vec<Fact>,
    gives: Fact,
}

/// The states with what each was drawn from.
///
/// - `labels`: for each piece told true or told false, the smallest
///   sets of takings it follows from, none in contradiction with
///   itself.
/// - `nogoods`: the smallest sets of takings that cannot all be true.
struct Drawn<'a> {
    model: &'a Model,
    weights: Vec<f64>,
    labels: BTreeMap<Fact, Vec<Env>>,
    nogoods: Vec<Env>,
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
fn steps(model: &Model, all: &[Piece]) -> Vec<Step> {
    let mut steps = Vec::new();
    for t in all.iter().filter(|t| !t.post.is_empty()) {
        let so = |x: &Piece, value: bool| (x.name.clone(), value);
        let mut holds: Vec<&Piece> = t.pre.iter().collect();
        if !given(model, t) {
            holds.push(t);
        }
        // A piece whose every piece after is true holds, whatever
        // is before it.
        if !given(model, t) {
            steps.push(Step {
                needs: t.post.iter().map(|q| so(q, true)).collect(),
                gives: so(t, true),
            });
        }
        for q in &t.post {
            let needs = holds.iter().map(|x| so(x, true)).collect();
            steps.push(Step {
                needs,
                gives: so(q, true),
            });
            for one in &holds {
                let mut needs = vec![so(q, false)];
                let others = holds.iter().filter(|x| x.name != one.name);
                needs.extend(others.map(|x| so(x, true)));
                steps.push(Step {
                    needs,
                    gives: so(one, false),
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
fn drawn<'a>(model: &'a Model, all: &[Piece]) -> Drawn<'a> {
    let steps = steps(model, all);
    let weights: Vec<f64> = model.taken.iter().map(|t| weight(model, t)).collect();
    let mut labels: BTreeMap<Fact, Vec<Env>> = BTreeMap::new();
    for (i, t) in model.taken.iter().enumerate() {
        // A taking that weighs nothing is not taken at all.
        if weights[i] > 0.0 {
            let fact = (t.piece.name.clone(), t.value);
            labels.entry(fact).or_default().push(vec![i]);
        }
    }
    let mut nogoods: Vec<Env> = Vec::new();
    loop {
        let mut moved = false;
        for step in &steps {
            let mut ways: Vec<Env> = vec![Vec::new()];
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
            return Drawn {
                model,
                weights,
                labels,
                nogoods,
            };
        }
    }
}

impl Drawn<'_> {
    fn told(&self, x: &Piece, value: bool) -> bool {
        self.labels.contains_key(&(x.name.clone(), value))
    }

    fn state(&self, x: &Piece) -> State {
        match (self.told(x, true), self.told(x, false)) {
            (false, false) => State::Unknown,
            (true, false) => State::True,
            (false, true) => State::False,
            (true, true) => State::Conflict,
        }
    }

    /// What a set of takings weighs together: one voice that takes
    /// one piece the same way twice counts once, as the firmer.
    fn together(&self, given_up: &[usize]) -> f64 {
        let mut said: Vec<((&Voice, &str, bool), f64)> = Vec::new();
        for i in given_up {
            let t = &self.model.taken[*i];
            let what = (&t.by, t.piece.name.as_str(), t.value);
            match said.iter_mut().find(|(old, _)| *old == what) {
                Some(found) => found.1 = f64::max(found.1, self.weights[*i]),
                None => said.push((what, self.weights[*i])),
            }
        }
        said.iter().map(|found| found.1).sum()
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
    /// Every piece that is not given, with its opinion.
    pub pieces: Vec<(Piece, Opinion)>,
    /// What cannot all be true, the lightest first: sets of taken
    /// pieces to give up, then for each piece that came out false the
    /// pieces it rests on that are not known true.
    pub suspects: Vec<Vec<Piece>>,
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
    let mut out: Vec<(Piece, Opinion)> = Vec::new();
    for x in all.iter().filter(|x| !given(model, x)) {
        let said = |values: &mut dyn Iterator<Item = bool>| {
            values.fold(State::Unknown, |state, value| joined(state, of(value)))
        };
        let evaluated = model.evaluated.iter().filter(|(p, _)| p.name == x.name);
        let taken = model.taken.iter().filter(|t| t.piece.name == x.name);
        let (judged, a) = (drawn.state(x), bias(model, x));
        let count = |value: bool| {
            let mut best = f64::INFINITY;
            let sets = &drawn.labels[&(x.name.clone(), value)];
            drawn.least(sets, &mut Vec::new(), &mut best);
            best
        };
        let sure = match judged {
            State::True => Some((count(true) + a * w) / (count(true) + w)),
            State::False => Some((count(false) + (1.0 - a) * w) / (count(false) + w)),
            _ => None,
        };
        let opinion = Opinion {
            evaluated: said(&mut evaluated.map(|(_, value)| *value)),
            taken: said(&mut taken.map(|t| t.value)),
            judged,
            sure,
            bias: a,
        };
        out.push((x.clone(), opinion));
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
    let mut suspects: Vec<Vec<Piece>> = Vec::new();
    for (set, _) in &given_up {
        let mut group: Vec<Piece> = Vec::new();
        for i in set {
            let x = &model.taken[*i].piece;
            if !group.iter().any(|old| old.name == x.name) {
                group.push(x.clone());
            }
        }
        suspects.push(group);
    }
    for t in all.iter().filter(|t| !t.post.is_empty()) {
        if !t.post.iter().any(|q| drawn.state(q) == State::False) {
            continue;
        }
        let mut open: Vec<Piece> = t.pre.clone();
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
pub fn opinion(model: &Model, x: &Piece) -> Opinion {
    let found = judge(model)
        .pieces
        .into_iter()
        .find(|(p, _)| p.name == x.name);
    found.map_or(
        Opinion {
            evaluated: State::Unknown,
            taken: State::Unknown,
            judged: State::Unknown,
            sure: None,
            bias: bias(model, x),
        },
        |(_, opinion)| opinion,
    )
}

/// The verdict of a model: conflict if any piece is in conflict; else
/// false if one is false; else unknown if one is unknown; else true.
// mitra: implements calculus::verdict
pub fn verdict(model: &Model) -> State {
    let states: Vec<State> = judge(model).pieces.iter().map(|(_, o)| o.judged).collect();
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
pub fn suspects(model: &Model) -> Vec<Vec<Piece>> {
    judge(model).suspects
}

/// How sure a verdict that holds is: its least sure piece, and which
/// one it is. Nothing unless every piece is true.
// mitra: implements calculus::least_sure
pub fn least_sure(model: &Model) -> Option<(Piece, f64)> {
    if verdict(model) != State::True {
        return None;
    }
    let mut least: Option<(Piece, f64)> = None;
    for (x, o) in judge(model).pieces {
        let n = o.sure?;
        if least.as_ref().is_none_or(|(_, m)| n < *m) {
            least = Some((x, n));
        }
    }
    least
}

/// An opinion in words: "true 0.91", "false 0.85", "unknown",
/// "conflict".
pub fn shown(o: &Opinion) -> String {
    let state = format!("{:?}", o.judged).to_lowercase();
    match o.sure {
        Some(n) => format!("{state} {n:.2}"),
        None => state,
    }
}
