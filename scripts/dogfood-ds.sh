#!/bin/sh
# Run mitra-ds's tests and record the results as evidence in its store.
# Usage: scripts/dogfood-ds.sh   (from the repository root)
set -eu
report=$(mktemp)
cargo build -q -p mitra-cli
cargo test -p mitra-ds -- --test-threads=1 > "$report" 2>&1 || true
cd crates/mitra-ds
rev=$(git rev-parse --short HEAD)
../../target/debug/mitra ingest "$report" --rev "$rev"
# Only the evidence from this revision: an earlier pass does not hide a failure now.
../../target/debug/mitra eval --rev "$rev"
rm -f "$report"
