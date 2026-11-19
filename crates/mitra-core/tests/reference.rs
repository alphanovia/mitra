//! Every example of the keyword reference (`docs/reference/keywords.md`
//! and the `kw-*.md` files beside it) is read and evaluated.
//!
//! A block fenced `mitra` is a whole model. A block fenced `evidence`
//! is evidence for the model above it. A block fenced `text` holds
//! lines of the report Mitra gives for the two: each must be in it.

use std::{fs, path::Path};

#[test]
fn every_example_runs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference");
    let (mut models, mut checked) = (0, 0);
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.starts_with("kw-") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let (mut model, mut evidence) = (String::new(), String::new());
        let mut block: Option<(String, String, usize)> = None;
        for (i, line) in text.lines().enumerate() {
            let Some((kind, body, start)) = &mut block else {
                if let Some(kind) = line.strip_prefix("```") {
                    block = Some((kind.to_string(), String::new(), i + 1));
                }
                continue;
            };
            if line != "```" {
                *body += line;
                *body += "\n";
                continue;
            }
            let at = format!("{name}:{start}");
            match kind.as_str() {
                "mitra" => (model, evidence) = (body.clone(), String::new()),
                "evidence" => evidence = body.clone(),
                "text" => {}
                other => panic!("{at}: a block fenced `{other}`"),
            }
            let report = mitra_core::run(&model, &evidence)
                .unwrap_or_else(|(_, e)| panic!("{at}: {e}\n{model}\n{evidence}"))
                .to_text();
            models += (kind == "mitra") as usize;
            if kind == "text" {
                for want in body.lines().filter(|l| !l.trim().is_empty()) {
                    assert!(
                        report.contains(want.trim_end()),
                        "{at}: no `{want}` in\n{report}"
                    );
                    checked += 1;
                }
            }
            block = None;
        }
        assert!(block.is_none(), "{name}: a block is not closed");
    }
    assert!(
        models >= 30 && checked >= 30,
        "{models} models, {checked} lines"
    );
}

/// Every keyword of the index has its entry.
#[test]
fn the_index_points_at_entries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/reference");
    let index = fs::read_to_string(root.join("keywords.md")).unwrap();
    let mut seen = 0;
    for link in index.split("](").skip(1) {
        let target = link.split(')').next().unwrap();
        let Some((file, anchor)) = target.split_once('#') else {
            continue;
        };
        let text = fs::read_to_string(root.join(file)).unwrap();
        let has = text.lines().filter_map(|l| l.strip_prefix("## ")).any(|h| {
            let slug: String = h
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .map(|c| if c == ' ' { '-' } else { c })
                .collect();
            slug == anchor
        });
        assert!(has, "keywords.md: no `## …` for {target}");
        seen += 1;
    }
    assert!(seen >= 40, "{seen} entries");
}
