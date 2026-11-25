//! The owner's yes (see confirm.mitra). Plain code: no model reads it.

use crate::harness::harness::Harness;
use crate::harness::reply::Reporter;
use crate::harness::security::Identity;
use crate::harness::tools::ToolGuard;
use crate::server::store::ConversationStore;
use crate::vocabulary::*;

pub struct Codes;
impl Codes {
    // mitra: implements Codes::fresh
    pub fn fresh() -> i64 {
        crate::fake::fresh()
    }
}

pub struct Confirmations;
impl Confirmations {
    /// "yes 123456", any case, spaces around: the code. Anything else,
    /// a plain "yes" too, is not a confirmation and goes to the harness.
    // mitra: implements Confirmations::code
    pub fn code(text: String) -> Option<i64> {
        let t = text.trim().to_lowercase();
        let rest = t.strip_prefix("yes ")?.trim();
        if !rest.is_empty() && rest.len() <= 9 && rest.bytes().all(|b| b.is_ascii_digit()) {
            rest.parse().ok()
        } else {
            None
        }
    }

    // mitra: implements Confirmations::confirm
    pub fn confirm(
        store: ConversationStore,
        c: i64,
        m: Message,
        tools: Vec<Tool>,
    ) -> (Reply, Vec<Annotation>) {
        let p = Identity::who(m.clone());
        let mut notes = vec![Annotation {
            by: Step::Identity,
            note: Note::Sender(p),
        }];
        match store.held(c) {
            Some(call) if Confirmations::allowed(p.role, m.recognised) => {
                let o = Outcome {
                    call: call.clone(),
                    output: Confirmations::run(call, tools),
                };
                notes.push(Annotation {
                    by: Step::Confirmations,
                    note: Note::Confirmed(c),
                });
                notes.push(Annotation {
                    by: Step::Confirmations,
                    note: Note::Called(o.clone()),
                });
                (Reporter::confirmed(o), notes)
            }
            _ => (Reporter::not_allowed(), notes),
        }
    }

    // mitra: implements Confirmations::allowed
    pub fn allowed(role: Role, recognised: bool) -> bool {
        role == Role::Owner && recognised
    }

    // mitra: implements Confirmations::run
    pub fn run(c: Call, tools: Vec<Tool>) -> Result<String, ToolFailure> {
        match Harness::lookup(c.tool.clone(), tools) {
            Some(t) => ToolGuard::screen(t.call(c.input)),
            None => Harness::unknown(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // mitra: supports Worker::yes_by_code
    #[test]
    fn a_yes_is_read_by_plain_code() {
        mitra::check_eq!(Confirmations::code(s("yes 123456")), Some(123456));
        mitra::check_eq!(Confirmations::code(s("  YES 42 ")), Some(42));
        mitra::check_eq!(Confirmations::code(s("yes")), None);
        mitra::check_eq!(Confirmations::code(s("yes 12a")), None);
        mitra::check_eq!(Confirmations::code(s("yes 1234567890123")), None);
        mitra::check_eq!(Confirmations::code(s("I said yes 123456")), None);
    }

    #[test]
    fn only_the_vouched_owner_may_say_yes() {
        mitra::check_eq!(Confirmations::allowed(Role::Owner, true), true);
        mitra::check_eq!(Confirmations::allowed(Role::Owner, false), false);
        mitra::check_eq!(Confirmations::allowed(Role::Member, true), false);
        mitra::check_eq!(Confirmations::allowed(Role::Stranger, true), false);
    }
}
