//! A fingerprint per piece of a model, worked out from what the piece
//! says and not from how it is written: comments, spacing and the order
//! of the declarations do not change it. Evidence said by a person
//! keeps the fingerprint of its piece as it was (`seen "…"`), and
//! counts for nothing once the piece says something else.

use crate::model::*;
use std::collections::BTreeMap;

/// Twelve hex digits of the FNV-1a hash of a text.
// mitra: implements fingerprint::hash
pub fn hash(text: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")[..12].to_string()
}

/// What each piece says, as one line: the text its fingerprint is of.
pub fn said(m: &Model) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for r in m.records.iter().filter(|r| r.of.is_none()) {
        let fields: Vec<String> = r
            .fields
            .iter()
            .map(|f| format!("{}: {}", f.0, f.1))
            .collect();
        out.insert(
            r.name.clone(),
            format!("record {{ {} }}", fields.join(", ")),
        );
    }
    for e in &m.enums {
        let body: Vec<&str> = e.body.split_whitespace().collect();
        let text = format!("enum <{}> {{ {} }}", e.params.join(", "), body.join(" "));
        out.insert(e.name.clone(), text);
    }
    for f in &m.functions {
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| format!("{}: {}", p.0, p.1))
            .collect();
        let body = f
            .body
            .as_ref()
            .map(crate::report::print::expr)
            .unwrap_or_default();
        let written = f.written.clone().unwrap_or_default();
        let text = format!(
            "fn ({}) -> {} {written} {{ {body} }}",
            params.join(", "),
            f.range
        );
        out.insert(f.name.clone(), text);
    }
    for c in &m.claims {
        if m.ensure(&c.name).is_some() || m.captured(&c.name).is_some() {
            continue;
        }
        let from = c.from.join(", ");
        let text = format!(
            "claim on {} {{ {} }} from [{from}]",
            c.func,
            crate::report::print::contract(m, c)
        );
        out.insert(c.name.clone(), text);
    }
    // A promise says its prose; its capture says the prose and every
    // claim the promise rests on, as each reads now.
    for e in &m.ensures {
        let needs = e.needs.clone().unwrap_or_default();
        out.insert(
            e.name.clone(),
            format!("ensure {:?} needs [{needs}]", e.prose),
        );
        let part =
            |c: &String| format!("{c} = {}", out.get(c).map(|t| hash(t)).unwrap_or_default());
        let mut on: Vec<String> = e.claims.iter().chain(&e.requires).map(part).collect();
        on.sort();
        let text = format!("capture of {:?} by [{}]", e.prose, on.join(", "));
        out.insert(format!("{}::captured", e.name), text);
    }
    out
}

/// The fingerprint of every piece, by its name.
// mitra: implements fingerprint::all
pub fn all(m: &Model) -> BTreeMap<String, String> {
    said(m).into_iter().map(|(k, v)| (k, hash(&v))).collect()
}

/// The fingerprint of the piece a name speaks of: `f::correct`,
/// `f::tied`, `f::uses` and the like speak of `f`.
// mitra: implements fingerprint::of
pub fn of<'a>(prints: &'a BTreeMap<String, String>, name: &str) -> Option<&'a String> {
    let mut name = name;
    loop {
        if let Some(p) = prints.get(name) {
            return Some(p);
        }
        name = name.rsplit_once("::")?.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // mitra: evidence for fingerprint::hash given text = "" gives "cbf29ce48422"
    #[test]
    fn t_hash_empty() {
        assert_eq!(hash(""), "cbf29ce48422");
    }

    // mitra: evidence for fingerprint::hash given text = "a" gives "af63dc4c8601"
    #[test]
    fn t_hash_a() {
        assert_eq!(hash("a"), "af63dc4c8601");
    }

    fn prints() -> BTreeMap<String, String> {
        BTreeMap::from([("s::f".to_string(), "aa".to_string())])
    }

    // mitra: evidence for fingerprint::t_of_the_piece
    #[test]
    fn t_of_the_piece() {
        assert_eq!(of(&prints(), "s::f"), Some(&"aa".to_string()));
    }

    // mitra: evidence for fingerprint::t_of_what_is_under_it
    #[test]
    fn t_of_what_is_under_it() {
        assert_eq!(of(&prints(), "s::f::correct"), Some(&"aa".to_string()));
    }

    // mitra: evidence for fingerprint::t_of_nothing
    #[test]
    fn t_of_nothing() {
        assert_eq!(of(&prints(), "s::g"), None);
    }
}
