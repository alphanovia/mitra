//! `tickets`: a to-do list stored as an append-only log.
//!
//! Each change is one line: `<crc> <payload>\n`, where `<crc>` is 8 lowercase hex
//! digits (FNV-1a 32 of the payload) and the payload is `A <id> <title>` or `D <id>`.
//! A line counts only if it ends in `\n` and its checksum matches; anything else
//! (a line cut short by a crash, or garbage from a failed write) is ignored.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub const LOG_FILE: &str = "tickets.log";

#[derive(Debug)]
pub enum Error {
    /// Wrong command or wrong number of arguments.
    Usage(String),
    /// The id is not a positive number.
    BadId(String),
    /// The title is empty or holds control characters.
    BadTitle,
    /// No ticket has this id.
    NotFound(u64),
    /// Reading or writing the log failed.
    Io(io::Error),
}

impl Error {
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::Usage(_) => 2,
            Error::BadId(_) | Error::BadTitle => 3,
            Error::NotFound(_) => 4,
            Error::Io(_) => 5,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Usage(m) => write!(
                f,
                "{m}\nusage: tickets add <title> | done <id> | list | show <id>"
            ),
            Error::BadId(s) => write!(f, "bad id: {s:?}"),
            Error::BadTitle => write!(f, "title must be non-empty and have no control characters"),
            Error::NotFound(id) => write!(f, "no ticket #{id}"),
            Error::Io(e) => write!(f, "log error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    Add { id: u64, title: String },
    Done { id: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

impl Ticket {
    pub fn line(&self) -> String {
        format!("#{} [{}] {}", self.id, if self.done { 'x' } else { ' ' }, self.title)
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub tickets: BTreeMap<u64, Ticket>,
    pub max_id: u64,
}

pub fn checksum(data: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in data {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

fn valid_title(t: &str) -> bool {
    !t.trim().is_empty() && !t.chars().any(char::is_control)
}

impl Record {
    fn payload(&self) -> String {
        match self {
            Record::Add { id, title } => format!("A {id} {title}"),
            Record::Done { id } => format!("D {id}"),
        }
    }

    /// The full log line, newline included.
    pub fn encode(&self) -> String {
        let p = self.payload();
        format!("{:08x} {p}\n", checksum(p.as_bytes()))
    }

    /// Decodes one line (without its `\n`). Returns `None` for anything invalid.
    pub fn decode(line: &[u8]) -> Option<Record> {
        if line.len() < 10 || line[8] != b' ' {
            return None;
        }
        let crc = std::str::from_utf8(&line[..8]).ok()?;
        if !crc.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return None;
        }
        let crc = u32::from_str_radix(crc, 16).ok()?;
        let payload = &line[9..];
        if checksum(payload) != crc {
            return None;
        }
        let payload = std::str::from_utf8(payload).ok()?;
        let mut parts = payload.splitn(3, ' ');
        let kind = parts.next()?;
        let id = parse_num(parts.next()?)?;
        match (kind, parts.next()) {
            ("A", Some(title)) if valid_title(title) => Some(Record::Add {
                id,
                title: title.to_string(),
            }),
            ("D", None) => Some(Record::Done { id }),
            _ => None,
        }
    }
}

fn parse_num(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok().filter(|&n| n > 0)
}

pub fn parse_id(s: &str) -> Result<u64, Error> {
    parse_num(s.strip_prefix('#').unwrap_or(s)).ok_or_else(|| Error::BadId(s.to_string()))
}

/// Rebuilds the state from the raw log bytes.
pub fn replay(data: &[u8]) -> State {
    let mut st = State::default();
    let mut lines: Vec<&[u8]> = data.split(|&b| b == b'\n').collect();
    lines.pop(); // the part after the last '\n' is incomplete (or empty)
    for line in lines {
        match Record::decode(line) {
            Some(Record::Add { id, title }) if id > st.max_id => {
                st.max_id = id;
                st.tickets.insert(id, Ticket { id, title, done: false });
            }
            Some(Record::Done { id }) => {
                if let Some(t) = st.tickets.get_mut(&id) {
                    t.done = true;
                }
            }
            _ => {}
        }
    }
    st
}

/// Writes one record in a single buffer. If the log does not end with `\n`
/// (a torn previous write), `\0\n` is put first: the torn bytes then end a
/// line of their own with a NUL, which no valid record holds, so they are
/// ignored even when only the final `\n` was missing.
pub fn write_record<W: Write>(w: &mut W, log_ends_clean: bool, rec: &Record) -> io::Result<()> {
    let mut buf = String::new();
    if !log_ends_clean {
        buf.push_str("\0\n");
    }
    buf.push_str(&rec.encode());
    w.write_all(buf.as_bytes())?;
    w.flush()
}

fn read_log(path: &Path) -> Result<Vec<u8>, Error> {
    match File::open(path) {
        Ok(mut f) => {
            let mut v = Vec::new();
            f.read_to_end(&mut v)?;
            Ok(v)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}

/// Appends a record and makes it durable before returning.
fn append(path: &Path, data: &[u8], rec: &Record) -> Result<(), Error> {
    let existed = path.exists();
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    let clean = data.is_empty() || data.ends_with(b"\n");
    write_record(&mut f, clean, rec)?;
    f.sync_all()?;
    if !existed {
        // Make the new directory entry durable too.
        let dir = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        };
        File::open(dir)?.sync_all()?;
    }
    Ok(())
}

/// Runs one command against the log at `path`; returns what to print on stdout.
pub fn run(args: &[String], path: &Path) -> Result<String, Error> {
    let cmd = args.first().map(String::as_str).unwrap_or("");
    match cmd {
        "add" => {
            if args.len() < 2 {
                return Err(Error::Usage("add needs a title".into()));
            }
            let title = args[1..].join(" ");
            if !valid_title(&title) {
                return Err(Error::BadTitle);
            }
            let data = read_log(path)?;
            let st = replay(&data);
            let id = st.max_id + 1;
            append(path, &data, &Record::Add { id, title })?;
            Ok(format!("#{id}"))
        }
        "done" => {
            let id = one_id(args)?;
            let data = read_log(path)?;
            let st = replay(&data);
            match st.tickets.get(&id) {
                None => Err(Error::NotFound(id)),
                Some(t) if t.done => Ok(String::new()),
                Some(_) => {
                    append(path, &data, &Record::Done { id })?;
                    Ok(String::new())
                }
            }
        }
        "list" => {
            if args.len() != 1 {
                return Err(Error::Usage("list takes no arguments".into()));
            }
            let st = replay(&read_log(path)?);
            Ok(st.tickets.values().map(Ticket::line).collect::<Vec<_>>().join("\n"))
        }
        "show" => {
            let id = one_id(args)?;
            let st = replay(&read_log(path)?);
            st.tickets.get(&id).map(Ticket::line).ok_or(Error::NotFound(id))
        }
        "" => Err(Error::Usage("missing command".into())),
        other => Err(Error::Usage(format!("unknown command {other:?}"))),
    }
}

fn one_id(args: &[String]) -> Result<u64, Error> {
    if args.len() != 2 {
        return Err(Error::Usage(format!("{} needs exactly one id", args[0])));
    }
    parse_id(&args[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A writer that accepts `limit` bytes, then fails.
    struct FailingWriter {
        out: Vec<u8>,
        limit: usize,
    }

    impl Write for FailingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let room = self.limit - self.out.len();
            if room == 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "disk full"));
            }
            let n = room.min(buf.len());
            self.out.extend_from_slice(&buf[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn add(id: u64, t: &str) -> Record {
        Record::Add { id, title: t.into() }
    }

    #[test]
    fn encode_decode_roundtrip() {
        for r in [add(1, "fix login"), add(42, "a  b # [x]"), Record::Done { id: 7 }] {
            let line = r.encode();
            assert!(line.ends_with('\n'));
            assert_eq!(Record::decode(line.trim_end_matches('\n').as_bytes()), Some(r));
        }
    }

    #[test]
    fn decode_rejects_corruption() {
        let line = add(1, "fix login").encode();
        let body = line.trim_end_matches('\n').as_bytes().to_vec();
        for i in 0..body.len() {
            let mut b = body.clone();
            b[i] ^= 0x01;
            assert_eq!(Record::decode(&b), None, "flip at {i}");
        }
        assert_eq!(Record::decode(b""), None);
        assert_eq!(Record::decode(b"garbage"), None);
    }

    #[test]
    fn every_prefix_of_a_record_is_ignored() {
        let mut log = add(1, "one").encode().into_bytes();
        let base = log.len();
        log.extend_from_slice(Record::Done { id: 1 }.encode().as_bytes());
        for cut in base..log.len() {
            let st = replay(&log[..cut]);
            assert_eq!(st.tickets.len(), 1);
            assert!(!st.tickets[&1].done, "cut at {cut}");
        }
        assert!(replay(&log).tickets[&1].done);
    }

    #[test]
    fn replay_ignores_bad_records() {
        let mut log = String::new();
        log += &add(1, "one").encode();
        log += &add(1, "dup").encode(); // id not increasing
        log += &Record::Done { id: 9 }.encode(); // unknown id
        log += "not a record\n";
        log += &add(2, "two").encode();
        let st = replay(log.as_bytes());
        let lines: Vec<_> = st.tickets.values().map(Ticket::line).collect();
        assert_eq!(lines, ["#1 [ ] one", "#2 [ ] two"]);
        assert_eq!(st.max_id, 2);
    }

    #[test]
    fn done_stays_done() {
        let mut log = String::new();
        log += &add(1, "one").encode();
        log += &Record::Done { id: 1 }.encode();
        log += &Record::Done { id: 1 }.encode();
        assert!(replay(log.as_bytes()).tickets[&1].done);
    }

    #[test]
    fn failed_write_then_recovery() {
        let log = add(1, "one").encode().into_bytes();
        let rec = add(2, "two");
        let full = rec.encode().len();
        for limit in 0..full {
            let mut w = FailingWriter { out: log.clone(), limit: log.len() + limit };
            assert!(write_record(&mut w, true, &rec).is_err());
            // The torn write is not visible.
            let st = replay(&w.out);
            assert_eq!(st.max_id, 1, "limit {limit}");
            // The next append still produces a readable record.
            let clean = w.out.ends_with(b"\n");
            let mut good = w.out.clone();
            write_record(&mut good, clean, &add(2, "again")).unwrap();
            let st = replay(&good);
            assert_eq!(st.tickets[&2].title, "again", "limit {limit}");
            assert_eq!(st.tickets[&1].title, "one");
        }
    }

    #[test]
    fn parse_ids() {
        assert_eq!(parse_id("3").unwrap(), 3);
        assert_eq!(parse_id("#3").unwrap(), 3);
        for bad in ["", "0", "-1", "x", "+3", "3.0", "#", "99999999999999999999999"] {
            assert!(matches!(parse_id(bad), Err(Error::BadId(_))), "{bad:?}");
        }
    }
}
