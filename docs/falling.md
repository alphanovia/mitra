# The fall, from a value to a verdict

Printed by `cargo run --example falling`. The model is `tests/falling.rs`.
Each measurement is evaluated, a timer takes what that gave, and the
calculus judges every piece. The keyword of a piece is read from the
type of what it is about. A dash means nothing was evaluated or taken.
A deduction is not listed: it is never judged.

`stone::fall` is Partial: declared, with two claims.

### The stone, three measurements, nothing reviewed

Verdict: unknown

| piece | keyword | step | evaluated | taken | judged |
|---|---|---|---|---|---|
| `stone::falls` | ensure | induction | - | - | unknown |
| `stone::only the height matters` | require | leaf | - | - | unknown |
| `stone::speed` | claim | leaf | - | - | unknown |
| `stone::slow` | claim | leaf | - | - | unknown |
| `stone::fall is the drop in the tower` | tie | leaf | - | true | true 0.92 |
| `stone::speed @ h = 5/1, v = 99/10` | evidence | leaf | true | true | true 0.92 |
| `stone::speed @ h = 10/1, v = 14/1` | evidence | leaf | true | true | true 0.92 |
| `stone::speed @ h = 20/1, v = 99/5` | evidence | leaf | true | true | true 0.92 |

### The stone, reviewed

Verdict: true 0.86, least sure: `stone::falls`

| piece | keyword | step | evaluated | taken | judged |
|---|---|---|---|---|---|
| `stone::falls` | ensure | induction | - | - | true 0.86 |
| `stone::only the height matters` | require | leaf | - | true | true 0.92 |
| `stone::speed` | claim | leaf | - | - | true 0.86 |
| `stone::slow` | claim | leaf | - | - | true 0.86 |
| `stone::fall is the drop in the tower` | tie | leaf | - | true | true 0.92 |
| `stone::speed @ h = 5/1, v = 99/10` | evidence | leaf | true | true | true 0.92 |
| `stone::speed @ h = 10/1, v = 14/1` | evidence | leaf | true | true | true 0.92 |
| `stone::speed @ h = 20/1, v = 99/5` | evidence | leaf | true | true | true 0.92 |
| `stone::represents` | ensure | induction | - | true | true 0.92 |

### The stone, reviewed, then one bad measurement

Verdict: conflict

| piece | keyword | step | evaluated | taken | judged |
|---|---|---|---|---|---|
| `stone::falls` | ensure | induction | - | - | conflict |
| `stone::only the height matters` | require | leaf | - | true | true 0.92 |
| `stone::speed` | claim | leaf | - | - | conflict |
| `stone::slow` | claim | leaf | - | - | conflict |
| `stone::fall is the drop in the tower` | tie | leaf | - | true | conflict |
| `stone::speed @ h = 5/1, v = 99/10` | evidence | leaf | true | true | conflict |
| `stone::speed @ h = 10/1, v = 14/1` | evidence | leaf | true | true | conflict |
| `stone::speed @ h = 20/1, v = 99/5` | evidence | leaf | true | true | conflict |
| `stone::speed @ h = 30/1, v = 99/10` | evidence | leaf | false | false | conflict |
| `stone::represents` | ensure | induction | - | true | conflict |

Suspects: `stone::speed @ h = 5/1, v = 99/10`

Suspects: `stone::speed @ h = 10/1, v = 14/1`

Suspects: `stone::speed @ h = 20/1, v = 99/5`

Suspects: `stone::fall is the drop in the tower`

Suspects: `stone::represents`

Suspects: `stone::speed @ h = 30/1, v = 99/10`

### The feather, the require not taken

Verdict: false

| piece | keyword | step | evaluated | taken | judged |
|---|---|---|---|---|---|
| `feather::falls` | ensure | induction | - | - | unknown |
| `feather::only the height matters` | require | leaf | - | - | unknown |
| `feather::speed` | claim | leaf | - | - | false 0.92 |
| `feather::slow` | claim | leaf | - | - | unknown |
| `feather::fall is the drop in the tower` | tie | leaf | - | true | true 0.92 |
| `feather::speed @ h = 5/1, v = 2/1` | evidence | leaf | false | false | false 0.92 |
| `feather::speed @ h = 10/1, v = 21/10` | evidence | leaf | false | false | false 0.92 |

Suspects: `feather::falls`, `feather::only the height matters`

### The feather, the require taken

Verdict: false

| piece | keyword | step | evaluated | taken | judged |
|---|---|---|---|---|---|
| `feather::falls` | ensure | induction | - | - | false 0.92 |
| `feather::only the height matters` | require | leaf | - | true | true 0.92 |
| `feather::speed` | claim | leaf | - | - | false 0.92 |
| `feather::slow` | claim | leaf | - | - | unknown |
| `feather::fall is the drop in the tower` | tie | leaf | - | true | true 0.92 |
| `feather::speed @ h = 5/1, v = 2/1` | evidence | leaf | false | false | false 0.92 |
| `feather::speed @ h = 10/1, v = 21/10` | evidence | leaf | false | false | false 0.92 |
