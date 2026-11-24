//! semver: bumps a semantic version. The spec is `semver.mitra`.

/// A level to bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Major,
    Minor,
    Patch,
}

/// Why the tool fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    Usage,
    UnknownLevel,
    BadVersion,
}

impl Failure {
    pub fn message(self) -> &'static str {
        match self {
            Failure::Usage => "usage: semver bump <major|minor|patch> <version>",
            Failure::UnknownLevel => "unknown level: expected major, minor or patch",
            Failure::BadVersion => "bad version: expected MAJOR.MINOR.PATCH[-PRE]",
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Failure::Usage => 2,
            Failure::UnknownLevel => 3,
            Failure::BadVersion => 4,
        }
    }
}

/// A version `MAJOR.MINOR.PATCH[-PRE]`. The fields are private so that
/// every `Version` renders to a text `parse` reads back: a pre-release is
/// never empty and never holds `+`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<String>,
}

/// Reads one number: decimal digits only, no sign, no leading zero but `0`.
fn number(t: &str) -> Result<u64, Failure> {
    let digits = !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    let leading_zero = t.len() > 1 && t.starts_with('0');
    if !digits || leading_zero {
        return Err(Failure::BadVersion);
    }
    t.parse().map_err(|_| Failure::BadVersion)
}

impl Version {
    /// A version, or `None` when the pre-release is empty or holds `+`.
    pub fn new(major: u64, minor: u64, patch: u64, pre: Option<&str>) -> Option<Version> {
        match pre {
            Some(p) if p.is_empty() || p.contains('+') => None,
            _ => Some(Version { major, minor, patch, pre: pre.map(String::from) }),
        }
    }

    pub fn major(&self) -> u64 {
        self.major
    }

    pub fn minor(&self) -> u64 {
        self.minor
    }

    pub fn patch(&self) -> u64 {
        self.patch
    }

    pub fn pre(&self) -> Option<&str> {
        self.pre.as_deref()
    }

    /// The bumped version, or `None` when the number at the level is the
    /// largest the tool can hold.
    // mitra: implements Version::bump
    pub fn bump(&self, l: Level) -> Option<Version> {
        Some(match l {
            Level::Major => Version { major: self.major.checked_add(1)?, minor: 0, patch: 0, pre: None },
            Level::Minor => Version { minor: self.minor.checked_add(1)?, patch: 0, pre: None, ..self.clone() },
            Level::Patch => Version { patch: self.patch.checked_add(1)?, pre: None, ..self.clone() },
        })
    }

    // mitra: implements Version::parse
    pub fn parse(s: &str) -> Result<Version, Failure> {
        if s.contains('+') {
            return Err(Failure::BadVersion);
        }
        let (core, pre) = match s.split_once('-') {
            Some((_, "")) => return Err(Failure::BadVersion),
            Some((core, pre)) => (core, Some(pre.to_string())),
            None => (s, None),
        };
        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            return Err(Failure::BadVersion);
        }
        Ok(Version { major: number(parts[0])?, minor: number(parts[1])?, patch: number(parts[2])?, pre })
    }

    // mitra: implements Version::render
    pub fn render(&self) -> String {
        match &self.pre {
            None => format!("{}.{}.{}", self.major, self.minor, self.patch),
            Some(p) => format!("{}.{}.{}-{}", self.major, self.minor, self.patch, p),
        }
    }
}

// mitra: implements Cli::level
pub fn level(t: &str) -> Result<Level, Failure> {
    match t {
        "major" => Ok(Level::Major),
        "minor" => Ok(Level::Minor),
        "patch" => Ok(Level::Patch),
        _ => Err(Failure::UnknownLevel),
    }
}

/// What `semver` prints, or why it fails: the shape of the command, then
/// the level, then the version.
// mitra: implements Cli::run
pub fn run<S: AsRef<str>>(args: &[S]) -> Result<String, Failure> {
    if args.len() != 3 || args[0].as_ref() != "bump" {
        return Err(Failure::Usage);
    }
    let l = level(args[1].as_ref())?;
    let v = Version::parse(args[2].as_ref())?;
    v.bump(l).map(|w| w.render()).ok_or(Failure::BadVersion)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(major: u64, minor: u64, patch: u64, pre: Option<&str>) -> Version {
        Version::new(major, minor, patch, pre).unwrap()
    }

    const LEVELS: [Level; 3] = [Level::Major, Level::Minor, Level::Patch];
    const PRES: [Option<&str>; 5] = [None, Some("rc.1"), Some("alpha"), Some("0"), Some("x-y.z")];

    /// Small versions, with and without a pre-release.
    fn versions() -> Vec<Version> {
        let mut all = Vec::new();
        for major in 0..4 {
            for minor in [0, 1, 9, 10] {
                for patch in [0, 1, 99] {
                    for pre in PRES {
                        all.push(v(major, minor, patch, pre));
                    }
                }
            }
        }
        all.push(v(u64::MAX, u64::MAX, u64::MAX, None));
        all
    }

    // ---- Version::bump

    // mitra: measures Version::bump (Version { major: 1, minor: 4, patch: 2, pre: None }, Level::Minor) -> Version { major: 1, minor: 5, patch: 0, pre: None }
    #[test]
    fn bump_minor() {
        assert_eq!(v(1, 4, 2, None).bump(Level::Minor), Some(v(1, 5, 0, None)));
    }

    // mitra: measures Version::bump (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Level::Major) -> Version { major: 2, minor: 0, patch: 0, pre: None }
    #[test]
    fn bump_major_drops_pre() {
        assert_eq!(v(1, 4, 2, Some("rc.1")).bump(Level::Major), Some(v(2, 0, 0, None)));
    }

    // mitra: measures Version::bump (Version { major: 0, minor: 9, patch: 9, pre: None }, Level::Patch) -> Version { major: 0, minor: 9, patch: 10, pre: None }
    #[test]
    fn bump_patch() {
        assert_eq!(v(0, 9, 9, None).bump(Level::Patch), Some(v(0, 9, 10, None)));
    }

    #[test]
    fn bump_overflow_is_none() {
        assert_eq!(v(u64::MAX, 0, 0, None).bump(Level::Major), None);
    }

    #[test]
    fn new_refuses_what_does_not_read_back() {
        assert_eq!(Version::new(1, 2, 3, Some("")), None);
        assert_eq!(Version::new(1, 2, 3, Some("a+b")), None);
    }

    /// The claims `drops_pre`, `grows` and `one_step` of `bump`, over every
    /// small version and every level.
    #[test]
    fn bump_claims_hold() {
        for s in versions() {
            for l in LEVELS {
                let Some(w) = s.bump(l) else {
                    assert_eq!(s.major, u64::MAX);
                    continue;
                };
                let (wm, wn, wp) = (w.major as i128, w.minor as i128, w.patch as i128);
                let (sm, sn, sp) = (s.major as i128, s.minor as i128, s.patch as i128);
                assert_eq!(w.pre, None, "drops_pre {s:?} {l:?}");
                assert!(wm > sm || (wm == sm && wn > sn) || (wm == sm && wn == sn && wp > sp), "grows {s:?} {l:?}");
                assert!(wm - sm + wn - sn <= 1 || wn == 0, "one_step {s:?} {l:?}");
            }
        }
    }

    // ---- Version::parse

    // mitra: measures Version::parse ("1.4.2") -> Ok(Version { major: 1, minor: 4, patch: 2, pre: None })
    #[test]
    fn parse_plain() {
        assert_eq!(Version::parse("1.4.2"), Ok(v(1, 4, 2, None)));
    }

    // mitra: measures Version::parse ("1.4.10") -> Ok(Version { major: 1, minor: 4, patch: 10, pre: None })
    #[test]
    fn parse_two_digits() {
        assert_eq!(Version::parse("1.4.10"), Ok(v(1, 4, 10, None)));
    }

    // mitra: measures Version::parse ("0.0.0") -> Ok(Version { major: 0, minor: 0, patch: 0, pre: None })
    #[test]
    fn parse_zeros() {
        assert_eq!(Version::parse("0.0.0"), Ok(v(0, 0, 0, None)));
    }

    // mitra: measures Version::parse ("1.4.2-rc.1") -> Ok(Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") })
    #[test]
    fn parse_pre() {
        assert_eq!(Version::parse("1.4.2-rc.1"), Ok(v(1, 4, 2, Some("rc.1"))));
    }

    // mitra: measures Version::parse ("") -> Err(Failure::BadVersion)
    #[test]
    fn parse_empty() {
        assert_eq!(Version::parse(""), Err(Failure::BadVersion));
    }

    // mitra: measures Version::parse ("1.4") -> Err(Failure::BadVersion)
    #[test]
    fn parse_two_parts() {
        assert_eq!(Version::parse("1.4"), Err(Failure::BadVersion));
    }

    // mitra: measures Version::parse ("1.4.2.0") -> Err(Failure::BadVersion)
    #[test]
    fn parse_four_parts() {
        assert_eq!(Version::parse("1.4.2.0"), Err(Failure::BadVersion));
    }

    // mitra: measures Version::parse ("01.4.2") -> Err(Failure::BadVersion)
    #[test]
    fn parse_leading_zero() {
        assert_eq!(Version::parse("01.4.2"), Err(Failure::BadVersion));
    }

    // mitra: measures Version::parse ("1.x.2") -> Err(Failure::BadVersion)
    #[test]
    fn parse_letter() {
        assert_eq!(Version::parse("1.x.2"), Err(Failure::BadVersion));
    }

    // mitra: measures Version::parse ("-1.4.2") -> Err(Failure::BadVersion)
    #[test]
    fn parse_sign() {
        assert_eq!(Version::parse("-1.4.2"), Err(Failure::BadVersion));
    }

    // mitra: measures Version::parse ("1.4.2-") -> Err(Failure::BadVersion)
    #[test]
    fn parse_empty_pre() {
        assert_eq!(Version::parse("1.4.2-"), Err(Failure::BadVersion));
    }

    // mitra: measures Version::parse ("1.4.2+build") -> Err(Failure::BadVersion)
    #[test]
    fn parse_build() {
        assert_eq!(Version::parse("1.4.2+build"), Err(Failure::BadVersion));
    }

    /// The claim `empty` of `parse`, and texts close to a version.
    #[test]
    fn parse_refuses_near_misses() {
        for t in ["", ".", "..", "1..2", "1.2.", " 1.2.3", "1.2.3 ", "1.2.3-+", "1.2.3-a+b", "+1.2.3", "1.02.3", "1.2.03", "99999999999999999999.0.0"] {
            assert_eq!(Version::parse(t), Err(Failure::BadVersion), "{t:?}");
        }
    }

    // ---- Version::render

    // mitra: measures Version::render (Version { major: 1, minor: 5, patch: 0, pre: None }) -> "1.5.0"
    #[test]
    fn render_plain() {
        assert_eq!(v(1, 5, 0, None).render(), "1.5.0");
    }

    // mitra: measures Version::render (Version { major: 1, minor: 2, patch: 3, pre: Some("rc.1") }) -> "1.2.3-rc.1"
    #[test]
    fn render_pre() {
        assert_eq!(v(1, 2, 3, Some("rc.1")).render(), "1.2.3-rc.1");
    }

    /// The claim `round_trip` of `render`, over every small version.
    #[test]
    fn render_round_trips() {
        for s in versions() {
            assert_eq!(Version::parse(&s.render()), Ok(s.clone()), "{s:?}");
        }
    }

    // ---- Cli::level

    // mitra: measures Cli::level ("minor") -> Ok(Level::Minor)
    #[test]
    fn level_minor() {
        assert_eq!(level("minor"), Ok(Level::Minor));
    }

    // mitra: measures Cli::level ("Minor") -> Err(Failure::UnknownLevel)
    #[test]
    fn level_case_matters() {
        assert_eq!(level("Minor"), Err(Failure::UnknownLevel));
    }

    // ---- Cli::run

    // mitra: measures Cli::run (["bump", "minor", "1.4.2"]) -> Ok("1.5.0")
    #[test]
    fn run_minor() {
        assert_eq!(run(&["bump", "minor", "1.4.2"]), Ok("1.5.0".to_string()));
    }

    // mitra: measures Cli::run (["bump", "major", "1.4.2-rc.1"]) -> Ok("2.0.0")
    #[test]
    fn run_major_pre() {
        assert_eq!(run(&["bump", "major", "1.4.2-rc.1"]), Ok("2.0.0".to_string()));
    }

    // mitra: measures Cli::run (["bump", "patch", "1.4.9"]) -> Ok("1.4.10")
    #[test]
    fn run_patch() {
        assert_eq!(run(&["bump", "patch", "1.4.9"]), Ok("1.4.10".to_string()));
    }

    /// The claim `usage` of `run`: any other number of words is a usage failure.
    #[test]
    fn run_usage_claim_holds() {
        let words = ["bump", "minor", "1.4.2", "x"];
        for n in 0..=4 {
            let r = run(&words[..n]);
            assert!(n == 3 || r == Err(Failure::Usage), "{n}");
        }
        assert_eq!(run(&["bump", "minor", "1.4.2", "1.4.2"]), Err(Failure::Usage));
    }

    // mitra: measures Cli::run (["bump", "minor"]) -> Err(Failure::Usage)
    #[test]
    fn run_too_few() {
        assert_eq!(run(&["bump", "minor"]), Err(Failure::Usage));
    }

    // mitra: measures Cli::run (["grow", "minor", "1.4.2"]) -> Err(Failure::Usage)
    #[test]
    fn run_not_bump() {
        assert_eq!(run(&["grow", "minor", "1.4.2"]), Err(Failure::Usage));
    }

    // mitra: measures Cli::run (["bump", "huge", "1.4.2"]) -> Err(Failure::UnknownLevel)
    #[test]
    fn run_unknown_level() {
        assert_eq!(run(&["bump", "huge", "1.4.2"]), Err(Failure::UnknownLevel));
    }

    // mitra: measures Cli::run (["bump", "huge", "1.4"]) -> Err(Failure::UnknownLevel)
    #[test]
    fn run_level_before_version() {
        assert_eq!(run(&["bump", "huge", "1.4"]), Err(Failure::UnknownLevel));
    }

    // mitra: measures Cli::run (["bump", "minor", "1.4"]) -> Err(Failure::BadVersion)
    #[test]
    fn run_bad_version() {
        assert_eq!(run(&["bump", "minor", "1.4"]), Err(Failure::BadVersion));
    }
}
