# Mitra for VS Code

Syntax highlighting for `.mitra` files, and for the tie comments
`// mitra: implements …` and `// mitra: evidence for …` in Rust files.
No language server: colours, comments and brackets only.

## Install, without publishing

From the repository root, link this folder into the extensions folder of
VS Code, then reload the window (`Developer: Reload Window`):

```sh
ln -s "$PWD/editors/vscode" ~/.vscode/extensions/mitra.mitra-0.1.0
```

On Windows (PowerShell), copy it instead:

```powershell
Copy-Item -Recurse editors\vscode $env:USERPROFILE\.vscode\extensions\mitra.mitra-0.1.0
```

For VS Code over SSH or in a container the folder is
`~/.vscode-server/extensions/`; for Cursor, `~/.cursor/extensions/`.

To hand it to someone as one file: `npx @vscode/vsce package` here
writes `mitra-0.1.0.vsix`, installed with
`code --install-extension mitra-0.1.0.vsix`.

## Keeping it in step with the language

`cargo test -p mitra-lang --test editors` checks that every word the
reader reserves, and every clause of an evidence block, is in the
grammar (`syntaxes/mitra.tmLanguage.json`). A new keyword fails that
test until the grammar has it.
