# Factory (draft 1)

A software factory fed by a Mitra model: a coding harness (Claude Code,
opencode, or a fake for the POC) writes the code, the tests switched on
run, and Mitra judges. Two paths: `build` (a model alone) and `change` (a
model change and the existing code).

Model only for now: start at `src/factory.mitra`. Choices still open are
marked CHOICE TO CONFIRM (F1, ...) and QUESTION (Q1, ...).

    mitra check examples/factory/src/factory.mitra
