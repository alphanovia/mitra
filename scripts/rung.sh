#!/bin/sh
# Run a rung of the ladder through the four levels and record what it
# says: its tests as evidence, its ties, and the report in
# model/expected.txt. The reviews in the ledger are kept as they are.
# Usage: scripts/rung.sh ladder/1-clock   (from the repository root)
# The code must be committed: the revision is the git hash of the
# rung's src/ and Cargo.toml, as in scripts/dogfood.sh.
set -eu
dir=${1%/}
pkg=$(sed -n 's/^name = "\(.*\)"/\1/p' "$dir/Cargo.toml")
if ! git diff --quiet HEAD -- "$dir/src" "$dir/Cargo.toml"; then
    echo "rung: commit the changes to $dir first: the evidence names the committed code" >&2
    exit 1
fi
rev=$(git ls-tree HEAD "$dir/src" "$dir/Cargo.toml" | git hash-object --stdin | cut -c1-12)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cargo build -q -p mitra-cli
mitra=$(pwd)/target/debug/mitra
cargo test -p "$pkg" > "$work/report.txt" 2>&1 || true
(cd "$dir" && "$mitra" assess "$work/report.txt" --rev "$rev" > /dev/null) || true
# The report is the one a test reads back: `mitra assess` in a copy.
cp -r "$dir" "$work/rung"
(cd "$work/rung" && "$mitra" assess > "$work/expected.txt") || true
cp "$work/expected.txt" "$dir/model/expected.txt"
tail -n 12 "$dir/model/expected.txt"
