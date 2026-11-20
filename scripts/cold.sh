#!/bin/sh
# Record the words of cold agents on a crate's store, from
# scripts/cold/<crate>.txt, at the revision of its committed code.
# Usage: scripts/cold.sh <crate>   (from the repository root)
set -eu
crate=$1
list="$PWD/scripts/cold/$crate.txt"
[ -e "$list" ] || exit 0
code=""
for p in src tests Cargo.toml; do
    [ -e "crates/$crate/$p" ] && code="$code crates/$crate/$p"
done
rev=$(git ls-tree HEAD $code | git hash-object --stdin | cut -c1-12)
cd "crates/$crate"
# One `mitra add` for all of them: the store is evaluated once.
text=$(awk '
    $1 == "review" { printf "evidence cold { for %s holds act review by \"%s\" cold }\n", $2, $3 }
    $1 == "tie" { printf "evidence cold { for %s holds act review by \"%s\" of \"test:%s\" cold }\n", $3, $4, $2 }
' "$list")
../../target/debug/mitra add "$text" --source "review:cold" --rev "$rev" | sed 's/^added \([0-9]*\).*/cold words recorded: \1/'
