//! The settings of a store: the numbers the trust of a tie and of a
//! sample are computed from. Read from the text of `settings.toml`,
//! handed by whoever read the file.

use std::collections::BTreeMap;

/// The settings in force. Absent from the file, a value is its default.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements settings::Settings
pub struct Settings {
    /// The share of its domain a sampled claim may fail on unseen.
    pub tolerance: f64,
    /// Whether a passing sample lets a claim read `holds assuming
    /// [c::sample]`. Not by default: a sample then covers nothing.
    pub counts: bool,
    /// The cost of each erasure of a reader, by name, where the file
    /// says one: the reader has its own for the others.
    pub erasure: BTreeMap<String, f64>,
    /// How often a word of each kind of voice is wrong, where the file
    /// says: `agent`, `cold`, `human`, `machine`, `proof`, `unknown`.
    pub doubt: BTreeMap<String, f64>,
    /// What a piece must rest on to be within policy, by kind of piece
    /// (`claim`, `ensure`). Nothing asked by default.
    pub policy: BTreeMap<String, String>,
    /// How far a piece of evidence of a kind is believed before anyone
    /// reviewed it, where the file says: `test`. With none, no verdict
    /// carries a confidence.
    pub bias: BTreeMap<String, f64>,
    /// The confidence a claim needs to hold, `[policy] holds`. With
    /// none, no verdict changes.
    pub holds: Option<f64>,
}

/// The kinds of voice, each with how often its word is taken to be
/// wrong when the file says nothing.
pub const DOUBT: [(&str, f64); 6] = [
    ("agent", 0.3),
    ("cold", 0.15),
    ("human", 0.05),
    ("machine", 0.1),
    ("proof", 0.01),
    ("unknown", 0.5),
];

impl Default for Settings {
    fn default() -> Self {
        Settings {
            tolerance: 0.001,
            counts: false,
            erasure: BTreeMap::new(),
            doubt: BTreeMap::new(),
            policy: BTreeMap::new(),
            bias: BTreeMap::new(),
            holds: None,
        }
    }
}

impl Settings {
    /// The settings a file states: `[sample]` with `tolerance` and
    /// `counts`, `[erasure]` with a cost from 0 to 1 by name. The part of
    /// TOML read is this: tables, `key = value`, numbers, `true`, `false`,
    /// comments.
    // mitra: implements settings::Settings::read
    pub fn read(text: &str) -> Result<Settings, String> {
        let mut s = Settings::default();
        let mut table = String::new();
        for (i, line) in text.lines().enumerate() {
            let at = |m: String| format!("line {}: {m}", i + 1);
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                table = name.trim().to_string();
                if !["sample", "erasure", "doubt", "policy", "bias"].contains(&table.as_str()) {
                    return Err(at(format!(
                        "no settings `[{table}]` (`[sample]`, `[erasure]`, `[doubt]`, `[policy]`, `[bias]`)"
                    )));
                }
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(at(format!("`{line}` is not `key = value`")));
            };
            let (key, value) = (key.trim(), value.trim());
            let share = || match value.parse::<f64>() {
                Ok(n) if (0.0..=1.0).contains(&n) => Ok(n),
                _ => Err(at(format!(
                    "`{key}` is a number from 0 to 1, not `{value}`"
                ))),
            };
            match (table.as_str(), key) {
                ("sample", "tolerance") => {
                    s.tolerance = share()?;
                    if s.tolerance == 0.0 {
                        return Err(at("`tolerance` is more than 0".to_string()));
                    }
                }
                ("sample", "counts") => {
                    s.counts = match value {
                        "true" => true,
                        "false" => false,
                        _ => {
                            return Err(at(format!("`counts` is `true` or `false`, not `{value}`")))
                        }
                    }
                }
                ("erasure", _) => {
                    s.erasure.insert(key.to_string(), share()?);
                }
                ("doubt", k) if DOUBT.iter().any(|d| d.0 == k) => {
                    s.doubt.insert(key.to_string(), share()?);
                }
                ("bias", "test") => {
                    s.bias.insert(key.to_string(), share()?);
                }
                ("policy", "holds") => s.holds = Some(share()?),
                ("policy", "claim" | "ensure") => {
                    let text = value.strip_prefix('"').and_then(|v| v.strip_suffix('"'));
                    let Some(text) = text else {
                        return Err(at(format!("`{key}` is a text in quotes, not `{value}`")));
                    };
                    crate::ledger::voices::policy_reads(text).map_err(at)?;
                    s.policy.insert(key.to_string(), text.to_string());
                }
                ("", _) => {
                    return Err(at(format!(
                        "`{key}` is under no table: `[sample]` or `[erasure]`"
                    )))
                }
                _ => return Err(at(format!("no setting `{key}` in `[{table}]`"))),
            }
        }
        Ok(s)
    }

    /// The trust of a sample of `n` draws that all passed: the chance
    /// that `n` draws would have met a failure, were the claim to fail on
    /// more than the tolerance of its domain.
    // mitra: implements settings::Settings::sample
    pub fn sample(&self, n: u64) -> f64 {
        1.0 - (1.0 - self.tolerance).powf(n as f64)
    }

    /// How often a word of this kind of voice is wrong.
    // mitra: implements settings::Settings::doubt
    pub fn doubt(&self, kind: &str) -> f64 {
        let own = DOUBT.iter().find(|d| d.0 == kind).map_or(1.0, |d| d.1);
        self.doubt.get(kind).copied().unwrap_or(own)
    }

    /// The trust of a tie whose path took these erasures: the product of
    /// what each leaves.
    // mitra: implements settings::Settings::strength
    pub fn strength(costs: &[f64]) -> f64 {
        costs.iter().map(|c| 1.0 - c).product()
    }

    /// The settings in force as a file, with the costs of a reader's
    /// profile where the file says none.
    // mitra: implements settings::Settings::written
    pub fn written(&self, profile: &[(&str, f64)]) -> String {
        let mut out = format!(
            "[sample]\ntolerance = {}\ncounts = {}\n\n[erasure]\n",
            self.tolerance, self.counts
        );
        for (name, cost) in profile {
            let cost = self.erasure.get(*name).unwrap_or(cost);
            out += &format!("{name} = {cost}\n");
        }
        out += "\n[doubt]\n";
        for (kind, _) in DOUBT {
            out += &format!("{kind} = {}\n", self.doubt(kind));
        }
        out += "\n[policy]\n";
        for kind in ["claim", "ensure"] {
            let asked = self.policy.get(kind).map_or("any", |p| p.as_str());
            out += &format!("{kind} = \"{asked}\"\n");
        }
        if let Some(h) = self.holds {
            out += &format!("holds = {h}\n");
        }
        if !self.bias.is_empty() {
            out += "\n[bias]\n";
            for (kind, b) in &self.bias {
                out += &format!("{kind} = {b}\n");
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // mitra: evidence for settings::absent_is_default
    #[test]
    fn absent_is_default() {
        assert_eq!(Settings::read(""), Ok(Settings::default()));
        assert_eq!(
            Settings::read("# nothing\n[sample]\n"),
            Ok(Settings::default())
        );
    }

    // mitra: evidence for settings::file_is_read
    #[test]
    fn file_is_read() {
        let s = Settings::read(
            "[sample]\ntolerance = 0.01 # loose\ncounts = true\n[erasure]\nwidth = 0.5\n",
        )
        .unwrap();
        assert_eq!((s.tolerance, s.counts), (0.01, true));
        assert_eq!(s.erasure.get("width"), Some(&0.5));
    }

    // mitra: evidence for settings::unknown_setting_is_refused
    #[test]
    fn unknown_setting_is_refused() {
        assert!(Settings::read("[other]\n").is_err());
        assert!(Settings::read("[sample]\nmore = 1\n").is_err());
        assert!(Settings::read("[erasure]\nwidth = 2\n").is_err());
        assert!(Settings::read("tolerance = 0.1\n").is_err());
        assert!(Settings::read("[sample]\ntolerance = 0\n").is_err());
    }

    // mitra: evidence for settings::sample_grows_less
    #[test]
    fn sample_grows_less() {
        let s = Settings::default();
        let (a, b, c) = (s.sample(0), s.sample(1000), s.sample(2000));
        assert_eq!(a, 0.0);
        assert!((b - 0.632).abs() < 0.001, "{b}");
        assert!(c > b && c - b < b - a && c < 1.0);
    }

    // mitra: evidence for settings::strength_is_a_product
    #[test]
    fn strength_is_a_product() {
        assert_eq!(Settings::strength(&[]), 1.0);
        assert!((Settings::strength(&[0.5, 0.5]) - 0.25).abs() < 1e-12);
        assert_eq!(Settings::strength(&[0.0, 0.0]), 1.0);
    }

    // mitra: evidence for settings::written_is_read_back
    #[test]
    fn written_is_read_back() {
        let s = Settings::read("[erasure]\nwidth = 0.5\n").unwrap();
        let text = s.written(&[("width", 0.02), ("field", 0.05)]);
        assert!(text.contains("width = 0.5\nfield = 0.05\n"), "{text}");
        let back = Settings::read(&text).unwrap();
        assert_eq!(back.erasure.get("field"), Some(&0.05));
        assert_eq!(back.tolerance, s.tolerance);
    }
}
