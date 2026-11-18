#!/bin/sh
# Vouch for every `uniform` and `uses` claim of a crate's model, at the
# revision of its committed code, after reviewing them against the code.
# Usage: scripts/vouch.sh <crate> <source>   (from the repository root)
set -eu
crate=$1
source=$2
code=""
for p in src tests Cargo.toml; do
    [ -e "crates/$crate/$p" ] && code="$code crates/$crate/$p"
done
rev=$(git ls-tree HEAD $code | git hash-object --stdin | cut -c1-12)
cd "crates/$crate"
# Both syntaxes: `claim c : uniform f` and `claim c = uniform(f)`, the
# latter named `subject::c` when written inside `subject { … }`.
{ grep -rhE 'mitra:' src tests 2>/dev/null; cat .mitra/*.mitra; } | awk '
    /^(subject|boundary) / { scope = $2 "::" }
    /^}/ { scope = "" }
    match($0, /claim [A-Za-z0-9_]+ : (only )?(uniform|uses) /) { split(substr($0, RSTART), w, " "); print w[2] }
    match($0, /claim [A-Za-z0-9_]+ = uniform\(/) { split(substr($0, RSTART), w, " "); print scope w[2] }
' | sort -u |
while read -r claim; do
    ../../target/debug/mitra add "evidence $claim at 1" --source "$source" --rev "$rev" >/dev/null
    echo "vouched: $claim"
done
