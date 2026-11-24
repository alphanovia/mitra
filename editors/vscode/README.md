# Mitra for VS Code

Colours `.mitra` files: keywords, part/record/function names, claim
names, types, strings, numbers (`1/3` too), `|-`, `->`, `::`, `[~T]`,
comments. Colours only: no checking, no completion.

## Install (local)

Link this folder into VS Code's extensions, then reload the window
(Command Palette: "Developer: Reload Window"):

```sh
# macOS / Linux, from the repository root
ln -s "$PWD/editors/vscode" ~/.vscode/extensions/mitra
```

On Windows, copy the folder to `%USERPROFILE%\.vscode\extensions\mitra`.
A link follows the repository; a copy must be made again after a change.

Or as a package: `npx @vscode/vsce package` in this folder, then
"Extensions: Install from VSIX…" with the `.vsix` it writes.

Open `examples/fall/fall.mitra` to see it.

## Keeping it in step

- `syntaxes/mitra.tmLanguage.json` lists the parser's keywords in its
  `"keyword-…"` entries. They must match `KEYWORDS` in `src/parse.rs`,
  changed in the same commit; `tests/editor_grammar.rs` fails when they
  drift.
- Not tested, so by hand: primitive type names (`"type"`), proof
  operations (`"proof-op"`, `PROOF_OPS` in `src/parse.rs`), symbols
  (`"operator"`, `SYMBOLS`).
- `"self-and-loops"` colours `self` and the loops `until(…)` and
  `.fold(…)`, which the parser reads as names, not keywords.
- `"proof-form"` colours settled forms the parser does not read yet
  (`by induction`, `by invariant`). When one becomes a parser keyword,
  move it to a `"keyword-…"` entry.
