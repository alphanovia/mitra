//! What Mitra is handed: the files of a model and the modules they
//! import, each a named text. Nothing in this crate reads a disk; finding
//! and reading the files is the work of `mitra-store`.

use crate::parse::Handed;
use crate::Error;

/// One file, as text: its name is what errors say.
#[derive(Debug, Clone, Default, PartialEq)]
// mitra: implements modules::Module
pub struct Module {
    pub name: String,
    pub text: String,
}

/// The files of one model, in the order they are read, and the modules
/// they may import.
#[derive(Debug, Clone, Default)]
// mitra: implements modules::Modules
pub struct Modules {
    files: Vec<Module>,
    pub handed: Handed,
    /// The files one after the other: a model is read as one text.
    text: String,
    /// For each line of `text`: its file and its line there.
    lines: Vec<(usize, usize)>,
}

impl Modules {
    /// A model in one text, importing nothing but `std`.
    // mitra: implements modules::Modules::alone
    pub fn alone(text: &str) -> Modules {
        let (name, text) = ("model".to_string(), text.to_string());
        Modules::new(vec![Module { name, text }], Handed::default())
    }

    /// The files of a model. They are read in the order given, except
    /// that a file comes after the files whose types it names.
    // mitra: implements modules::Modules::new
    pub fn new(files: Vec<Module>, handed: Handed) -> Modules {
        let files = in_order(files);
        let (mut text, mut lines) = (String::new(), Vec::new());
        for (i, f) in files.iter().enumerate() {
            for (n, l) in f.text.lines().enumerate() {
                text += l;
                text.push('\n');
                lines.push((i, n + 1));
            }
        }
        // One text handed alone is read as it is, to the byte.
        if let [only] = &files[..] {
            text = only.text.clone();
        }
        Modules {
            files,
            handed,
            text,
            lines,
        }
    }

    /// The files, in the order they are read.
    pub fn files(&self) -> &[Module] {
        &self.files
    }

    /// The whole model, as it is read.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// A line of the model, as `file:line`.
    // mitra: implements modules::Modules::place
    pub fn place(&self, line: usize) -> String {
        match self.lines.get(line.wrapping_sub(1)) {
            Some((file, n)) => format!("{}:{n}", self.files[*file].name),
            None => self.first(),
        }
    }

    /// An error of the model, at its own file and line. An error past
    /// the last line names the first file.
    pub fn locate(&self, err: &Error) -> String {
        match self.lines.get(err.line.wrapping_sub(1)) {
            Some((file, n)) => format!("{}: line {n}: {}", self.files[*file].name, err.message),
            None => format!("{}: {}", self.first(), err.message),
        }
    }

    fn first(&self) -> String {
        self.files.first().map_or(String::new(), |f| f.name.clone())
    }
}

/// The files of a model, each after the files whose subjects it names a
/// type of: a type is declared above where it is used, whatever the file
/// names. Files that name each other keep the order given.
fn in_order(files: Vec<Module>) -> Vec<Module> {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let mut read = Vec::new();
    for f in &files {
        // Comments name other files freely.
        let code: Vec<&str> = f
            .text
            .lines()
            .map(|l| l.split("//").next().unwrap_or(""))
            .collect();
        let code = code.join("\n");
        // The blocks a file declares, and its records and enums.
        let (mut blocks, mut types): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
        let words: Vec<&str> = code.split_whitespace().collect();
        for pair in words.windows(2) {
            let name = pair[1].trim_end_matches([';', '{']).to_string();
            match pair[0] {
                "subject" | "boundary" => blocks.push(name),
                "record" | "enum" => types.push(name),
                _ => {}
            }
        }
        read.push((code, blocks, types));
    }
    let names = |i: usize, j: usize| {
        // `impl R { … }` adds to a record declared in another file.
        let adds = read[i].0.lines().any(|l| {
            let mut w = l.split_whitespace();
            matches!((w.next(), w.next()), (Some("impl"), Some(r)) if read[j].2.iter().any(|t| t == r))
        });
        adds || read[j].1.iter().any(|b| {
            let path = format!("{b}::");
            // `reader::Model` is a type of that file, to be declared
            // above; `reader::model(…)` is a call and `verdicts::Holds` a
            // variant, read wherever they are declared.
            read[i].0.match_indices(&path).any(|(at, _)| {
                let after: String = read[i].0[at + path.len()..]
                    .chars()
                    .take_while(|c| word(*c))
                    .collect();
                let before = read[i].0[..at].chars().next_back();
                !before.is_some_and(word) && read[j].2.contains(&after)
            })
        })
    };
    let mut left: Vec<usize> = (0..files.len()).collect();
    let mut order = Vec::new();
    while !left.is_empty() {
        let free = |i: &usize| !left.iter().any(|j| j != i && names(*i, *j));
        let next = left.iter().copied().find(|i| free(i)).unwrap_or(left[0]);
        left.retain(|i| *i != next);
        order.push(next);
    }
    let mut files: Vec<Option<Module>> = files.into_iter().map(Some).collect();
    order.into_iter().filter_map(|i| files[i].take()).collect()
}
