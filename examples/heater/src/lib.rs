/// A heater: a target temperature, and whether it heats.
// mitra: implements Heater
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(mitra::Observe))]
pub struct Heater {
    pub target: i64,
    pub on: bool,
}

impl Heater {
    /// On below the target, off at or above it.
    // mitra: implements Heater::decide
    pub fn decide(self, t: i64) -> Heater {
        Heater {
            on: t < self.target,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heats_below_the_target() {
        let cold = Heater {
            target: 20,
            on: false,
        };
        mitra::check_eq!(
            Heater::decide(cold, 18),
            Heater {
                target: 20,
                on: true
            }
        );
    }

    #[test]
    fn rests_at_the_target() {
        let warm = Heater {
            target: 20,
            on: true,
        };
        mitra::check_eq!(
            Heater::decide(warm, 20),
            Heater {
                target: 20,
                on: false
            }
        );
    }

    #[test]
    fn decides_the_same_twice() {
        let h = Heater {
            target: 20,
            on: false,
        };
        let once = h.clone().decide(18);
        mitra::check_eq!(Heater::decide(once.clone(), 18), once);
    }
}
