#!/bin/sh
# Run mitra-ds's tests and record the results as evidence in its store.
# Usage: scripts/dogfood-ds.sh   (from the repository root)
#
# The revision is the git hash of the crate's code (src/, tests/,
# Cargo.toml), not of the commit: committing new evidence does not make it
# stale, and only a change to the code does. The code must be committed,
# so the hash names what was tested.
set -eu
code="crates/mitra-ds/src crates/mitra-ds/tests crates/mitra-ds/Cargo.toml"
if ! git diff --quiet HEAD -- $code; then
    echo "dogfood-ds: commit the changes to mitra-ds first: the evidence names the committed code" >&2
    exit 1
fi
rev=$(git ls-tree HEAD $code | git hash-object --stdin | cut -c1-12)
report=$(mktemp)
trap 'rm -f "$report"' EXIT
cargo build -q -p mitra-cli
if ! cargo test -p mitra-ds --no-run -q 2>/dev/null; then
    echo "dogfood-ds: mitra-ds's tests do not build" >&2
    exit 1
fi
status=0
cargo test -p mitra-ds -- --test-threads=1 > "$report" 2>&1 || status=1
if ! grep -q '^test .* \.\.\. ' "$report"; then
    echo "dogfood-ds: no test results in the report" >&2
    exit 1
fi
cd crates/mitra-ds
../../target/debug/mitra ingest "$report" --rev "$rev"
# Only the evidence from this code: an earlier pass does not hide a failure now.
../../target/debug/mitra eval --rev "$rev"
exit $status
