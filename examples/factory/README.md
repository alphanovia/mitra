# Factory (draft 2)

A software factory fed by a Mitra model: a coding harness (Claude Code,
opencode, or a fake so tests need no inference) writes the code, the tests switched on
run, and Mitra judges. In: a git revision, the model's root and the build's
root. Out: a branch of commits from that revision, with a report. A first
shot, a change request and a repeat shot are the same run.

Model only for now: start at `src/factory.mitra`. Choices still open are
marked CHOICE TO CONFIRM (Fn) and QUESTION.

    mitra check examples/factory/src/factory.mitra
