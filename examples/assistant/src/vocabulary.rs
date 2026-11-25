//! The words of the system (see vocabulary.mitra).

use mitra::Observe;

// mitra: implements Device
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Observe)]
pub enum Device {
    Phone,
    Computer,
    Wearable,
}

// mitra: implements Holder
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Observe)]
pub struct Holder {
    pub household: i64,
    pub device: Device,
    pub number: i64,
}

// mitra: implements Party
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Party {
    Client(Holder),
    Assistant,
}

// mitra: implements MessageId
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Observe)]
pub struct MessageId {
    pub from: Party,
    pub n: i64,
}

/// Recorded voice. The fake speech reads its bytes as the words said.
// mitra: implements Audio
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Audio {
    pub bytes: String,
}

// mitra: implements Body
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub enum Body {
    Text(String),
    Voice(Audio),
}

// mitra: implements Envelope
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Envelope {
    pub id: MessageId,
    pub body: Body,
}

// mitra: implements Receipt
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Receipt {
    Received(MessageId),
    Read(MessageId),
    Delivered(MessageId),
}

// mitra: implements Input
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub enum Input {
    Typed(String),
    Spoken(Audio),
}

// mitra: implements NetFailure
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetFailure {
    Unreachable,
    Refused,
}

/// What a device shows on every call. A secret.
// mitra: implements Token
#[derive(Clone, Debug, PartialEq, Eq, Hash, Observe)]
pub struct Token {
    pub secret: String,
}

// mitra: implements Step
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Gateway,
    Worker,
    Speech,
    Identity,
    Commands,
    Gate,
    Purpose,
    Policy,
    Confirmations,
}

// mitra: implements Note
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Note {
    Read,
    Transcript(String),
    Sender(Person),
    Verdict(Verdict),
    Ask(Ask),
    Called(Outcome),
    Answered(MessageId),
    Speech(Audio),
    Delivered,
    Held(Hold),
    Confirmed(i64),
    Begun(Call),
}

// mitra: implements Annotation
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Annotation {
    pub by: Step,
    pub note: Note,
}

// mitra: implements Hold
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Hold {
    pub code: i64,
    pub call: Call,
    pub asked: String,
}

// mitra: implements Message
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Message {
    pub holder: Holder,
    pub text: String,
    pub recognised: bool,
}

// mitra: implements Person
#[derive(Clone, Copy, Debug, PartialEq, Eq, Observe)]
pub struct Person {
    pub role: Role,
}

// mitra: implements Role
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Observe)]
pub enum Role {
    Owner,
    Member,
    Stranger,
}

// mitra: implements Verdict
#[derive(Clone, Copy, Debug, PartialEq, Eq, Observe)]
pub enum Verdict {
    Act,
    Noise,
    Malicious,
    Distress,
    Meta,
}

// mitra: implements Ask
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Ask {
    pub text: String,
    pub clear: bool,
    pub easy: bool,
}

// mitra: implements Machine
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Observe)]
pub enum Machine {
    Local,
    Cloud,
}

// mitra: implements Route
#[derive(Clone, Copy, Debug, PartialEq, Eq, Observe)]
pub enum Route {
    Work,
    AskBack,
    Help,
    Tune,
    Decline,
}

// mitra: implements Context
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Context {
    pub turns: Vec<Turn>,
}

// mitra: implements Habits
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Habits {}

// mitra: implements Preferences
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Preferences {}

// mitra: implements Level
#[derive(Clone, Copy, Debug, PartialEq, Eq, Observe)]
pub enum Level {
    Reads,
    Acts,
    Irreversible,
}

/// A tool of the assistant's own list. What it does is the fake's.
// mitra: implements Tool
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub level: Level,
}

impl Tool {
    // mitra: implements Tool::call
    pub fn call(self, input: String) -> Result<String, ToolFailure> {
        crate::fake::tool_call(self, input)
    }
}

// mitra: implements Call
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Call {
    pub tool: String,
    pub input: String,
}

// mitra: implements Answer
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Answer {
    pub text: String,
    pub calls: Vec<Call>,
}

// mitra: implements LlmFailure
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LlmFailure {
    TooLong,
    Unreachable,
}

// mitra: implements ToolFailure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Observe)]
pub enum ToolFailure {
    Failed,
    Refused,
    AwaitsYes,
    Unknown,
}

// mitra: implements Decision
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Run,
    Confirm,
    Refuse,
}

// mitra: implements Outcome
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Outcome {
    pub call: Call,
    pub output: Result<String, ToolFailure>,
}

// mitra: implements HarnessFailure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Observe)]
pub enum HarnessFailure {
    EmptyPrompt,
    Llm(LlmFailure),
    Tool(ToolFailure),
    TooManyTurns,
}

// mitra: implements Reply
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Reply {
    pub text: String,
}

// mitra: implements Turn
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub enum Turn {
    User(String),
    Model(Answer),
    Results(Vec<Outcome>),
}

/// A short way to write a text in code and tests.
pub fn s(t: &str) -> String {
    t.to_string()
}

/// Written with the enum's name: `Unreachable`, `Confirm`, `Speech` and
/// `Read` are variants of two enums each, `Assistant` is also a part, and the reader takes a bare one
/// as no Mitra value (R001), even where the field's type says which.
macro_rules! observed_whole {
    ($($t:ident),*) => {$(
        impl mitra::Observe for $t {
            fn observe(&self) -> String {
                format!("{}::{:?}", stringify!($t), self)
            }
        }
    )*};
}
observed_whole!(NetFailure, LlmFailure, Decision, Step);

impl mitra::Observe for Party {
    fn observe(&self) -> String {
        match self {
            Party::Client(h) => format!("Party::Client({})", h.observe()),
            Party::Assistant => s("Party::Assistant"),
        }
    }
}

impl mitra::Observe for Receipt {
    fn observe(&self) -> String {
        let (v, id) = match self {
            Receipt::Received(id) => ("Received", id),
            Receipt::Read(id) => ("Read", id),
            Receipt::Delivered(id) => ("Delivered", id),
        };
        format!("Receipt::{v}({})", id.observe())
    }
}

impl mitra::Observe for Note {
    fn observe(&self) -> String {
        match self {
            Note::Read => s("Note::Read"),
            Note::Delivered => s("Note::Delivered"),
            Note::Transcript(t) => format!("Note::Transcript({})", t.observe()),
            Note::Sender(p) => format!("Note::Sender({})", p.observe()),
            Note::Verdict(v) => format!("Note::Verdict({})", v.observe()),
            Note::Ask(a) => format!("Note::Ask({})", a.observe()),
            Note::Called(o) => format!("Note::Called({})", o.observe()),
            Note::Answered(id) => format!("Note::Answered({})", id.observe()),
            Note::Speech(a) => format!("Note::Speech({})", a.observe()),
            Note::Held(h) => format!("Note::Held({})", h.observe()),
            Note::Confirmed(c) => format!("Note::Confirmed({})", c.observe()),
            Note::Begun(c) => format!("Note::Begun({})", c.observe()),
        }
    }
}
