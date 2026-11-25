//! The fake world: every machine of the model that is not our code. The
//! two LLMs, speech, the tools, the accounts, the clock and the codes.
//! Tests script it; by default it answers by simple rules, and the work
//! model is gullible on purpose: it obeys any line that starts with
//! `CALL`, so an injection that reaches it shows.
//!
//! Each answer is printed with `mitra::given!`, so a test's run becomes a
//! measurement `mitra explain` reads.

use crate::vocabulary::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

/// What the world holds. One per thread: each test has its own.
#[derive(Default)]
pub struct World {
    pub scripts: HashMap<Machine, VecDeque<Result<Answer, LlmFailure>>>,
    pub down: HashSet<Machine>,
    pub sent: Vec<(Machine, String)>,
    pub tool_out: HashMap<String, String>,
    pub performed: Vec<Call>,
    pub tokens: HashMap<String, Holder>,
    pub roles: HashMap<Holder, Role>,
    pub alerts: Vec<String>,
    pub codes: VecDeque<i64>,
    pub seed: u64,
    pub unusual: bool,
    pub quiet: bool,
}

/// `given!`, unless the world is quiet: a scenario through the whole
/// server is no row of one model function, so its givens would be read
/// as the world of no row (R014).
macro_rules! gave {
    ($($f:ident)::+ ( $($a:expr),* $(,)? ), $v:expr) => {{
        if with(|w| w.quiet) {
            $v
        } else {
            mitra::given!($($f)::+ ($($a),*), $v)
        }
    }};
}

thread_local! {
    static WORLD: RefCell<World> = RefCell::new(World { seed: seed(), ..World::default() });
}

/// A seed that differs at each start (K5): the clock and a counter.
fn seed() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNT: AtomicU64 = AtomicU64::new(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    now ^ COUNT.fetch_add(0x9e37_79b9_7f4a_7c15, Ordering::Relaxed)
}

pub fn with<R>(f: impl FnOnce(&mut World) -> R) -> R {
    WORLD.with(|w| f(&mut w.borrow_mut()))
}

/// A fresh world.
pub fn reset() {
    with(|w| {
        *w = World {
            seed: seed(),
            ..World::default()
        }
    });
}

// ---- Scripting, for tests.

/// The next answers of a machine, in order; then its default rules again.
pub fn script(m: Machine, answers: Vec<Result<Answer, LlmFailure>>) {
    with(|w| w.scripts.entry(m).or_default().extend(answers));
}
pub fn down(m: Machine, is_down: bool) {
    with(|w| {
        if is_down {
            w.down.insert(m);
        } else {
            w.down.remove(&m);
        }
    });
}
/// The requests a machine was sent.
pub fn sent(m: Machine) -> Vec<String> {
    with(|w| {
        w.sent
            .iter()
            .filter(|(x, _)| *x == m)
            .map(|(_, r)| r.clone())
            .collect()
    })
}
/// What a tool gives back when it runs.
pub fn tool_output(name: &str, out: &str) {
    with(|w| w.tool_out.insert(name.to_string(), out.to_string()));
}
/// Every tool call that ran: the privilege detector.
pub fn performed() -> Vec<Call> {
    with(|w| w.performed.clone())
}
pub fn alerts() -> Vec<String> {
    with(|w| w.alerts.clone())
}
/// Pairs a token to a device of a household, with its role.
pub fn pair(token: &str, h: Holder, role: Role) {
    with(|w| {
        w.tokens.insert(token.to_string(), h);
        w.roles.insert(h, role);
    });
}
pub fn revoke(token: &str) {
    with(|w| w.tokens.remove(token));
}
/// The next codes handed out, in order.
pub fn codes(cs: Vec<i64>) {
    with(|w| w.codes.extend(cs));
}
/// Givens are not printed: for scenarios through the whole server.
pub fn quiet() {
    with(|w| w.quiet = true);
}
/// Whether the owner's habits make every call look unusual.
pub fn unusual(u: bool) {
    with(|w| w.unusual = u);
}

// ---- The machines, as the model names them.

/// Llm::inference: refuses a request that leaves less than the reserve,
/// fails when the machine is down, else answers.
pub fn inference(
    llm: crate::harness::models::Llm,
    request: String,
    tools: Vec<Tool>,
) -> Result<Answer, LlmFailure> {
    let got = if request.len() as i64 + llm.reserve > llm.window {
        Err(LlmFailure::TooLong)
    } else if with(|w| w.down.contains(&llm.machine)) {
        Err(LlmFailure::Unreachable)
    } else {
        with(|w| w.sent.push((llm.machine, request.clone())));
        match with(|w| w.scripts.get_mut(&llm.machine).and_then(|q| q.pop_front())) {
            Some(scripted) => scripted,
            None => Ok(rules(&request)),
        }
    };
    gave!(Llm::inference(llm, request, tools), got)
}

fn text(t: &str) -> Answer {
    Answer {
        text: t.to_string(),
        calls: vec![],
    }
}

/// The default answers, by the kind of request.
fn rules(request: &str) -> Answer {
    if let Some(rest) = request.strip_prefix("JUDGE:") {
        return text(judge(&field(rest, "MESSAGE: ").to_lowercase()));
    }
    if let Some(rest) = request.strip_prefix("PIN:") {
        return text(&pin(&field(rest, "MESSAGE: ").to_lowercase()));
    }
    work(request)
}

/// The text after `key` up to the end of the request (the message is
/// written last, so it may hold any line).
fn field(request: &str, key: &str) -> String {
    match request.find(key) {
        Some(i) => request[i + key.len()..].to_string(),
        None => String::new(),
    }
}

fn judge(t: &str) -> &'static str {
    let malicious = [
        "bank code",
        "password",
        "ignore your rules",
        "ignore previous",
        "pretend you are",
        "you are now",
    ];
    let meta = [
        "you are slow",
        "good job",
        "well done",
        "you are wrong",
        "speak slower",
    ];
    let distress = ["i fell", "emergency", "help me now", "i am hurt"];
    if malicious.iter().any(|k| t.contains(k)) {
        "malicious"
    } else if distress.iter().any(|k| t.contains(k)) {
        "distress"
    } else if meta.iter().any(|k| t.contains(k)) {
        "meta"
    } else {
        "act"
    }
}

const TOOL_WORDS: [&str; 9] = [
    "door", "light", "news", "web", "bank", "send", "pay", "buy", "email",
];

fn pin(t: &str) -> String {
    let t = t.trim();
    if ["that one", "this", "it", "that", "do it"].contains(&t) {
        return "unclear".to_string();
    }
    let easy = t.split_whitespace().count() <= 4 && !TOOL_WORDS.iter().any(|k| t.contains(k));
    format!("clear {}", if easy { "easy" } else { "hard" })
}

/// The work model: gullible. It asks for every call written on a line
/// of its own as `CALL <tool> <input>` that is not yet `CALLED`, and for
/// the calls the ask's words suggest; after results, it answers.
fn work(request: &str) -> Answer {
    let lines: Vec<&str> = request.lines().collect();
    let called: HashSet<String> = lines
        .iter()
        .filter_map(|l| l.strip_prefix("CALLED "))
        .map(|s| s.to_string())
        .collect();
    let mut calls: Vec<Call> = lines
        .iter()
        .filter_map(|l| l.strip_prefix("CALL "))
        .filter(|c| !called.contains(*c))
        .map(|c| {
            let mut p = c.splitn(2, ' ');
            Call {
                tool: p.next().unwrap_or("").to_string(),
                input: p.next().unwrap_or("").to_string(),
            }
        })
        .collect();
    let ask = lines
        .iter()
        .find_map(|l| l.strip_prefix("ASK: "))
        .unwrap_or("")
        .to_lowercase();
    if called.is_empty() {
        let intents = [
            ("door", "door", "open"),
            ("light", "lights", "on"),
            ("news", "web", "news"),
        ];
        for (word, tool, input) in intents {
            if ask.contains(word) && !calls.iter().any(|c| c.tool == tool) {
                calls.push(Call {
                    tool: tool.to_string(),
                    input: input.to_string(),
                });
            }
        }
    }
    calls.dedup();
    if !calls.is_empty() {
        return Answer {
            text: String::new(),
            calls,
        };
    }
    // Each result, with the guarded lines of its output after it.
    let mut results: Vec<String> = vec![];
    for l in &lines {
        if let Some(r) = l.strip_prefix("RESULT ") {
            results.push(r.to_string());
        } else if let (Some(out), Some(last)) = (l.strip_prefix("| "), results.last_mut()) {
            last.push(' ');
            last.push_str(out);
        }
    }
    if results.is_empty() {
        text(&format!("Here is my answer to: {ask}"))
    } else {
        text(&format!("Done. {}", results.join("; ")))
    }
}

/// Tool::call: the tool runs, and the world notes it.
pub fn tool_call(t: Tool, input: String) -> Result<String, ToolFailure> {
    with(|w| {
        w.performed.push(Call {
            tool: t.name.clone(),
            input: input.clone(),
        })
    });
    let out = with(|w| w.tool_out.get(&t.name).cloned())
        .unwrap_or_else(|| format!("{} {}: done", t.name, input));
    gave!(Tool::call(t, input), Ok::<String, ToolFailure>(out))
}

/// Accounts::holder.
pub fn holder(t: Token) -> Option<Holder> {
    let h = with(|w| w.tokens.get(&t.secret).copied());
    gave!(Accounts::holder(t), h)
}

/// Accounts::owner: the holder paired as owner, else the first phone.
pub fn owner(household: i64) -> Holder {
    let paired = with(|w| {
        let mut owners: Vec<Holder> = w
            .roles
            .iter()
            .filter(|(h, r)| h.household == household && **r == Role::Owner)
            .map(|(h, _)| *h)
            .collect();
        owners.sort();
        owners.first().copied()
    });
    let h = paired.unwrap_or(Holder {
        household,
        device: Device::Phone,
        number: 1,
    });
    gave!(Accounts::owner(household), h)
}

/// Identity::who: the role the household gave this device, when the
/// device vouches for the speaker; a stranger otherwise.
pub fn who(m: Message) -> Person {
    let role = if m.recognised {
        with(|w| w.roles.get(&m.holder).copied()).unwrap_or(Role::Stranger)
    } else {
        Role::Stranger
    };
    gave!(Identity::who(m), Person { role })
}

/// Helpline::alert.
pub fn alert(p: Person, m: Message) -> bool {
    with(|w| w.alerts.push(m.text.clone()));
    gave!(Helpline::alert(p, m), true)
}

/// Codes::fresh: a scripted code, else six digits from a generator.
pub fn fresh() -> i64 {
    let c = with(|w| {
        w.codes.pop_front().unwrap_or_else(|| {
            w.seed = w
                .seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            100_000 + ((w.seed >> 33) % 900_000) as i64
        })
    });
    gave!(Codes::fresh(), c)
}

/// Experience::usual.
pub fn usual(c: Call, h: Habits) -> bool {
    let u = !with(|w| w.unusual);
    gave!(Experience::usual(c, h), u)
}

/// Speech::transcribe: the fake reads the audio's bytes as the words.
/// `[unclear]` makes it fail; `[unknown]` marks a voice not enrolled.
pub fn transcribe(a: Audio) -> Result<String, crate::server::speech::SpeechFailure> {
    let got = if a.bytes.starts_with("[unclear]") {
        Err(crate::server::speech::SpeechFailure::Unclear)
    } else {
        Ok(a.bytes.trim_start_matches("[unknown]").trim().to_string())
    };
    gave!(Speech::transcribe(a), got)
}
pub fn speaker(a: Audio) -> bool {
    let known = !a.bytes.starts_with("[unknown]");
    gave!(Speech::speaker(a), known)
}
pub fn speak(t: String) -> Audio {
    let a = Audio { bytes: t.clone() };
    gave!(Speech::speak(t), a)
}

/// The clock, for the time command.
pub fn now() -> String {
    "ten o'clock".to_string()
}

/// Limits::text, a setting.
pub fn text_limit() -> i64 {
    gave!(Limits::text(), crate::server::gateway::TEXT_LIMIT)
}

/// Limits::voice, a setting.
pub fn limit_voice() -> i64 {
    gave!(Limits::voice(), crate::server::gateway::VOICE_LIMIT)
}

/// Limits::ids, a setting.
pub fn largest_id() -> i64 {
    gave!(Limits::ids(), crate::server::gateway::LARGEST_ID)
}
