#!/usr/bin/env python3
"""How much of a crate its model covers: every function of src/, tied to
the model (`// mitra: implements …` above it) or not, with the lines of
each, and the size of the model beside the size of the code.

    scripts/coverage.py                 # the three crates, a summary
    scripts/coverage.py --list          # and every function
    scripts/coverage.py --write         # writes docs/coverage.md

Tests (`#[cfg(test)]` to the end of the file) are left out. A line counts
when it is neither blank nor a comment.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CRATES = ["mitra-ds", "mitra-cli", "mitra-core"]
FN = re.compile(r"^\s*(?:pub(?:\([a-z]+\))? )?(?:const )?fn (\w+)")


def counted(line):
    text = line.strip()
    return bool(text) and not text.startswith("//")


def functions(path):
    """Each function of a file: name, tie, first line, lines of code."""
    lines = path.read_text().split("\n")
    end = next((i for i, l in enumerate(lines) if l.strip() == "#[cfg(test)]"), len(lines))
    found, i = [], 0
    while i < end:
        m = FN.match(lines[i])
        if not m:
            i += 1
            continue
        # The tie is in the comments and attributes right above.
        tie, j = None, i - 1
        while j >= 0 and lines[j].strip().startswith(("//", "#[")):
            t = re.match(r"\s*// mitra: implements (\S+)", lines[j])
            tie = t.group(1) if t else tie
            j -= 1
        # To the brace that closes the body.
        depth, k, opened = 0, i, False
        while k < end:
            code = re.sub(r'"(\\.|[^"\\])*"|\'(\\.|[^\'\\])\'|//.*', "", lines[k])
            depth += code.count("{") - code.count("}")
            opened = opened or "{" in code
            if (opened and depth <= 0) or (not opened and code.rstrip().endswith(";")):
                break
            k += 1
        size = sum(counted(l) for l in lines[i : k + 1])
        found.append((m.group(1), tie, i + 1, size))
        i += 1
    code = sum(counted(l) for l in lines[:end])
    return found, code


def crate(name):
    src = ROOT / "crates" / name / "src"
    rows, code = [], 0
    for path in sorted(src.rglob("*.rs")):
        found, n = functions(path)
        code += n
        rows += [(path.relative_to(src).as_posix(), *f) for f in found]
    store = ROOT / "crates" / name / ".mitra"
    model, words = 0, {"subject": 0, "ensure": 0, "require": 0}
    for path in sorted(store.glob("*.mitra")):
        if path.name != "evidence.mitra":
            lines = path.read_text().split("\n")
            model += sum(counted(l) for l in lines)
            for w in words:
                words[w] += sum(l.split()[:1] == [w] for l in lines)
    return rows, code, model, words


def plural(n, word):
    return f"{n} {word}" + ("" if n == 1 else "s")


def sized(name, code, model, words):
    """The model beside the code, in one line."""
    s, e, r = (words[w] for w in ("subject", "ensure", "require"))
    return (f"{name}: {model} lines of model over {code} lines of code, in "
            f"{plural(s, 'subject')} with {plural(e, 'ensure')} and {plural(r, 'require')}.")


def report(listing):
    out = ["# How much of Mitra its model covers", ""]
    out += ["Written by `scripts/coverage.py --write`. A function is covered when a"]
    out += ["`// mitra: implements …` comment ties it to the model. Tests are left out."]
    out += ["", "| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |"]
    out += ["|---|---|---|---|---|---|---|---|---|---|"]
    total = [0] * 6
    details, sizes = [], []
    for name in CRATES:
        rows, code, model, words = crate(name)
        sizes.append("- " + sized(name, code, model, words))
        tied = [r for r in rows if r[2]]
        lines, tied_lines = sum(r[4] for r in rows), sum(r[4] for r in tied)
        nums = [len(rows), len(tied), lines, tied_lines, code, model]
        total = [a + b for a, b in zip(total, nums)]
        out.append(row(name, nums))
        details += ["", f"## {name}", "", "| File | Function | Line | Lines | Tied to |", "|---|---|---|---|---|"]
        for f, fn, tie, line, size in sorted(rows, key=lambda r: (r[2] is None, r[0], r[3])):
            details.append(f"| {f} | `{fn}` | {line} | {size} | {tie or ''} |")
    out.append(row("all", total))
    out += [""] + sizes
    return "\n".join(out + (details if listing else [])) + "\n"


def row(name, n):
    share = lambda a, b: f"{100 * a / b:.0f}%" if b else "-"
    return (f"| {name} | {n[0]} | {n[1]} | {share(n[1], n[0])} | {n[2]} | {n[3]} | "
            f"{share(n[3], n[2])} | {n[4]} | {n[5]} | {share(n[5], n[4])} |")


if __name__ == "__main__":
    if "--write" in sys.argv:
        (ROOT / "docs" / "coverage.md").write_text(report(True))
    else:
        print(report("--list" in sys.argv), end="")
