//! Rung 1 of the ladder: a clock of 24 hours. The model is in `model/`.

// mitra: implements clock::next
pub fn next(h: u8) -> u8 {
    if h == 23 { 0 } else { h + 1 }
}

// mitra: implements clock::back
pub fn back(h: u8) -> u8 {
    if h == 0 { 23 } else { h - 1 }
}

// mitra: implements clock::after
pub fn after(h: u8, n: u64) -> u8 {
    let mut h = h;
    for _ in 0..n % 24 {
        h = next(h);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    // mitra: evidence for clock::steps given h in 0..=22 act exhaustive written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn every_hour_steps() {
        for h in 0..=22 {
            assert_eq!(next(h), h + 1);
        }
    }

    // mitra: evidence for clock::midnight written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn midnight_wraps() {
        assert_eq!(next(23), 0);
    }

    // mitra: evidence for clock::round_trip given h in 0..=23 act exhaustive written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn every_hour_comes_back() {
        for h in 0..=23 {
            assert_eq!(back(next(h)), h);
            assert_eq!(next(back(h)), h);
        }
    }

    // mitra: evidence for clock::stays_put given h in 0..=23 act exhaustive written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn no_hours_is_no_move() {
        for h in 0..=23 {
            assert_eq!(after(h, 0), h);
        }
    }

    // Every hour, and 125 numbers of hours spread under 1000.
    // mitra: evidence for clock::one_more given n in day sampled 3000 written "agent:anthropic/fable-5.2/ladder"
    #[test]
    fn one_more_hour_is_one_more_step() {
        for h in 0..=23 {
            for i in 0..125u64 {
                let n = i * 7919 % 1000;
                assert_eq!(after(h, n + 1), next(after(h, n)));
            }
        }
    }
}
