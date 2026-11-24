//! tickets: a to-do list on the command line, kept in a log file.
//!
//! Every change is one event appended to the log; the board is rebuilt by
//! replaying the log. The model is `tickets.mitra`.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Added(i64, String),
    Done(i64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Whole(Event),
    Cut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Add(String),
    Finish(i64),
    List,
    Show(i64),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Id(i64),
    All(Vec<Ticket>),
    One(Ticket),
    Nothing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Usage,
    EmptyTitle,
    NoSuchTicket,
    Storage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Write(Event, Reply),
    Read(Reply),
    Fail(Failure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Printed(Reply),
    Failed(Failure),
}

// ---------------------------------------------------------------- Disk

/// The log file, opened and locked for the length of one command, so that
/// two commands run at once never pick the same id.
pub struct Disk {
    file: File,
    /// Whether the file ends with a newline (or is empty): when it does not,
    /// its last line was cut, and the next append starts on a fresh line so
    /// that the cut line is never glued to a whole one.
    ends_clean: bool,
}

impl Disk {
    pub fn open(path: &Path) -> std::io::Result<Disk> {
        let new = !path.exists();
        let file = OpenOptions::new().read(true).append(true).create(true).open(path)?;
        if new {
            // the new file's name must survive a crash too
            let dir = match path.parent() {
                Some(d) if !d.as_os_str().is_empty() => d,
                _ => Path::new("."),
            };
            File::open(dir)?.sync_all()?;
        }
        file.lock()?;
        Ok(Disk { file, ends_clean: true })
    }

    // mitra: implements Disk::read
    /// Every line of the log; a last line with no newline is given as it is
    /// (the codec reads a cut one as `Line::Cut`).
    pub fn read(&mut self) -> std::io::Result<Vec<String>> {
        let mut bytes = Vec::new();
        self.file.seek(SeekFrom::Start(0))?;
        self.file.read_to_end(&mut bytes)?;
        self.ends_clean = bytes.last().map_or(true, |b| *b == b'\n');
        let mut lines: Vec<String> = bytes
            .split(|b| *b == b'\n')
            .map(|l| String::from_utf8_lossy(l).into_owned())
            .collect();
        if self.ends_clean {
            // the piece after the last newline is empty
            lines.pop();
        }
        Ok(lines)
    }

    // mitra: implements Disk::append
    /// Append one line and flush it to the disk; true only once it is there.
    pub fn append(&mut self, line: &str) -> bool {
        let mut text = String::new();
        if !self.ends_clean {
            text.push('\n');
        }
        text.push_str(line);
        text.push('\n');
        let ok = self.file.write_all(text.as_bytes()).is_ok() && self.file.sync_all().is_ok();
        self.ends_clean = ok;
        ok
    }
}

// ---------------------------------------------------------------- Codec

/// Events as lines of text: `added <id> <title>$` or `done <id>$`. The title
/// is escaped so that it holds no newline and no `$`: the `$` that ends a
/// line is its only one, so any cut of a line lacks it and reads as `Cut`.
pub struct Codec;

const END: char = '$';

fn escape(t: &str) -> String {
    let mut s = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '$' => s.push_str("\\d"),
            c => s.push(c),
        }
    }
    s
}

fn unescape(t: &str) -> Option<String> {
    let mut s = String::with_capacity(t.len());
    let mut chars = t.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                '\\' => s.push('\\'),
                'n' => s.push('\n'),
                'r' => s.push('\r'),
                'd' => s.push('$'),
                _ => return None,
            }
        } else {
            s.push(c);
        }
    }
    Some(s)
}

/// An id: decimal digits, maybe after a `-`.
fn number(t: &str) -> Option<i64> {
    let digits = t.strip_prefix('-').unwrap_or(t);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    t.parse().ok()
}

impl Codec {
    // mitra: implements Codec::encode
    pub fn encode(e: &Event) -> String {
        match e {
            Event::Added(id, t) => format!("added {} {}{}", id, escape(t), END),
            Event::Done(id) => format!("done {}{}", id, END),
        }
    }

    // mitra: implements Codec::decode
    pub fn decode(t: &str) -> Line {
        Codec::event(t).map_or(Line::Cut, Line::Whole)
    }

    fn event(t: &str) -> Option<Event> {
        let body = t.strip_suffix(END)?;
        if body.contains(END) {
            return None;
        }
        if let Some(rest) = body.strip_prefix("added ") {
            let (id, title) = rest.split_once(' ')?;
            Some(Event::Added(number(id)?, unescape(title)?))
        } else if let Some(id) = body.strip_prefix("done ") {
            Some(Event::Done(number(id)?))
        } else {
            None
        }
    }

    // mitra: implements Codec::reads
    /// A line reads back as its event; any cut of it reads as Cut.
    pub fn reads(e: &Event, cut: &str) -> bool {
        let line = Codec::encode(e);
        Codec::decode(&line) == Line::Whole(e.clone())
            && (!line.starts_with(cut) || cut == line || Codec::decode(cut) == Line::Cut)
    }
}

// ---------------------------------------------------------------- Board

/// The tickets, rebuilt from the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub tickets: Vec<Ticket>,
    pub next: i64,
}

impl Board {
    pub fn empty() -> Board {
        Board { tickets: Vec::new(), next: 1 }
    }

    // mitra: implements Board::apply
    pub fn apply(&self, e: &Event) -> Board {
        match e {
            Event::Added(id, t) => match id.checked_add(1) {
                // an id below the next one is not new: ignored. So is the
                // largest i64, which has no next id (the model's integers have
                // no largest; the tool never writes it, since ids count from 1).
                Some(next) if *id >= self.next => {
                    let mut tickets = self.tickets.clone();
                    tickets.push(Ticket { id: *id, title: t.clone(), done: false });
                    Board { tickets, next }
                }
                _ => self.clone(),
            },
            Event::Done(id) => Board {
                tickets: self
                    .tickets
                    .iter()
                    .map(|t| Ticket { done: t.done || t.id == *id, ..t.clone() })
                    .collect(),
                next: self.next,
            },
        }
    }

    // mitra: implements Board::replay
    pub fn replay(lines: &[Line]) -> Board {
        lines.iter().fold(Board::empty(), |b, l| match l {
            Line::Whole(e) => b.apply(e),
            Line::Cut => b,
        })
    }

    // mitra: implements Board::is_done
    pub fn is_done(&self, id: i64) -> bool {
        self.tickets.iter().any(|t| t.id == id && t.done)
    }

    // mitra: implements Board::has
    pub fn has(&self, id: i64) -> bool {
        self.tickets.iter().any(|t| t.id == id)
    }
}

// ---------------------------------------------------------------- Cli

/// The command line: read the arguments, plan the step, answer.
pub struct Cli;

fn id_arg(t: &str) -> Option<i64> {
    number(t)
}

impl Cli {
    // mitra: implements Cli::read
    /// The arguments after the program's name.
    pub fn read(args: &[String]) -> Command {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        match args.as_slice() {
            ["add", t] => Command::Add(t.to_string()),
            ["done", n] => id_arg(n).map_or(Command::Unknown, Command::Finish),
            ["list"] => Command::List,
            ["show", n] => id_arg(n).map_or(Command::Unknown, Command::Show),
            _ => Command::Unknown,
        }
    }

    // mitra: implements Cli::plan
    pub fn plan(c: &Command, b: &Board) -> Step {
        match c {
            Command::Add(t) => {
                if t.is_empty() {
                    Step::Fail(Failure::EmptyTitle)
                } else if b.next == i64::MAX {
                    // no id is left: the event would be ignored (see apply),
                    // so it is not written nor acknowledged
                    Step::Fail(Failure::Storage)
                } else {
                    Step::Write(Event::Added(b.next, t.clone()), Reply::Id(b.next))
                }
            }
            Command::Finish(id) => {
                if b.has(*id) {
                    Step::Write(Event::Done(*id), Reply::Nothing)
                } else {
                    Step::Fail(Failure::NoSuchTicket)
                }
            }
            Command::List => Step::Read(Reply::All(b.tickets.clone())),
            Command::Show(id) => b
                .tickets
                .iter()
                .find(|t| t.id == *id)
                .map_or(Step::Fail(Failure::NoSuchTicket), |t| Step::Read(Reply::One(t.clone()))),
            Command::Unknown => Step::Fail(Failure::Usage),
        }
    }

    // mitra: implements Cli::answer
    /// Acknowledge a change only once the disk has it.
    pub fn answer(s: &Step, written: bool) -> Outcome {
        match s {
            Step::Write(_, r) => {
                if written {
                    Outcome::Printed(r.clone())
                } else {
                    Outcome::Failed(Failure::Storage)
                }
            }
            Step::Read(r) => Outcome::Printed(r.clone()),
            Step::Fail(f) => Outcome::Failed(*f),
        }
    }
}

// ---------------------------------------------------------------- running

fn ticket_line(t: &Ticket) -> String {
    format!("#{} [{}] {}\n", t.id, if t.done { 'x' } else { ' ' }, t.title)
}

/// What a reply prints on standard output.
pub fn render(r: &Reply) -> String {
    match r {
        Reply::Id(id) => format!("#{}\n", id),
        Reply::All(ts) => ts.iter().map(ticket_line).collect(),
        Reply::One(t) => ticket_line(t),
        Reply::Nothing => String::new(),
    }
}

/// The exit code of a failure (never 0).
pub fn code(f: Failure) -> i32 {
    match f {
        Failure::Usage => 2,
        Failure::EmptyTitle => 3,
        Failure::NoSuchTicket => 4,
        Failure::Storage => 5,
    }
}

/// One run of the tool on the log at `log`: read, replay, plan, write, answer.
pub fn run(args: &[String], log: &Path) -> Outcome {
    let c = Cli::read(args);
    if c == Command::Unknown {
        return Outcome::Failed(Failure::Usage);
    }
    let mut disk = match Disk::open(log) {
        Ok(d) => d,
        Err(_) => return Outcome::Failed(Failure::Storage),
    };
    let lines = match disk.read() {
        Ok(ls) => ls,
        Err(_) => return Outcome::Failed(Failure::Storage),
    };
    let board = Board::replay(&lines.iter().map(|l| Codec::decode(l)).collect::<Vec<_>>());
    let step = Cli::plan(&c, &board);
    let written = match &step {
        Step::Write(e, _) => disk.append(&Codec::encode(e)),
        _ => false,
    };
    Cli::answer(&step, written)
}

#[cfg(test)]
mod tests;
