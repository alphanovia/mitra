#!/usr/bin/env python3
"""The deliberate-breaks benchmark: break the code one way at a time, run
the crate's tests, record them as evidence, vouch again blindly for every
`uniform` and `uses` claim, and print what `cargo test` and Mitra say.

Usage: scripts/breaks.py <crate> [-v]   (from the repository root)
Needs a clean working tree: `git checkout` undoes a run that was killed.
The breaks of a crate are in scripts/breaks/<crate>.txt: blocks separated
by a line `==`, each `name`, `file`, `--`, old text, `--`, new text.
Nothing is left changed: files and evidence are restored after each."""
import os, re, shutil, signal, subprocess, sys, tempfile

crate = sys.argv[1]
verbose = "-v" in sys.argv
root = f"crates/{crate}"
# A copy: the tests of mitra-cli rebuild the binary, with the break in it.
work = tempfile.mkdtemp()
mitra = f"{work}/mitra"
evidence = f"{root}/.mitra/evidence.mitra"


def run(cmd, cwd=None, limit=None):
    # MITRA_BREAKS: the tests that compare a store with its expected
    # verdicts skip themselves, since the evidence is replaced here.
    env = dict(os.environ, MITRA_BREAKS="1")
    out = tempfile.TemporaryFile(mode="w+")
    # Its own process group: a test that hangs is killed with cargo.
    p = subprocess.Popen(cmd, cwd=cwd, stdout=out, stderr=out, env=env, start_new_session=True)
    try:
        code = p.wait(timeout=limit)
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGKILL)
        p.wait()
        return None, ""
    out.seek(0)
    return code, out.read()


def assumptions():
    names = []
    for d, _, files in os.walk(root):
        for f in sorted(files):
            if f.endswith((".rs", ".mitra")) and "target" not in d:
                lines = open(os.path.join(d, f)).read().split("\n")
                lines = lines if f.endswith(".mitra") else []
                # `claim c = uniform(…)`, named `subject::c` inside
                # `subject { … }`.
                scope = ""
                for l in lines:
                    m = re.match(r"(?:subject|boundary) (\w+)", l)
                    scope = m.group(1) + "::" if m else "" if l.startswith("}") else scope
                    names += [scope + c for c in re.findall(r"claim (\w+) = uniform\(", l)]
    return sorted(set(names))


def judge(name):
    code, report = run(["cargo", "test", "-p", crate, "--", "--test-threads=1"], limit=300)
    if code is None:
        return f"{name} | tests hang | no report, so no evidence"
    if "test result" not in report:
        return f"{name}: does not build"
    open(f"{work}/report.txt", "w").write(report)
    run([mitra, "ingest", f"{work}/report.txt", "--rev", "break"], cwd=root)
    vouches = "\n".join(f"evidence review {{ for {a} holds }}" for a in assumptions())
    if vouches:
        run([mitra, "add", vouches, "--source", "review:blind", "--rev", "break"], cwd=root)
    _, out = run([mitra, "eval", "--rev", "break"], cwd=root)
    if verbose:
        print(out)
    failing = re.findall(r"^([\w:]+): fails", out, re.M)
    certain = sorted(set(re.findall(r"^    certain +the code of `([\w:]+)`", out, re.M)))
    first = sorted(set(re.findall(r"help: look first at `([\w:]+)`", out)))
    _, todo = run([mitra, "next", "--rev", "break"], cwd=root)
    causes = sorted(set(re.findall(r"^root cause: [^`]*`([\w:]+)`", todo, re.M)))
    tests = "fail" if code else "pass"
    line = f"{name} | tests {tests} | fails: {', '.join(failing) or 'none'}"
    line += f" | code named certain: {', '.join(certain) or 'none'}"
    if first:
        line += f" | look first at: {', '.join(first)}"
    if causes:
        line += f" | root cause: {', '.join(causes)}"
    return line


run(["cargo", "build", "-q", "-p", "mitra-cli"])
shutil.copy("target/debug/mitra", mitra)
kept = open(evidence).read()
blocks = open(f"scripts/breaks/{crate}.txt").read().split("\n==\n")
try:
    for b in blocks:
        head, old, new = (b.strip("\n") + "\n").split("\n--\n")
        new = new.rstrip("\n")
        name, path = head.split("\n")
        source = open(path).read()
        assert source.count(old) == 1, f"{name}: text found {source.count(old)} times"
        open(evidence, "w").write("")
        open(path, "w").write(source.replace(old, new))
        try:
            print(judge(name), flush=True)
        finally:
            open(path, "w").write(source)
            open(evidence, "w").write(kept)
finally:
    open(evidence, "w").write(kept)
