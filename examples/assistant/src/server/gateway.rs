//! The single entry (see gateway.mitra).

use crate::server::store::{ConversationStore, Stored};
use crate::vocabulary::*;

/// The size limit of a text message, in bytes: a setting (H15). Our
/// model's window holds it with room (see harness::models).
pub const TEXT_LIMIT: i64 = 4_000;
/// The size limit of a voice message, in bytes: a setting (K8). The fake
/// speech reads the bytes as the words, so it is the text limit here.
pub const VOICE_LIMIT: i64 = 4_000;
/// The largest id a device may use: a setting (K9), far from overflow.
pub const LARGEST_ID: i64 = 1 << 53;

pub struct Accounts;
impl Accounts {
    // mitra: implements Accounts::holder
    pub fn holder(t: Token) -> Option<Holder> {
        crate::fake::holder(t)
    }

    /// The owner's phone in a household: where held calls are told.
    // mitra: implements Accounts::owner
    pub fn owner(household: i64) -> Holder {
        crate::fake::owner(household)
    }
}

pub struct Limits;
impl Limits {
    // mitra: implements Limits::text
    pub fn text() -> i64 {
        crate::fake::text_limit()
    }
    // mitra: implements Limits::voice
    pub fn voice() -> i64 {
        crate::fake::limit_voice()
    }
    // mitra: implements Limits::ids
    pub fn ids() -> i64 {
        crate::fake::largest_id()
    }
}

pub struct Gateway;
impl Gateway {
    // mitra: implements Gateway::may_post
    pub fn may_post(h: Holder, p: Party) -> bool {
        p == Party::Client(h)
    }

    // mitra: implements Gateway::post
    pub fn post(
        store: ConversationStore,
        t: Token,
        e: Envelope,
    ) -> (ConversationStore, Result<Receipt, NetFailure>) {
        if Gateway::admits(Accounts::holder(t), store.clone(), e.clone()) {
            let (store, receipt) = Gateway::receive(store, e);
            (store, Ok(receipt))
        } else {
            (store, Err(NetFailure::Refused))
        }
    }

    // mitra: implements Gateway::admits
    pub fn admits(h: Option<Holder>, store: ConversationStore, e: Envelope) -> bool {
        match h {
            Some(x) => {
                x.household == store.household
                    && Gateway::may_post(x, e.id.from)
                    && Gateway::small(e.body.clone())
                    && e.id.n > 0
                    && e.id.n <= Limits::ids()
                    && !store.conflicts(e)
            }
            None => false,
        }
    }

    // mitra: implements Gateway::small
    pub fn small(b: Body) -> bool {
        match b {
            Body::Text(t) => t.len() as i64 <= Limits::text(),
            Body::Voice(v) => crate::server::speech::Speech::bytes(v) <= Limits::voice(),
        }
    }

    // mitra: implements Gateway::receive
    pub fn receive(store: ConversationStore, e: Envelope) -> (ConversationStore, Receipt) {
        let id = e.id;
        (store.record(e), Receipt::Received(id))
    }

    // mitra: implements Gateway::fetch
    pub fn fetch(store: ConversationStore, t: Token, after: i64) -> Vec<Stored> {
        match Accounts::holder(t) {
            Some(h) if h.household == store.household => Gateway::replies(store, h, after),
            _ => vec![],
        }
    }

    // mitra: implements Gateway::replies
    pub fn replies(store: ConversationStore, to: Holder, after: i64) -> Vec<Stored> {
        store.for_client(to, after)
    }

    // mitra: implements Gateway::delivered
    pub fn delivered(store: ConversationStore, t: Token, id: MessageId) -> ConversationStore {
        match Accounts::holder(t) {
            Some(h) => store.annotate_for(
                id,
                h,
                Annotation {
                    by: Step::Gateway,
                    note: Note::Delivered,
                },
            ),
            None => store,
        }
    }

    // mitra: implements Gateway::receipt
    /// Where a client's own message stands (W12).
    pub fn receipt(store: ConversationStore, t: Token, n: i64) -> Option<Receipt> {
        match Accounts::holder(t) {
            Some(h) if h.household == store.household => Gateway::status(
                store,
                MessageId {
                    from: Party::Client(h),
                    n,
                },
            ),
            _ => Gateway::no_receipt(),
        }
    }

    // mitra: implements Gateway::status
    pub fn status(store: ConversationStore, id: MessageId) -> Option<Receipt> {
        store
            .messages
            .iter()
            .rfind(|m| m.id == id)
            .map(|m| Gateway::stage(id, m.read(), store.reply_delivered(m)))
    }

    // mitra: implements Gateway::stage
    pub fn stage(id: MessageId, read: bool, delivered: bool) -> Receipt {
        if delivered {
            Receipt::Delivered(id)
        } else if read {
            Receipt::Read(id)
        } else {
            Receipt::Received(id)
        }
    }

    // mitra: implements Gateway::no_receipt
    pub fn no_receipt() -> Option<Receipt> {
        None
    }

    // mitra: implements Gateway::typing_for
    pub fn typing_for(store: ConversationStore, t: Token) -> bool {
        match Accounts::holder(t) {
            Some(h) => Gateway::typing(store, h),
            None => false,
        }
    }

    // mitra: implements Gateway::typing
    pub fn typing(store: ConversationStore, d: Holder) -> bool {
        store
            .messages
            .iter()
            .any(|m| m.id.from == Party::Client(d) && m.read() && !m.answered())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake;

    fn computer() -> Holder {
        Holder {
            household: 1,
            device: Device::Computer,
            number: 1,
        }
    }
    fn token(t: &str) -> Token {
        Token { secret: s(t) }
    }
    fn hello(from: Holder) -> Envelope {
        Envelope {
            id: MessageId {
                from: Party::Client(from),
                n: 1,
            },
            body: Body::Text(s("hello")),
        }
    }

    #[test]
    fn a_paired_device_posts_as_itself() {
        fake::reset();
        fake::pair("cli", computer(), Role::Member);
        let stored = ConversationStore::new(1).record(hello(computer()));
        mitra::check_eq!(
            Gateway::post(ConversationStore::new(1), token("cli"), hello(computer())),
            (
                stored,
                Ok::<Receipt, NetFailure>(Receipt::Received(hello(computer()).id))
            )
        );
    }

    #[test]
    fn a_device_posting_as_another_is_refused() {
        fake::reset();
        fake::pair(
            "wear",
            Holder {
                device: Device::Wearable,
                ..computer()
            },
            Role::Member,
        );
        mitra::check_eq!(
            Gateway::post(ConversationStore::new(1), token("wear"), hello(computer())),
            (
                ConversationStore::new(1),
                Err::<Receipt, NetFailure>(NetFailure::Refused)
            )
        );
    }

    #[test]
    fn another_household_is_refused() {
        fake::reset();
        let other = Holder {
            household: 2,
            ..computer()
        };
        fake::pair("other", other, Role::Owner);
        mitra::check_eq!(
            Gateway::post(ConversationStore::new(1), token("other"), hello(other)),
            (
                ConversationStore::new(1),
                Err::<Receipt, NetFailure>(NetFailure::Refused)
            )
        );
    }

    // mitra: supports Gateway::sized
    #[test]
    fn a_text_over_the_limit_is_refused() {
        fake::reset();
        mitra::check_eq!(
            Gateway::small(Body::Text("x".repeat(TEXT_LIMIT as usize + 1))),
            false
        );
        mitra::check_eq!(Gateway::small(Body::Text(s("hello"))), true);
    }
}
