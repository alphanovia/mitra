//! Something said, right or wrong, and the opinion of it.

/// Something said, right or wrong. Known by its name only.
// mitra: implements piece::Piece
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Piece {
    pub name: String,
}

impl Piece {
    pub fn named(name: &str) -> Piece {
        Piece {
            name: name.to_string(),
        }
    }
}

// mitra: implements piece::piece_id
pub fn piece_id(x: &Piece) -> &str {
    &x.name
}

/// What is believed of a piece, as counts: `r` of yes, `s` of no. `a`
/// is the base rate: what to expect with no evidence.
// mitra: implements piece::Opinion
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opinion {
    pub r: f64,
    pub s: f64,
    pub a: f64,
}

/// No evidence: the base rate alone.
// mitra: implements piece::nothing
pub fn nothing(a: f64) -> Opinion {
    Opinion { r: 0.0, s: 0.0, a }
}

/// One yes.
// mitra: implements piece::yes
pub fn yes() -> Opinion {
    Opinion {
        r: 1.0,
        s: 0.0,
        a: 1.0,
    }
}

/// One no.
// mitra: implements piece::no
pub fn no() -> Opinion {
    Opinion {
        r: 0.0,
        s: 1.0,
        a: 0.0,
    }
}

/// The number an opinion reads as: the share of yes, the base rate
/// counted `w` times. `w` is the weight of what is expected before any
/// evidence, in counts: a setting of the whole system, given by the
/// model, never chosen here.
// mitra: implements piece::projected
pub fn projected(o: Opinion, w: f64) -> f64 {
    (o.r + o.a * w) / (o.r + o.s + w)
}

/// The share of an opinion that is evidence for.
// mitra: implements piece::belief
pub fn belief(o: Opinion, w: f64) -> f64 {
    o.r / (o.r + o.s + w)
}

/// The share of an opinion that is evidence against.
// mitra: implements piece::disbelief
pub fn disbelief(o: Opinion, w: f64) -> f64 {
    o.s / (o.r + o.s + w)
}

/// How much of an opinion is not evidence: 1 with none, towards 0 as
/// the counts grow.
// mitra: implements piece::uncertainty
pub fn uncertainty(o: Opinion, w: f64) -> f64 {
    w / (o.r + o.s + w)
}

/// Believed enough for the policy, and never with no yes at all.
// mitra: implements piece::reaches
pub fn reaches(o: Opinion, policy: f64, w: f64) -> bool {
    o.r > 0.0 && projected(o, w) >= policy
}

/// The evidence against is at least as strong as the evidence for.
// mitra: implements piece::against
pub fn against(o: Opinion) -> bool {
    o.s > 0.0 && o.s >= o.r
}

/// Where the piece is right nine times out of ten: how often it is
/// right is a Beta of `r + a·w` yes and `s + (1 - a)·w` no, and this
/// is from its 5th to its 95th hundredth.
// mitra: implements piece::interval
pub fn interval(o: Opinion, w: f64) -> (f64, f64) {
    let (a, b) = (o.r + o.a * w, o.s + (1.0 - o.a) * w);
    if a <= 0.0 {
        return (0.0, 0.0);
    }
    if b <= 0.0 {
        return (1.0, 1.0);
    }
    (beta_at(0.05, a, b), beta_at(0.95, a, b))
}

/// ln of the gamma function (Lanczos).
fn ln_gamma(x: f64) -> f64 {
    const G: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        let pi = std::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let t = x + 7.5;
    let mut sum = G[0];
    for (k, g) in G.iter().enumerate().skip(1) {
        sum += g / (x + k as f64);
    }
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + sum.ln()
}

/// The chance that a Beta(a, b) is under `x` (continued fraction, Lentz).
fn beta_under(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    if x > (a + 1.0) / (a + b + 2.0) {
        return 1.0 - beta_under(1.0 - x, b, a);
    }
    let ln_beta = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
    let front = (a * x.ln() + b * (1.0 - x).ln() - ln_beta).exp() / a;
    let tiny = 1e-300;
    let (mut c, mut d) = (1.0, 1.0 - (a + b) * x / (a + 1.0));
    d = 1.0 / if d.abs() < tiny { tiny } else { d };
    let mut h = d;
    for m in 1..400 {
        let m = m as f64;
        for step in [
            m * (b - m) * x / ((a + 2.0 * m - 1.0) * (a + 2.0 * m)),
            -(a + m) * (a + b + m) * x / ((a + 2.0 * m) * (a + 2.0 * m + 1.0)),
        ] {
            d = 1.0 + step * d;
            d = 1.0 / if d.abs() < tiny { tiny } else { d };
            c = 1.0 + step / c;
            c = if c.abs() < tiny { tiny } else { c };
            h *= d * c;
        }
    }
    front * h
}

/// Where a Beta(a, b) is under `x` with chance `q`.
fn beta_at(q: f64, a: f64, b: f64) -> f64 {
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..60 {
        let mid = (low + high) / 2.0;
        match beta_under(mid, a, b) < q {
            true => low = mid,
            false => high = mid,
        }
    }
    (low + high) / 2.0
}
