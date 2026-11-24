#!/usr/bin/env bash
# The log is tickets.log in the current folder.
# Grades a tickets tool as a user would: arguments in, standard output and
# exit code out, the log file in a fresh folder. Usage: ./acceptance.sh FOLDER
set -u
dir="$(cd "${1:?usage: acceptance.sh FOLDER}" && pwd)"
(cd "$dir" && cargo build --release -q) || { echo "build failed"; exit 2; }
bin="$dir/target/release/tickets"
[ -x "$bin" ] || { echo "no target/release/tickets in $dir"; exit 2; }

pass=0; fail=0
run() { out="$("$bin" "$@" 2>/dev/null)"; code=$?; }
ok() { # ok EXPECTED ARGS...
    local want="$1"; shift; run "$@"
    if [ "$code" = 0 ] && [ "$out" = "$want" ]; then pass=$((pass+1))
    else fail=$((fail+1)); printf 'FAIL  [%s] tickets %s\n      want "%s", got "%s" (exit %s)\n' "$case" "$*" "$want" "$out" "$code"; fi
}
no() { # no ARGS...: a failure prints nothing and exits non-zero
    run "$@"
    if [ "$code" != 0 ] && [ -z "$out" ]; then pass=$((pass+1))
    else fail=$((fail+1)); printf 'FAIL  [%s] tickets %s\n      want a failure, got "%s" (exit %s)\n' "$case" "$*" "$out" "$code"; fi
}
fresh() { case="$1"; work="$(mktemp -d)"; cd "$work"; }

fresh "basics"
ok ""                    list
ok "#1"                  add "fix login"
ok "#2"                  add "write docs"
ok "#1 [ ] fix login"    show 1
ok ""                    done 1
ok "#1 [x] fix login"    show 1
ok ""                    done 1
ok "#1 [x] fix login
#2 [ ] write docs"       list

fresh "failures"
no
no frobnicate
no add
no add ""
no done 1
no show 1
no done x
no show -1
ok "#1"                  add "a"
no done 2
no done 1 2
ok "#1 [ ] a"            list

fresh "titles"
ok "#1"                  add "#2 [x] fake"
ok "#2"                  add "a|b \"c\" 3"
ok "#1 [ ] #2 [x] fake
#2 [ ] a|b \"c\" 3"      list

fresh "ids survive restarts"
for i in 1 2 3 4 5 6 7 8 9 10 11; do "$bin" add "t$i" >/dev/null 2>&1; done
ok "#12"                 add "t12"
ok "#12 [ ] t12"         show 12

# A crash while writing the last line: cut its end, byte by byte. The cut
# line must be ignored, never read as some other ticket, and the next id
# must not reuse a ticket that was acknowledged.
fresh "cut last line"
"$bin" add "first" >/dev/null 2>&1; "$bin" done 1 >/dev/null 2>&1
log="$work/tickets.log"
before=$(wc -c < "$log")
"$bin" add "second ticket" >/dev/null 2>&1
full=$(wc -c < "$log"); bad=""
for ((k = before; k < full; k++)); do
    d="$(mktemp -d)"; head -c "$k" "$log" > "$d/tickets.log"; cd "$d"
    # Cut only before the final line break, the line is whole: it may be
    # read as the change it is, though nobody acknowledged it.
    run list; { [ "$code" = 0 ] && { [ "$out" = "#1 [x] first" ] ||
        { [ "$k" = $((full - 1)) ] && [ "$out" = "#1 [x] first
#2 [ ] second ticket" ]; }; }; } || bad="$bad $k"
    run add "third"; [ "$code" = 0 ] && [ "$out" != "#1" ] || bad="$bad $k(add)"
done
if [ -z "$bad" ] && [ "$full" -gt "$before" ]; then pass=$((pass+1))
else fail=$((fail+1)); echo "FAIL  [cut last line] wrong at cuts:$bad"; fi

# The disk refuses the write: nothing may be acknowledged.
fresh "disk fails"
ok "#1"                  add "first"
mv tickets.log kept; mkdir tickets.log
no add "second"
rmdir tickets.log; mv kept tickets.log
ok "#1 [ ] first"        list

# Two runs at once: 20 adds in parallel must give 20 different ids.
fresh "parallel adds"
for i in $(seq 1 20); do "$bin" add "p$i" > "out.$i" 2>/dev/null & done; wait
ids="$(cat out.* | sort -u | wc -l)"; rows="$("$bin" list 2>/dev/null | wc -l)"
if [ "$ids" = 20 ] && [ "$rows" = 20 ]; then pass=$((pass+1))
else fail=$((fail+1)); echo "FAIL  [parallel adds] $ids different ids printed, $rows tickets listed, of 20"; fi

echo "acceptance: $pass passed, $fail failed, of $((pass+fail))"
