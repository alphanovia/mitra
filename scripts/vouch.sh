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
{ grep -rhE 'mitra:' src tests 2>/dev/null; cat .mitra/model.mitra; } | grep -oE 'claim \w+ : (only )?(uniform|uses) ' | cut -d' ' -f2 | sort -u |
while read -r claim; do
    ../../target/debug/mitra add "evidence $claim at 1" --source "$source" --rev "$rev" >/dev/null
    echo "vouched: $claim"
done
