# The fall, from a value to a verdict

Printed by `cargo run --example falling`. The model is `tests/falling.rs`.
Each measurement is evaluated, a timer takes what that gave, and the
calculus judges every piece. A dash means nothing was evaluated or taken.
A deduction is not listed: it is never judged.

### The stone, three measurements, nothing reviewed

Verdict: unknown

| piece | step | evaluated | taken | judged | interval |
|---|---|---|---|---|---|
| `stone::falls` | induction | - | - | unknown | 0.00 to 1.00 |
| `stone::only the height matters` | leaf | - | - | unknown | 0.00 to 1.00 |
| `stone::speed` | leaf | - | - | unknown | 0.00 to 1.00 |
| `stone::slow` | leaf | - | - | unknown | 0.00 to 1.00 |
| `stone::is the one dropped` | leaf | - | true | true 0.92 | 0.83 to 1.00 |
| `stone::speed @ h = 5/1, v = 99/10` | leaf | true | true | true 0.92 | 0.83 to 1.00 |
| `stone::speed @ h = 10/1, v = 14/1` | leaf | true | true | true 0.92 | 0.83 to 1.00 |
| `stone::speed @ h = 20/1, v = 99/5` | leaf | true | true | true 0.92 | 0.83 to 1.00 |

### The stone, reviewed

Verdict: true 0.86, least sure: `stone::falls`

| piece | step | evaluated | taken | judged | interval |
|---|---|---|---|---|---|
| `stone::falls` | induction | - | - | true 0.86 | 0.71 to 1.00 |
| `stone::only the height matters` | leaf | - | true | true 0.92 | 0.83 to 1.00 |
| `stone::speed` | leaf | - | - | true 0.86 | 0.71 to 1.00 |
| `stone::slow` | leaf | - | - | true 0.86 | 0.71 to 1.00 |
| `stone::is the one dropped` | leaf | - | true | true 0.92 | 0.83 to 1.00 |
| `stone::speed @ h = 5/1, v = 99/10` | leaf | true | true | true 0.92 | 0.83 to 1.00 |
| `stone::speed @ h = 10/1, v = 14/1` | leaf | true | true | true 0.92 | 0.83 to 1.00 |
| `stone::speed @ h = 20/1, v = 99/5` | leaf | true | true | true 0.92 | 0.83 to 1.00 |
| `stone::represents` | induction | - | true | true 0.92 | 0.83 to 1.00 |

### The stone, reviewed, then one bad measurement

Verdict: conflict

| piece | step | evaluated | taken | judged | interval |
|---|---|---|---|---|---|
| `stone::falls` | induction | - | - | conflict | 0.29 to 0.41 |
| `stone::only the height matters` | leaf | - | true | true 0.92 | 0.83 to 1.00 |
| `stone::speed` | leaf | - | - | conflict | 0.29 to 0.41 |
| `stone::slow` | leaf | - | - | conflict | 0.71 to 1.00 |
| `stone::is the one dropped` | leaf | - | true | conflict | 0.59 to 0.71 |
| `stone::speed @ h = 5/1, v = 99/10` | leaf | true | true | conflict | 0.59 to 0.71 |
| `stone::speed @ h = 10/1, v = 14/1` | leaf | true | true | conflict | 0.59 to 0.71 |
| `stone::speed @ h = 20/1, v = 99/5` | leaf | true | true | conflict | 0.59 to 0.71 |
| `stone::speed @ h = 30/1, v = 99/10` | leaf | false | false | conflict | 0.29 to 0.41 |
| `stone::represents` | induction | - | true | conflict | 0.45 to 0.55 |

Cannot all be true: `stone::speed @ h = 5/1, v = 99/10`

Cannot all be true: `stone::speed @ h = 10/1, v = 14/1`

Cannot all be true: `stone::speed @ h = 20/1, v = 99/5`

Cannot all be true: `stone::is the one dropped`

Cannot all be true: `stone::represents`

Cannot all be true: `stone::speed @ h = 30/1, v = 99/10`

### The feather, hypothesis not taken

Verdict: false

| piece | step | evaluated | taken | judged | interval |
|---|---|---|---|---|---|
| `feather::falls` | induction | - | - | unknown | 0.00 to 1.00 |
| `feather::only the height matters` | leaf | - | - | unknown | 0.00 to 1.00 |
| `feather::speed` | leaf | - | - | false 0.92 | 0.00 to 0.17 |
| `feather::slow` | leaf | - | - | unknown | 0.00 to 1.00 |
| `feather::is the one dropped` | leaf | - | true | true 0.92 | 0.83 to 1.00 |
| `feather::speed @ h = 5/1, v = 2/1` | leaf | false | false | false 0.92 | 0.00 to 0.17 |
| `feather::speed @ h = 10/1, v = 21/10` | leaf | false | false | false 0.92 | 0.00 to 0.17 |

Cannot all be true: `feather::falls`, `feather::only the height matters`

### The feather, hypothesis taken

Verdict: false

| piece | step | evaluated | taken | judged | interval |
|---|---|---|---|---|---|
| `feather::falls` | induction | - | - | false 0.92 | 0.00 to 0.17 |
| `feather::only the height matters` | leaf | - | true | true 0.92 | 0.83 to 1.00 |
| `feather::speed` | leaf | - | - | false 0.92 | 0.00 to 0.17 |
| `feather::slow` | leaf | - | - | unknown | 0.00 to 1.00 |
| `feather::is the one dropped` | leaf | - | true | true 0.92 | 0.83 to 1.00 |
| `feather::speed @ h = 5/1, v = 2/1` | leaf | false | false | false 0.92 | 0.00 to 0.17 |
| `feather::speed @ h = 10/1, v = 21/10` | leaf | false | false | false 0.92 | 0.00 to 0.17 |
