//! Reading evidence: an `evidence` block, the entries of a ledger, the
//! stamp `mitra add` writes.

use crate::reader::prelude::*;

/// How an error on a name the model does not have starts.
pub(crate) const NO_PIECE: &str = "the model has no function or claim";

/// One evidence block of a ledger, read without a model.
#[derive(Debug, Clone, Default, PartialEq)]
// mitra: implements ledger::Entry
pub struct Entry {
    /// The piece it names, as written.
    pub about: String,
    pub source: Option<String>,
    pub rev: Option<String>,
    pub seen: Option<String>,
    /// Its first and its last line.
    pub first: usize,
    pub last: usize,
}

impl P {
    /// evidence name { for target  given a = 1, b in 0..=9  holds  trust 1  source "…" }
    pub(crate) fn evidence(&mut self) -> R<Evidence> {
        let line = self.line();
        self.expect("evidence")?;
        let label = self.name()?;
        self.expect("{")?;
        self.expect("for")?;
        let target = self.path()?;
        let target = self.find(&target)?;
        let (mut given, mut says, mut trust) = (Vec::new(), None, 1.0);
        let (mut source, mut rev, mut seen) = (label, None, None);
        // `sampled 1000`: the count, whether it passed, and the domain as
        // written (`m in valid, o in valid`).
        let mut sampled: Option<(u64, bool)> = None;
        let mut domain: Vec<String> = Vec::new();
        let mut by_helper = false;
        let clauses = [
            "given", "holds", "fails", "passes", "gives", "sampled", "trust", "source", "rev",
            "seen", "act", "by", "written", "cold", "of",
        ];
        let (mut act, mut by, mut written, mut cold, mut of) = (None, None, None, false, None);
        while !self.eat("}") {
            let at = self.line();
            let clause = self.name()?;
            match clause.as_str() {
                "given" => loop {
                    let x = self.name()?;
                    let exact = self.eat("=");
                    if !exact && !self.eat("in") && !self.eat(":") {
                        return self.err(format!("expected `{x} = value` or `{x} in set`"));
                    }
                    // `m in valid`: the values a helper of the model says
                    // yes to. A domain to sample from, never a set to cover.
                    let helper = (!exact).then(|| self.resolve(self.peek())).filter(|h| {
                        let one = self.names.fns.get(h).is_some_and(|s| s.params.len() == 1);
                        one && !self.names.opaque.contains(h)
                    });
                    let from = self.pos;
                    if helper.is_some() {
                        self.pos += 1;
                        by_helper = true;
                        domain.push(format!("{x} in {}", join(&self.toks[from..self.pos])));
                    } else {
                        let set = match exact {
                            true => Set::point(&self.value()?),
                            false => self.set()?,
                        };
                        let word = if exact { "=" } else { "in" };
                        domain.push(format!("{x} {word} {}", join(&self.toks[from..self.pos])));
                        given.push((x, exact, set));
                    }
                    if !self.eat(",") {
                        break;
                    }
                },
                "holds" | "fails" if sampled.is_some() => {
                    sampled = sampled.map(|s| (s.0, clause == "holds"));
                }
                "sampled" => {
                    let n = self.number()?;
                    if n < 1.0 || n.fract() != 0.0 {
                        return Err(Error::new(at, "`sampled` takes how many inputs were drawn"));
                    }
                    sampled = Some((n as u64, true));
                    says = Some((clause, None, 0.0));
                }
                "holds" | "fails" | "passes" => says = Some((clause, None, 0.0)),
                "gives" if self.is("rate") && self.peek_at(1) == "=" => {
                    self.pos += 2;
                    let negative = self.eat("-");
                    let x = self.number()?;
                    says = Some(("rate".to_string(), None, if negative { -x } else { x }));
                }
                "gives" => says = Some((clause, Some(self.value()?), 0.0)),
                "trust" => {
                    trust = self.number()?;
                    if !(0.0..=1.0).contains(&trust) {
                        return Err(Error::new(
                            at,
                            format!("trust {trust}: it goes from 0 to 1"),
                        ));
                    }
                }
                "act" => {
                    let name = self.name()?;
                    if !crate::model::ACTS.contains(&name.as_str()) {
                        let all = crate::model::ACTS.join(", ");
                        return Err(Error::new(at, format!("no act `{name}` ({all})")));
                    }
                    // `act sample 1000` is `sampled 1000`.
                    if name == "sample" && self.kind() == Some(Kind::Num) {
                        let n = self.number()?;
                        if n < 1.0 || n.fract() != 0.0 {
                            return Err(Error::new(
                                at,
                                "`sample` takes how many inputs were drawn",
                            ));
                        }
                        sampled = Some((n as u64, sampled.is_none_or(|s| s.1)));
                        says = Some(("sampled".to_string(), None, 0.0));
                    }
                    act = Some(name);
                }
                "cold" => cold = true,
                "source" | "rev" | "seen" | "by" | "written" | "of" => {
                    let Some(Old::Str(s)) = self.next_old() else {
                        self.pos -= 1;
                        return self.err(format!("`{clause}` needs a string"));
                    };
                    let voice = ["by", "written"].contains(&clause.as_str());
                    if voice && crate::model::voice_parts(&s).is_none() {
                        return Err(Error::new(
                            at,
                            format!(
                                "`{s}` is no voice: `agent:vendor/model/session`, \
                                 `human:name` or `machine:tool`"
                            ),
                        ));
                    }
                    match clause.as_str() {
                        "rev" => rev = Some(s),
                        "seen" => seen = Some(s),
                        "by" => by = Some(s),
                        "written" => written = Some(s),
                        "of" => of = Some(s),
                        _ => source = s,
                    }
                }
                other => {
                    let msg = format!("expected one of {}, got `{other}`", clauses.join(", "));
                    return Err(Error::new(at, msg));
                }
            }
        }
        let Some((says, value, rate)) = says else {
            return Err(Error::new(
                line,
                "the evidence says nothing: `holds`, `fails`, `passes`, `sampled n` or `gives …`",
            ));
        };
        let at = |m: String| Err(Error::new(line, m));
        let function = self.names.fns.get(&target).cloned();
        let names: Vec<String> = match (&function, self.names.claims.get(&target)) {
            (Some(sig), _) => sig.params.iter().map(|p| p.0.clone()).collect(),
            (None, Some(claim)) => claim.0.clone(),
            (None, None) => return at(format!("the model has no `{target}`")),
        };
        if let Some((x, _, _)) = given.iter().find(|g| !names.contains(&g.0)) {
            return at(format!("`{target}` has no input `{x}`"));
        }
        // Each input: its value, a set, or every value.
        let each = |x: &String| match given.iter().find(|g| &g.0 == x) {
            Some((_, true, v)) => Position {
                var: None,
                set: Some(v.clone()),
            },
            Some((_, false, set)) => Position {
                var: Some(x.clone()),
                set: Some(set.clone()),
            },
            None => Position {
                var: Some(x.clone()),
                set: None,
            },
        };
        let mut pattern: Pattern = names.iter().map(each).collect();
        let mut about = target.clone();
        let judgment = match (says.as_str(), function.is_some(), value) {
            ("rate", _, _) => {
                let measures = self.names.claims.get(&target).map(|c| c.1.clone());
                match measures.as_deref() {
                    Some([one]) => {
                        about = crate::report::print::expr(one);
                        pattern.clear();
                        Judgment::Measured(one.clone(), rate)
                    }
                    _ => return at(format!("`{target}` is not a claim on one measure")),
                }
            }
            ("sampled", _, _) => {
                if given.iter().any(|g| g.1) {
                    return at("a sample is drawn from a set: `given a in …`".to_string());
                }
                let (count, ok) = sampled.expect("read with the clause");
                let over = Some(std::mem::take(&mut pattern)).filter(|_| !by_helper);
                Judgment::Sampled {
                    count,
                    ok,
                    domain: domain.join(", "),
                    over,
                }
            }
            _ if by_helper => {
                return at(
                    "a helper names a domain to sample from: write `sampled 1000`".to_string(),
                )
            }
            // A test that states no input: on a claim or a function with
            // inputs it supports and settles nothing; with none, it holds.
            ("passes", _, _) if !given.is_empty() => {
                return at("`passes` states no input: with `given`, write `holds`".to_string())
            }
            ("passes", _, _) if !names.is_empty() => {
                pattern.clear();
                Judgment::Passed
            }
            ("passes", true, _) => Judgment::Correct(true),
            ("passes", false, _) => Judgment::Holds(true),
            ("gives", true, Some(v)) => Judgment::Output(v),
            ("gives", _, _) => return at(format!("`{target}` is a claim: `holds` or `fails`")),
            (says, true, _) => Judgment::Correct(says == "holds"),
            // The whole claim, or the claim on some inputs.
            (says, false, _) => {
                if given.is_empty() {
                    pattern.clear();
                }
                Judgment::Holds(says == "holds")
            }
        };
        // What an act can hold is checked against what the block says
        // and what it is on.
        if let Some(a) = act.as_deref() {
            let whole = self
                .names
                .claims
                .get(&target)
                .is_some_and(|c| c.0.is_empty())
                && function.is_none();
            let point = given.iter().all(|g| g.1) && (given.len() == names.len() || whole);
            let kept = ["::captured", "::tied", "::uses"]
                .iter()
                .find(|s| about.ends_with(*s));
            let judged = matches!(judgment, Judgment::Holds(_) | Judgment::Correct(_));
            let denies = matches!(
                judgment,
                Judgment::Holds(false)
                    | Judgment::Correct(false)
                    | Judgment::Sampled { ok: false, .. }
            );
            let why = match (a, &judgment) {
                ("report", _) if !denies => Some("a report is of a failure: it says `fails`"),
                ("report", _) => None,
                ("tie", _) if !matches!(kept, Some(&"::tied") | Some(&"::uses")) => {
                    Some("a tie is on `f::tied` or `f::uses`")
                }
                ("test" | "sample" | "exhaustive" | "measure", _) if kept.is_some() => {
                    Some("it is not about an input: it takes a review, a proof or a tie")
                }
                ("sample", Judgment::Sampled { .. }) => None,
                ("sample", _) => {
                    Some("a sample says how many inputs were drawn: `act sample 1000`")
                }
                (_, Judgment::Sampled { .. }) => Some("inputs that were drawn are `act sample`"),
                ("measure", Judgment::Measured(..)) => None,
                ("measure", _) => Some("a measure gives a rate: `gives rate = 0.9`"),
                (_, Judgment::Measured(..)) => Some("a rate is `act measure`"),
                ("test", Judgment::Output(_) | Judgment::Passed) => None,
                ("test", _) if judged && point => None,
                ("test", _) => Some(
                    "a test is a point: over a set of inputs it is `act exhaustive` or \
                     `act sample n`, and on a whole claim a review or a proof",
                ),
                ("exhaustive", _) if judged && !given.is_empty() => None,
                ("exhaustive", _) => {
                    Some("an exhaustive run states the set it ran through: `given a in …`")
                }
                ("review" | "proof", _) if judged => None,
                ("review" | "proof", _) => Some("a review or a proof says `holds` or `fails`"),
                _ => None,
            };
            if let Some(why) = why {
                return at(format!("`act {a}` cannot say this of `{about}`: {why}"));
            }
        }
        if of.is_some() && !matches!(judgment, Judgment::Holds(_) | Judgment::Correct(_)) {
            return at("a review `of` evidence says `holds` or `fails`".to_string());
        }
        Ok(Evidence {
            about,
            pattern,
            judgment,
            confidence: trust,
            source: Some(source),
            rev,
            seen,
            line,
            act,
            by,
            written,
            cold,
            of,
        })
    }
}

/// The evidence of a ledger, then what the model itself declares. A
/// block on a piece the model no longer has is set aside, not refused.
pub fn evidence(src: &str, m: &Model, names: &Names) -> Result<(Vec<Evidence>, Vec<Aside>), Error> {
    let mut aside = Vec::new();
    let mut out = blocks(src, m, names, Some(&mut aside))?;
    let lines = src.lines().count().max(out.last().map_or(0, |e| e.line));
    for (i, d) in names.declared.iter().enumerate() {
        out.push(Evidence {
            line: lines + i + 1,
            ..d.clone()
        });
    }
    Ok((out, aside))
}

/// The evidence blocks of a text, each on a piece the model has.
pub fn evidence_alone(src: &str, m: &Model, names: &Names) -> Result<Vec<Evidence>, Error> {
    blocks(src, m, names, None)
}

/// The words after `source`, `rev` and `seen` in each evidence block of
/// a text, with its first and last line and the piece it names as
/// written: read without a model.
// mitra: implements ledger::entries
pub fn entries(src: &str) -> Vec<Entry> {
    let toks = lex(src).unwrap_or_default();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < toks.len() {
        let open = toks[i].text == "evidence" && toks.get(i + 2).is_some_and(|t| t.text == "{");
        if !open {
            i += 1;
            continue;
        }
        let mut e = Entry {
            first: toks[i].line,
            source: Some(toks[i + 1].text.clone()),
            ..Entry::default()
        };
        let (mut depth, mut j) = (0, i + 2);
        while j < toks.len() {
            let word = |w: &str| toks[j].kind == Kind::Name && toks[j].text == w && depth == 1;
            let said = toks.get(j + 1).filter(|t| t.kind == Kind::Str);
            let said = said.map(|t| unquote(&t.text));
            match toks[j].text.as_str() {
                "{" => depth += 1,
                "}" => depth -= 1,
                _ if word("source") && said.is_some() => e.source = said,
                _ if word("rev") && said.is_some() => e.rev = said,
                _ if word("seen") && said.is_some() => e.seen = said,
                _ if word("for") && e.about.is_empty() => {
                    // `a::b::c`: names with `::` between them.
                    let mut k = j + 1;
                    while let Some(t) = toks.get(k).filter(|t| t.kind == Kind::Name) {
                        e.about += &t.text;
                        if toks.get(k + 1).is_none_or(|t| t.text != "::") {
                            break;
                        }
                        e.about += "::";
                        k += 2;
                    }
                }
                _ => {}
            }
            if depth == 0 {
                break;
            }
            j += 1;
        }
        e.last = toks.get(j).map_or(e.first, |t| t.line);
        out.push(e);
        i = j + 1;
    }
    out
}

/// The evidence blocks of a file. With `aside`, a block on a piece the
/// model does not have is put there; without, it is an error.
pub(crate) fn blocks(
    src: &str,
    m: &Model,
    names: &Names,
    mut aside: Option<&mut Vec<Aside>>,
) -> Result<Vec<Evidence>, Error> {
    let mut all: BTreeSet<String> = names.fns.keys().cloned().collect();
    all.extend(names.claims.keys().cloned());
    // The tie of each function to its code: `for f::tied`.
    let mut names = names.clone();
    for f in names.opaque.clone() {
        names
            .claims
            .insert(format!("{f}::tied"), (Vec::new(), Vec::new()));
        all.insert(format!("{f}::tied"));
    }
    // What a function with no `uses` calls outside its block: `for f::uses`.
    for f in names.opaque.clone() {
        let uses = format!("{f}::uses");
        if !names.claims.contains_key(&uses) {
            names.claims.insert(uses.clone(), (Vec::new(), Vec::new()));
            all.insert(uses);
        }
    }
    // And of each record to its struct: `for Record::tied`.
    for r in m.records.iter().filter(|r| r.of.is_none()) {
        let tied = format!("{}::tied", r.name);
        names.claims.insert(tied.clone(), (Vec::new(), Vec::new()));
        all.insert(tied);
    }
    // And of each enum to the enum of the code: `for Enum::tied`.
    for e in &m.enums {
        let tied = format!("{}::tied", e.name);
        names.claims.insert(tied.clone(), (Vec::new(), Vec::new()));
        all.insert(tied);
    }
    let mut p = P {
        toks: scope_types(lex(src)?, &names.scoped, &names.top, true),
        all,
        known: names.clone(),
        names,
        records: m.records.clone(),
        enums: m.enums.clone(),
        functions: m.functions.clone(),
        ..P::default()
    };
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    while p.pos < p.toks.len() {
        let block = p.is("evidence") && p.peek_at(2) == "{" && p.kind_at(1) == Some(Kind::Name);
        if block {
            let (start, line) = (p.pos, p.line());
            match (p.evidence(), aside.as_deref_mut()) {
                (Ok(e), _) => out.push(e),
                (Err(e), Some(aside)) if e.message.starts_with(NO_PIECE) => {
                    // To the end of the block.
                    let (mut depth, mut end) = (0, start + 2);
                    while let Some(t) = p.toks.get(end) {
                        depth += (t.text == "{") as i32 - (t.text == "}") as i32;
                        end += 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    p.pos = end;
                    let known = entries(src).into_iter().find(|x| x.first == line);
                    let known = known.unwrap_or_default();
                    aside.push(Aside {
                        about: known.about,
                        source: known.source,
                        rev: known.rev,
                        why: Why::Orphaned,
                        line,
                    });
                }
                (Err(e), _) => return Err(e),
            }
            continue;
        }
        let line = p.line();
        let hint = earlier(lines[line - 1]).map_or(
            "evidence is written `evidence name { for f given a = 1 holds }`",
            |h| h.1,
        );
        let msg = format!("expected an `evidence` block: {hint}");
        return Err(Error::new(line, msg));
    }
    Ok(out)
}

/// Turn the calls of a record's name into records.
/// The names an expression reads that it does not bind itself, in `used`
/// and returned.
pub(crate) fn free(e: &Expr, scope: &mut Vec<String>, used: &mut Vec<String>) -> Vec<String> {
    match e {
        Expr::Lit(_) => {}
        Expr::Var(x) => {
            if !scope.contains(x) && !used.contains(x) {
                used.push(x.clone());
            }
        }
        Expr::Let(x, a, b) => {
            free(a, scope, used);
            scope.push(x.clone());
            free(b, scope, used);
            scope.pop();
        }
        Expr::Match(a, arms) => {
            free(a, scope, used);
            for arm in arms {
                let names = arm.variant.iter().flat_map(|v| v.1.clone());
                let n = scope.len();
                scope.extend(names);
                free(&arm.body, scope, used);
                scope.truncate(n);
            }
        }
        Expr::Score(..) => {}
        Expr::Rate(x, _, a) => {
            scope.push(x.clone());
            free(a, scope, used);
            scope.pop();
        }
        Expr::Each(_, x, list, a) => {
            free(list, scope, used);
            scope.push(x.clone());
            free(a, scope, used);
            scope.pop();
        }
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) => {
            free(a, scope, used);
        }
        Expr::Call(_, args)
        | Expr::Tuple(args)
        | Expr::Correct(_, args)
        | Expr::Record(_, args)
        | Expr::List(args) => {
            args.iter().for_each(|a| {
                free(a, scope, used);
            });
        }
        Expr::Binary(_, a, b) | Expr::Index(a, b) => {
            free(a, scope, used);
            free(b, scope, used);
        }
        Expr::If(a, b, c) => {
            free(a, scope, used);
            free(b, scope, used);
            free(c, scope, used);
        }
    }
    used.clone()
}

/// Why a value of a record with unstated parts is refused.
pub(crate) fn unwritten(name: &str) -> String {
    format!(
        "`{name}` has parts the model does not state (`record {name};` or `..`): \
         no value of it can be written"
    )
}

pub(crate) fn records_in(e: &mut Expr, names: &[String]) {
    match e {
        Expr::Lit(_) | Expr::Var(_) => {}
        Expr::Call(f, args) => {
            args.iter_mut().for_each(|a| records_in(a, names));
            if names.contains(f) {
                *e = Expr::Record(std::mem::take(f), std::mem::take(args));
            }
        }
        Expr::Tuple(args) | Expr::Correct(_, args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter_mut().for_each(|a| records_in(a, names))
        }
        Expr::Match(a, arms) => {
            records_in(a, names);
            arms.iter_mut()
                .for_each(|arm| records_in(&mut arm.body, names));
        }
        Expr::Score(..) => {}
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) | Expr::Rate(_, _, a) => {
            records_in(a, names)
        }
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) | Expr::Each(_, _, a, b) => {
            records_in(a, names);
            records_in(b, names);
        }
        Expr::If(a, b, c) => {
            records_in(a, names);
            records_in(b, names);
            records_in(c, names);
        }
    }
}

/// A declaration as written, from its first line to the end of its
/// braces, on one line.
pub fn written(lines: &[String], line: usize) -> Option<String> {
    let mut depth = 0i32;
    let mut out = String::new();
    for l in lines.get(line.checked_sub(1)?..)? {
        let l = l.split("//").next().unwrap_or("").trim();
        depth += l.matches('{').count() as i32 - l.matches('}').count() as i32;
        if !out.is_empty() && !l.is_empty() {
            out.push(' ');
        }
        out += l;
        if depth <= 0 && !out.is_empty() {
            break;
        }
    }
    let short: String = out.chars().take(200).collect();
    Some(match short.len() < out.len() {
        true => format!("{short} …"),
        false => out,
    })
}

/// Evidence blocks with `source "…"` and `rev "…"` added to each block
/// that does not say it. A block that already has the clause is an error.
pub fn stamp(
    text: &str,
    source: Option<&str>,
    rev: Option<&str>,
    seen: &[Option<String>],
) -> Result<String, String> {
    let mut block = 0;
    let mut out = String::new();
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    // The block being read, without its strings: where a clause shows.
    let mut bare = String::new();
    for c in text.chars() {
        if quoted {
            (quoted, escaped) = (escaped || c != '"', !escaped && c == '\\');
            out.push(c);
            continue;
        }
        match c {
            '"' => {
                quoted = true;
                bare.push('"');
            }
            '{' => depth += 1,
            '}' if depth == 1 => {
                depth = 0;
                let has = |k: &str| {
                    let words: Vec<&str> = bare.split_whitespace().collect();
                    words.windows(2).any(|w| w[0] == k && w[1].starts_with('"'))
                };
                for (k, v) in [("source", source), ("rev", rev)] {
                    let Some(v) = v else { continue };
                    if has(k) {
                        return Err(format!(
                            "--{k} is given twice: in the option and in the evidence"
                        ));
                    }
                    if !out.ends_with(' ') {
                        out.push(' ');
                    }
                    out += &format!("{k} \"{v}\" ");
                }
                // The fingerprint of its piece, block by block.
                if let Some(Some(v)) = seen.get(block) {
                    if !out.ends_with(' ') {
                        out.push(' ');
                    }
                    out += &format!("seen \"{v}\" ");
                }
                block += 1;
                bare.clear();
            }
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth > 0 && c != '"' {
            bare.push(c);
        }
        out.push(c);
    }
    Ok(out)
}

/// The functions whose tie to the code the evidence speaks of.
pub fn tied(evidence: &[Evidence], names: &Names) -> Vec<String> {
    let found = |f: &&String| evidence.iter().any(|e| e.about == format!("{f}::tied"));
    names.opaque.iter().filter(found).cloned().collect()
}
