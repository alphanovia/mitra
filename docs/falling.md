# The fall, from a value to a verdict

Printed by `cargo run --example falling`. The model is `tests/falling.rs`.
Each measurement is evaluated, a timer takes what that gave, and the
calculus judges every piece. A dash means nothing was evaluated or taken.

### The stone, three measurements, nothing reviewed

Verdict: unknown

| piece | step | evaluated | taken | judged |
|---|---|---|---|---|
| `stone::falls` | deduction | - | - | unknown |
| `stone::only the height matters` | leaf | - | - | unknown |
| `stone::speed` | leaf | - | - | unknown |
| `stone::stays slow` | deduction | - | - | unknown |
| `stone::slow` | leaf | - | - | unknown |
| `stone::speed @ h = 5/1, v = 99/10` | leaf | true | true | true 0.92 |
| `stone::speed @ h = 10/1, v = 14/1` | leaf | true | true | true 0.92 |
| `stone::speed @ h = 20/1, v = 99/5` | leaf | true | true | true 0.92 |

### The stone, reviewed

Verdict: true 0.92, least sure: `stone::falls`

| piece | step | evaluated | taken | judged |
|---|---|---|---|---|
| `stone::falls` | deduction | - | - | true 0.92 |
| `stone::only the height matters` | leaf | - | true | true 0.92 |
| `stone::speed` | leaf | - | - | true 0.92 |
| `stone::stays slow` | deduction | - | true | true 0.92 |
| `stone::slow` | leaf | - | - | true 0.92 |
| `stone::speed @ h = 5/1, v = 99/10` | leaf | true | true | true 0.92 |
| `stone::speed @ h = 10/1, v = 14/1` | leaf | true | true | true 0.92 |
| `stone::speed @ h = 20/1, v = 99/5` | leaf | true | true | true 0.92 |
| `stone::represents` | induction | - | true | true 0.92 |

### The stone, reviewed, then one bad measurement

Verdict: conflict

| piece | step | evaluated | taken | judged |
|---|---|---|---|---|
| `stone::falls` | deduction | - | - | conflict |
| `stone::only the height matters` | leaf | - | true | true 0.92 |
| `stone::speed` | leaf | - | - | conflict |
| `stone::stays slow` | deduction | - | true | true 0.92 |
| `stone::slow` | leaf | - | - | true 0.92 |
| `stone::speed @ h = 5/1, v = 99/10` | leaf | true | true | conflict |
| `stone::speed @ h = 10/1, v = 14/1` | leaf | true | true | conflict |
| `stone::speed @ h = 20/1, v = 99/5` | leaf | true | true | conflict |
| `stone::speed @ h = 30/1, v = 99/10` | leaf | false | false | conflict |
| `stone::represents` | induction | - | true | conflict |

Cannot all be true: `stone::speed @ h = 5/1, v = 99/10`

Cannot all be true: `stone::speed @ h = 10/1, v = 14/1`

Cannot all be true: `stone::speed @ h = 20/1, v = 99/5`

Cannot all be true: `stone::represents`

Cannot all be true: `stone::speed @ h = 30/1, v = 99/10`

### The feather, hypothesis not taken

Verdict: false

| piece | step | evaluated | taken | judged |
|---|---|---|---|---|
| `feather::falls` | deduction | - | - | unknown |
| `feather::only the height matters` | leaf | - | - | unknown |
| `feather::speed` | leaf | - | - | false 0.95 |
| `feather::stays slow` | deduction | - | - | unknown |
| `feather::slow` | leaf | - | - | unknown |
| `feather::speed @ h = 5/1, v = 2/1` | leaf | false | false | false 0.92 |
| `feather::speed @ h = 10/1, v = 21/10` | leaf | false | false | false 0.92 |

Cannot all be true: `feather::falls`, `feather::only the height matters`

### The feather, hypothesis taken

Verdict: false

| piece | step | evaluated | taken | judged |
|---|---|---|---|---|
| `feather::falls` | deduction | - | - | false 0.92 |
| `feather::only the height matters` | leaf | - | true | true 0.92 |
| `feather::speed` | leaf | - | - | false 0.95 |
| `feather::stays slow` | deduction | - | - | unknown |
| `feather::slow` | leaf | - | - | unknown |
| `feather::speed @ h = 5/1, v = 2/1` | leaf | false | false | false 0.92 |
| `feather::speed @ h = 10/1, v = 21/10` | leaf | false | false | false 0.92 |
