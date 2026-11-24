#!/usr/bin/env bash
# Grades a semver tool as a user would: arguments in, standard output and
# exit code out. Usage: ./acceptance.sh FOLDER (a cargo project).
set -u
dir="${1:?usage: acceptance.sh FOLDER}"
(cd "$dir" && cargo build --release -q) || { echo "build failed"; exit 2; }
bin="$(ls "$dir"/target/release/semver 2>/dev/null || true)"
[ -x "$bin" ] || { echo "no target/release/semver in $dir"; exit 2; }

pass=0; fail=0
ok() { # ok EXPECTED ARGS...: prints EXPECTED, exit 0
    local want="$1"; shift
    local got code; got="$("$bin" "$@" 2>/dev/null)"; code=$?
    if [ "$code" = 0 ] && [ "$got" = "$want" ]; then pass=$((pass+1))
    else fail=$((fail+1)); printf 'FAIL  semver %s\n      want "%s", got "%s" (exit %s)\n' "$*" "$want" "$got" "$code"; fi
}
no() { # no ARGS...: prints nothing on stdout, exit not 0
    local got code; got="$("$bin" "$@" 2>/dev/null)"; code=$?
    if [ "$code" != 0 ] && [ -z "$got" ]; then pass=$((pass+1))
    else fail=$((fail+1)); printf 'FAIL  semver %s\n      want a failure, got "%s" (exit %s)\n' "$*" "$got" "$code"; fi
}

# Bumps.
ok 1.5.0     bump minor 1.4.2
ok 2.0.0     bump major 1.4.2
ok 1.4.3     bump patch 1.4.2
ok 1.4.10    bump patch 1.4.9
ok 1.0.0     bump major 0.0.0
ok 10.20.31  bump patch 10.20.30
# A pre-release is dropped.
ok 2.0.0     bump major 1.4.2-rc.1
ok 1.3.0     bump minor 1.2.3-beta
ok 1.2.4     bump patch 1.2.3-rc.1
# The command line.
no
no bump
no bump minor
no grow minor 1.4.2
no bump minor 1.4.2 extra
no bump huge 1.4.2
no bump Minor 1.4.2
# The version.
no bump minor ""
no bump minor 1.4
no bump minor 1.4.2.0
no bump minor 01.4.2
no bump minor 1.04.2
no bump minor 1.x.2
no bump minor -1.4.2
no bump minor +1.4.2
no bump minor v1.4.2
no bump minor " 1.4.2"
no bump minor 1..2
no bump minor 1.4.2-
no bump minor 1.4.2+build

echo "acceptance: $pass passed, $fail failed, of $((pass+fail))"
