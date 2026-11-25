//! Help, tuning and the journal (see care.mitra).

use crate::vocabulary::*;

pub struct Helpline;
impl Helpline {
    // mitra: implements Helpline::alert
    pub fn alert(p: Person, m: Message) -> bool {
        crate::fake::alert(p, m)
    }
}

pub struct Tuning;
impl Tuning {
    /// Tonight: nothing is tuned yet; the judgement is only kept as a note.
    // mitra: implements Tuning::adjust
    pub fn adjust(_m: Message, _p: Person) -> Preferences {
        Preferences {}
    }
}
