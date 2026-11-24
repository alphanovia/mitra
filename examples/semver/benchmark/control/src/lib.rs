//! Parse and bump semantic versions.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Usage,
    UnknownLevel(String),
    WrongPartCount(usize),
    EmptyPart,
    NotANumber(String),
    LeadingZero(String),
    TooLarge(String),
    EmptyPreRelease,
    EmptyBuild,
    Overflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Usage => write!(f, "usage: semver bump <major|minor|patch> <version>"),
            Error::UnknownLevel(s) => write!(f, "unknown level '{s}': expected major, minor or patch"),
            Error::WrongPartCount(n) => write!(f, "expected 3 numeric parts, found {n}"),
            Error::EmptyPart => write!(f, "empty numeric part"),
            Error::NotANumber(s) => write!(f, "'{s}' is not a number"),
            Error::LeadingZero(s) => write!(f, "'{s}' has a leading zero"),
            Error::TooLarge(s) => write!(f, "'{s}' is too large"),
            Error::EmptyPreRelease => write!(f, "empty pre-release tag"),
            Error::EmptyBuild => write!(f, "empty build metadata"),
            Error::Overflow => write!(f, "bump would overflow"),
        }
    }
}

impl std::error::Error for Error {}

impl std::str::FromStr for Level {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "major" => Ok(Level::Major),
            "minor" => Ok(Level::Minor),
            "patch" => Ok(Level::Patch),
            _ => Err(Error::UnknownLevel(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Option<String>,
}

fn parse_num(s: &str) -> Result<u64, Error> {
    if s.is_empty() {
        return Err(Error::EmptyPart);
    }
    if !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::NotANumber(s.to_string()));
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(Error::LeadingZero(s.to_string()));
    }
    s.parse().map_err(|_| Error::TooLarge(s.to_string()))
}

fn check_ident(s: &str, empty: Error) -> Result<(), Error> {
    let ok = !s.is_empty()
        && s.split('.').all(|p| {
            !p.is_empty() && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        });
    if ok { Ok(()) } else { Err(empty) }
}

impl std::str::FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let (rest, build) = match s.split_once('+') {
            Some((r, b)) => (r, Some(b)),
            None => (s, None),
        };
        if let Some(b) = build {
            check_ident(b, Error::EmptyBuild)?;
        }
        let (core, pre) = match rest.split_once('-') {
            Some((c, p)) => (c, Some(p)),
            None => (rest, None),
        };
        if let Some(p) = pre {
            check_ident(p, Error::EmptyPreRelease)?;
        }
        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::WrongPartCount(parts.len()));
        }
        Ok(Version {
            major: parse_num(parts[0])?,
            minor: parse_num(parts[1])?,
            patch: parse_num(parts[2])?,
            pre: pre.map(str::to_string),
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(p) = &self.pre {
            write!(f, "-{p}")?;
        }
        Ok(())
    }
}

impl Version {
    pub fn bump(&self, level: Level) -> Result<Version, Error> {
        let inc = |n: u64| n.checked_add(1).ok_or(Error::Overflow);
        let (major, minor, patch) = match level {
            Level::Major => (inc(self.major)?, 0, 0),
            Level::Minor => (self.major, inc(self.minor)?, 0),
            Level::Patch => (self.major, self.minor, inc(self.patch)?),
        };
        Ok(Version { major, minor, patch, pre: None })
    }
}

/// Run the command line (arguments without the program name).
pub fn run(args: &[String]) -> Result<String, Error> {
    match args {
        [cmd, level, version] if cmd == "bump" => {
            let level: Level = level.parse()?;
            let version: Version = version.parse()?;
            Ok(version.bump(level)?.to_string())
        }
        _ => Err(Error::Usage),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Result<Version, Error> {
        s.parse()
    }

    #[test]
    fn bumps() {
        assert_eq!(v("1.4.2").unwrap().bump(Level::Minor).unwrap().to_string(), "1.5.0");
        assert_eq!(v("1.4.2").unwrap().bump(Level::Major).unwrap().to_string(), "2.0.0");
        assert_eq!(v("1.4.2").unwrap().bump(Level::Patch).unwrap().to_string(), "1.4.3");
        assert_eq!(v("0.0.0").unwrap().bump(Level::Patch).unwrap().to_string(), "0.0.1");
    }

    #[test]
    fn drops_prerelease_and_build() {
        assert_eq!(v("1.4.2-rc.1").unwrap().bump(Level::Patch).unwrap().to_string(), "1.4.3");
        assert_eq!(v("1.4.2-beta+build.5").unwrap().bump(Level::Major).unwrap().to_string(), "2.0.0");
        assert_eq!(v("1.4.2+b1").unwrap().bump(Level::Minor).unwrap().to_string(), "1.5.0");
    }

    #[test]
    fn parse_errors() {
        assert_eq!(v(""), Err(Error::WrongPartCount(1)));
        assert_eq!(v("1.2"), Err(Error::WrongPartCount(2)));
        assert_eq!(v("1.2.3.4"), Err(Error::WrongPartCount(4)));
        assert_eq!(v("1..3"), Err(Error::EmptyPart));
        assert_eq!(v("1.x.3"), Err(Error::NotANumber("x".into())));
        assert_eq!(v("v1.2.3"), Err(Error::NotANumber("v1".into())));
        assert_eq!(v(" 1.2.3"), Err(Error::NotANumber(" 1".into())));
        assert_eq!(v("1.-2.3"), Err(Error::WrongPartCount(2)));
        assert_eq!(v("01.2.3"), Err(Error::LeadingZero("01".into())));
        assert_eq!(v("1.2.99999999999999999999"), Err(Error::TooLarge("99999999999999999999".into())));
        assert_eq!(v("1.2.3-"), Err(Error::EmptyPreRelease));
        assert_eq!(v("1.2.3-a..b"), Err(Error::EmptyPreRelease));
        assert_eq!(v("1.2.3+"), Err(Error::EmptyBuild));
        assert_eq!(v("1.2.3-a b"), Err(Error::EmptyPreRelease));
    }

    #[test]
    fn overflow() {
        let max = format!("{}.0.0", u64::MAX);
        assert_eq!(v(&max).unwrap().bump(Level::Major), Err(Error::Overflow));
        let max = format!("0.0.{}", u64::MAX);
        assert_eq!(v(&max).unwrap().bump(Level::Patch), Err(Error::Overflow));
        assert_eq!(v(&max).unwrap().bump(Level::Minor).unwrap().to_string(), "0.1.0");
    }

    #[test]
    fn levels() {
        assert_eq!("minor".parse::<Level>(), Ok(Level::Minor));
        assert_eq!("Minor".parse::<Level>(), Err(Error::UnknownLevel("Minor".into())));
    }

    #[test]
    fn run_usage() {
        let a = |xs: &[&str]| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(run(&a(&["bump", "minor", "1.4.2"])), Ok("1.5.0".into()));
        assert_eq!(run(&a(&[])), Err(Error::Usage));
        assert_eq!(run(&a(&["bump", "minor"])), Err(Error::Usage));
        assert_eq!(run(&a(&["bump", "minor", "1.4.2", "x"])), Err(Error::Usage));
        assert_eq!(run(&a(&["raise", "minor", "1.4.2"])), Err(Error::Usage));
    }
}
