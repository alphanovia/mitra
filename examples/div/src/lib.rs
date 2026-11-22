// mitra: implements calc::div
pub fn div(a: u32, b: u32) -> u32 {
    if b == 0 {
        0
    } else {
        a / b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // mitra: measures calc::div (6, 2) -> 3
    #[test]
    fn div_by_two() {
        assert_eq!(div(6, 2), 3);
    }

    // mitra: measures calc::div (7, 0) -> 0
    #[test]
    fn div_by_zero() {
        assert_eq!(div(7, 0), 0);
    }

    // mitra: measures calc::div (9, 3) -> 3
    #[test]
    fn div_by_three() {
        assert_eq!(div(9, 3), 3);
    }
}
