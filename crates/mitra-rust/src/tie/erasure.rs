//! The Rust profile of erasures: what a signature of the code may say that
//! the model leaves out, each at a cost.

use super::*;

/// What this reader may forget of the code to reach the model, each
/// with what it costs the tie: `1 - cost` is what is left of its trust.
pub const PROFILE: [(&str, f64); 11] = [
    // `&`, `&mut`, `mut`, a lifetime.
    ("reference", 0.0),
    // `Box`, `Rc`, `Arc`.
    ("pointer", 0.01),
    // `Cow<str>`, `PathBuf`.
    ("owned", 0.01),
    // `impl Into<T>`, `impl AsRef<T>`.
    ("conversion", 0.02),
    // `impl Iterator<Item = T>` read as a list.
    ("sequence", 0.03),
    // The width of the numbers of a signature.
    ("width", 0.02),
    // A trait that bounds an input in the code alone.
    ("bound", 0.02),
    // A type the model states narrower than the code can.
    ("range", 0.05),
    // A type the model leaves as any value.
    ("any", 0.05),
    // The type of the error of a `Result`.
    ("error", 0.05),
    // A field of a struct the record does not state, without `..`.
    ("field", 0.05),
];

/// An erasure taken on the path of a tie: its name and on what.
pub(super) type Erased = Vec<(&'static str, String)>;

/// What is forgotten of one type of the code to read it as the model's.
pub(super) fn forgot(on: &str, rust: &str, model: &str, opaque: bool, into: &mut Erased) {
    let has = |words: &[&str]| words.iter().any(|w| rust.contains(w));
    let lifetime = rust
        .match_indices('\'')
        .any(|(i, _)| rust[i + 1..].starts_with(|c: char| c.is_alphabetic() || c == '_'));
    let mut took = |name: &'static str, what: String| into.push((name, what));
    if has(&["&", "mut "]) || lifetime {
        took("reference", on.to_string());
    }
    if has(&["Box<", "Rc<", "Arc<"]) {
        took("pointer", on.to_string());
    }
    if has(&["Cow<", "PathBuf"]) {
        took("owned", on.to_string());
    }
    if has(&["Into<", "AsRef<"]) {
        took("conversion", on.to_string());
    }
    if has(&["Iterator<"]) {
        took("sequence", on.to_string());
    }
    if opaque {
        // A type parameter on both sides: nothing is forgotten.
    } else if model.contains("Any") {
        took("any", format!("{on}: {rust}"));
    } else if ["..", "{", "|", "\\", "&"]
        .iter()
        .any(|n| model.contains(n))
    {
        took("range", format!("{on}: {model} for {rust}"));
    }
}

/// `Result<T, E>` as its two types; `io::Result<T>` has its error unsaid.
pub(super) fn result_of(ty: &str) -> Option<(String, Option<String>)> {
    let t = ty.trim();
    let open = t.find('<')?;
    if t[..open].rsplit("::").next()? != "Result" {
        return None;
    }
    let parts = pieces(t[open + 1..].strip_suffix('>')?);
    match parts.as_slice() {
        [ok] => Some((ok.clone(), None)),
        [ok, err] => Some((ok.clone(), Some(err.clone()))),
        _ => None,
    }
}
