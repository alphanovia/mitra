//! The conversation store: bodies fixed, notes only added (see store.mitra).
//! In memory tonight: `Disk::durable` is not built (a POC).

use crate::harness::conversation::Conversation;
use crate::vocabulary::*;
use mitra::Observe;

// mitra: implements Stored
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Stored {
    pub id: MessageId,
    pub to: Party,
    pub body: Body,
    pub notes: Vec<Annotation>,
}

impl Stored {
    // mitra: implements Stored::annotate
    pub fn annotate(self, a: Annotation) -> Stored {
        let mut notes = self.notes;
        notes.push(a);
        Stored {
            id: self.id,
            to: self.to,
            body: self.body,
            notes,
        }
    }

    // mitra: implements Stored::text
    pub fn text(self) -> Option<String> {
        match self.body {
            Body::Text(s) => Some(s),
            Body::Voice(_) => self.notes.iter().rev().find_map(|a| match &a.note {
                Note::Transcript(s) => Some(s.clone()),
                _ => None,
            }),
        }
    }

    // mitra: implements Stored::read
    pub fn read(&self) -> bool {
        self.notes.iter().any(|a| a.note == Note::Read)
    }
    // mitra: implements Stored::answered
    pub fn answered(&self) -> bool {
        self.notes
            .iter()
            .any(|a| matches!(a.note, Note::Answered(_)))
    }
    // mitra: implements Stored::acted
    pub fn acted(&self) -> bool {
        self.notes
            .iter()
            .any(|a| a.note == Note::Verdict(Verdict::Act))
    }
    // mitra: implements Stored::vouched
    pub fn vouched(&self) -> bool {
        self.notes
            .iter()
            .any(|a| matches!(a.note, Note::Sender(p) if p.role != Role::Stranger))
    }
    // mitra: implements Stored::read_output
    pub fn read_output(&self) -> bool {
        self.notes
            .iter()
            .any(|a| matches!(&a.note, Note::Called(o) if Stored::ran(o.clone())))
    }
    // mitra: implements Stored::ran
    pub fn ran(o: Outcome) -> bool {
        o.output.is_ok()
    }
    // mitra: implements Stored::answer_is
    pub fn answer_is(&self, id: MessageId) -> bool {
        self.notes.iter().any(|a| a.note == Note::Answered(id))
    }
    // mitra: implements Stored::delivered
    pub fn delivered(&self) -> bool {
        self.notes.iter().any(|a| a.note == Note::Delivered)
    }
    // mitra: implements Stored::speech
    pub fn speech(&self) -> Option<Audio> {
        self.notes.iter().rev().find_map(|a| match &a.note {
            Note::Speech(v) => Some(v.clone()),
            _ => None,
        })
    }
}

// mitra: implements ConversationStore
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct ConversationStore {
    pub household: i64,
    pub messages: Vec<Stored>,
}

impl ConversationStore {
    pub fn new(household: i64) -> ConversationStore {
        ConversationStore {
            household,
            messages: vec![],
        }
    }

    // mitra: implements ConversationStore::holds
    pub fn holds(&self, id: MessageId) -> bool {
        self.messages.iter().any(|m| m.id == id)
    }

    // mitra: implements ConversationStore::conflicts
    pub fn conflicts(&self, e: Envelope) -> bool {
        self.messages
            .iter()
            .any(|m| m.id == e.id && m.body != e.body)
    }

    // mitra: implements ConversationStore::held
    pub fn held(&self, code: i64) -> Option<Call> {
        let mut c = None;
        for m in &self.messages {
            for a in &m.notes {
                match &a.note {
                    Note::Held(x) if x.code == code => c = Some(x.call.clone()),
                    Note::Confirmed(y) if *y == code => c = None,
                    _ => {}
                }
            }
        }
        c
    }

    // mitra: implements ConversationStore::record
    pub fn record(self, e: Envelope) -> ConversationStore {
        if self.holds(e.id) {
            return self;
        }
        let mut messages = self.messages;
        messages.push(Stored {
            id: e.id,
            to: Party::Assistant,
            body: e.body,
            notes: vec![],
        });
        ConversationStore {
            household: self.household,
            messages,
        }
    }

    // mitra: implements ConversationStore::annotate
    pub fn annotate(self, id: MessageId, a: Annotation) -> ConversationStore {
        ConversationStore {
            household: self.household,
            messages: self
                .messages
                .into_iter()
                .map(|m| if m.id == id { m.annotate(a.clone()) } else { m })
                .collect(),
        }
    }

    // mitra: implements ConversationStore::annotate_for
    pub fn annotate_for(self, id: MessageId, h: Holder, a: Annotation) -> ConversationStore {
        ConversationStore {
            household: self.household,
            messages: self
                .messages
                .into_iter()
                .map(|m| {
                    if m.id == id && m.to == Party::Client(h) {
                        m.annotate(a.clone())
                    } else {
                        m
                    }
                })
                .collect(),
        }
    }

    // mitra: implements ConversationStore::annotate_all
    pub fn annotate_all(self, id: MessageId, ns: Vec<Annotation>) -> ConversationStore {
        ns.into_iter().fold(self, |s, a| s.annotate(id, a))
    }

    // mitra: implements ConversationStore::next_id
    pub fn next_id(&self) -> MessageId {
        MessageId {
            from: Party::Assistant,
            n: self.messages.len() as i64 + 1,
        }
    }

    // mitra: implements ConversationStore::post
    pub fn post(self, r: Stored) -> ConversationStore {
        let mut messages = self.messages;
        messages.push(r);
        ConversationStore {
            household: self.household,
            messages,
        }
    }

    // mitra: implements ConversationStore::for_client
    pub fn for_client(&self, h: Holder, after: i64) -> Vec<Stored> {
        self.messages
            .iter()
            .filter(|m| m.to == Party::Client(h) && m.id.n > after)
            .cloned()
            .collect()
    }

    // mitra: implements ConversationStore::history
    pub fn history(&self, h: Holder) -> Conversation {
        let mut turns = vec![];
        for m in &self.messages {
            if m.id.from == Party::Client(h) && m.answered() && m.acted() && m.vouched() {
                if let Some(t) = m.clone().text() {
                    turns.push(Turn::User(t));
                    turns.extend(self.reply_turns(m.clone()));
                }
            }
        }
        Conversation { turns }
    }

    // mitra: implements ConversationStore::reply_delivered
    pub fn reply_delivered(&self, m: &Stored) -> bool {
        self.messages
            .iter()
            .any(|x| m.answer_is(x.id) && x.delivered())
    }

    // mitra: implements ConversationStore::reply_turns
    pub fn reply_turns(&self, m: Stored) -> Vec<Turn> {
        if m.read_output() {
            return vec![];
        }
        self.messages
            .iter()
            .filter(|r| m.answer_is(r.id))
            .filter_map(|r| match &r.body {
                Body::Text(t) => Some(Turn::Model(Answer {
                    text: t.clone(),
                    calls: vec![],
                })),
                Body::Voice(_) => None,
            })
            .collect()
    }

    /// The client holders that wrote to this store, in order of first message.
    pub fn holders(&self) -> Vec<Holder> {
        let mut hs = vec![];
        for m in &self.messages {
            if let Party::Client(h) = m.id.from {
                if !hs.contains(&h) {
                    hs.push(h);
                }
            }
        }
        hs
    }
}

/// The calls written down before they run (H13). Kept by the fake world
/// tonight, as the store is kept in memory: `Disk::durable` is not built.
pub struct Ahead;
impl Ahead {
    // mitra: implements Ahead::mark
    pub fn mark(c: Call) {
        crate::fake::mark(c)
    }
    // mitra: implements Ahead::marked
    pub fn marked() -> Vec<Call> {
        crate::fake::marked()
    }
    // mitra: implements Ahead::clear
    pub fn clear() {
        crate::fake::clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn computer() -> Holder {
        Holder {
            household: 1,
            device: Device::Computer,
            number: 1,
        }
    }
    fn hello() -> Envelope {
        Envelope {
            id: MessageId {
                from: Party::Client(computer()),
                n: 1,
            },
            body: Body::Text(s("hello")),
        }
    }

    #[test]
    fn a_message_is_kept_once() {
        let one = ConversationStore::new(1).record(hello());
        mitra::check_eq!(ConversationStore::record(one.clone(), hello()), one);
    }

    #[test]
    fn the_same_id_with_other_words_conflicts() {
        let one = ConversationStore::new(1).record(hello());
        let other = Envelope {
            body: Body::Text(s("open the door")),
            ..hello()
        };
        mitra::check_eq!(ConversationStore::conflicts(&one, other), true);
        mitra::check_eq!(ConversationStore::conflicts(&one, hello()), false);
    }

    #[test]
    fn a_note_is_added_after_the_others() {
        let m = Stored {
            id: hello().id,
            to: Party::Assistant,
            body: Body::Voice(Audio { bytes: s("hi") }),
            notes: vec![],
        };
        let a = Annotation {
            by: Step::Speech,
            note: Note::Transcript(s("hello")),
        };
        mitra::check_eq!(
            Stored::annotate(m.clone(), a.clone()),
            Stored {
                notes: vec![a],
                ..m
            }
        );
    }

    // mitra: supports Confirmations::guessing_fails
    #[test]
    fn a_code_works_once() {
        let call = Call {
            tool: s("door"),
            input: s("open"),
        };
        let held = ConversationStore::new(1).record(hello()).annotate(
            hello().id,
            Annotation {
                by: Step::Policy,
                note: Note::Held(Hold {
                    code: 7,
                    call: call.clone(),
                    asked: s("open the door"),
                }),
            },
        );
        mitra::check_eq!(ConversationStore::held(&held, 7), Some(call));
        mitra::check_eq!(ConversationStore::held(&held, 8), None);
        let used = held.annotate(
            hello().id,
            Annotation {
                by: Step::Confirmations,
                note: Note::Confirmed(7),
            },
        );
        mitra::check_eq!(ConversationStore::held(&used, 7), None);
    }
}
