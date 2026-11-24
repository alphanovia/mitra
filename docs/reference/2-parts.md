# 2. Parts

A model is made of **parts**. A part is a piece of the system with a
name, what it needs, and what it promises.

## Subject and environment

- A **subject** is yours: you build it, you can change it.
- An **environment** is the world around it: you rely on it, you cannot
  change it.

From [examples/fall/fall.mitra](../../examples/fall/fall.mitra), a stone dropped from
a tower, in the light of the sun:

```mitra
environment sun {
    fn light() -> w: rational;
    ensure shines: "at least 200 W/m² at noon";
}
```

```mitra
subject stone {
    require light: sun::shines;
    require air: "still air";
```

## Require and ensure

- `require name: …;` is what a part **needs**. Either another part's
  ensure (`sun::shines`), or a hypothesis in words (`"still air"`).
- `ensure name: "…" from (…);` is what a part **promises**, and what
  the promise rests on: its claims, its requires.

```mitra
    ensure lands_slow: "lands slowly" from (speed::slow, drop::falls);
```

An ensure with no `from` (like `sun::shines`) rests on nothing in the
model: only evidence can hold it. A require in words is the same: Mitra
reports what each verdict rests on, so a hypothesis is never hidden.

Inside a part, a short path starts from the part: in `stone`,
`drop::falls` is `stone::drop::falls`. Anything else is written whole:
`sun::shines`.

## Parts inside parts

A larger system is parts made of parts. From
[examples/language/parts.mitra](../../examples/language/parts.mitra):

```mitra
subject Home {
    require power: "the house has power";
    ensure comfortable: "the heater keeps the room near its target"
        from (Heater::follows, Sensor::accurate);

    environment Sensor {
        fn read() -> t: integer;
        ensure accurate: "reads the room within 1 degree";
    }
```

An outer part holds only parts, requires, ensures and end-to-end
examples (M028); its own work is done by its inner parts. Its ensures
rest on its inner parts' ensures, or its own requires (M029).

A part's name is one in the whole model: `Home::Heater` is also just
`Heater`.

## A part is a type

A part may hold one nameless `record { … }`: its data. The part is then
a type, named like the part.

```mitra
    subject Heater {
        record { target: integer, on: bool }

        fn decide(self, t: integer) -> h: Heater {
            Heater { on: t < self.target, ..self }
        } where {
            claim keeps_target: h.target == self.target;
            examples { (Heater { target: 20, on: false }, 18) -> Heater { target: 20, on: true } }
        }

        fn off(target: integer) -> h: Heater { Heater { target: target, on: false } }
```

- A function whose first parameter is a bare `self` is a **method**,
  called `h.decide(18)`.
- Any other function is **static**, called `Heater::off(20)`.
- `Heater { on: …, ..self }` is a new `Heater` with `on` changed and the
  rest from `self`, as in Rust.

## Data parts, written short

A part that only holds data is written in one line:

```mitra
subject record Point { x: rational, y: rational }
```

```mitra
subject enum Shape { Dot(Point), Square(rational) }
```

`subject record Point { … }` is the same as `subject Point { record { … } }`.

Every named type is a data part: a part never declares a named record
or enum inside it. A data part is read anywhere, with no `use`. In a
tie or a store, its path is `Point::Point`: the part, then its type.

## Reopening a part

A part can be reopened, here or in another file, to add to it: same
kind, at most one record in all, no name given twice.

```mitra
subject Home::Heater {
    fn toggled(self) -> h: Heater { Heater { on: !self.on, ..self } } where {
        examples { (Heater { target: 20, on: true }) -> Heater { target: 20, on: false } }
    }
}
```

A file is not a part: a part may span files, and a file may hold many
parts.

## Kinds of names

Part names are `CamelCase` in new models (`Heater`); older models such
as `fall.mitra` use lowercase, which still reads. Built-in types are
lowercase (`integer`, `bool`).
