//! What the trials share: the Beta of an opinion and where it lies.
#![allow(dead_code)]

use mitra_core::model::Opinion;

/// ln of the gamma function (Lanczos).
pub fn ln_gamma(x: f64) -> f64 {
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
pub fn ln_beta(a: f64, b: f64) -> f64 {
    ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)
}

/// The regularised incomplete beta function: the chance that a
/// Beta(a, b) is under `x` (continued fraction, Lentz).
pub fn beta_under(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    if x > (a + 1.0) / (a + b + 2.0) {
        return 1.0 - beta_under(1.0 - x, b, a);
    }
    let front = (a * x.ln() + b * (1.0 - x).ln() - ln_beta(a, b)).exp() / a;
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
pub fn beta_at(q: f64, a: f64, b: f64) -> f64 {
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

/// How often the piece is right, nine times out of ten: from the 5th
/// to the 95th hundredth of its Beta.
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
