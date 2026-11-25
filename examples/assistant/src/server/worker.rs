//! The worker: one message from taken up to answered (see worker.mitra).

use crate::harness::harness::Harness;
use crate::harness::models::Models;
use crate::harness::reply::Reporter;
use crate::server::confirm::Confirmations;
use crate::server::gateway::Accounts;
use crate::server::speech::Speech;
use crate::server::store::{ConversationStore, Stored};
use crate::vocabulary::*;

pub struct Worker;

fn note(by: Step, note: Note) -> Annotation {
    Annotation { by, note }
}

impl Worker {
    // mitra: implements Worker::address
    pub fn address(store: ConversationStore, m: Stored) -> (MessageId, Party) {
        (store.next_id(), m.id.from)
    }

    // mitra: implements Worker::device
    pub fn device(p: Party) -> Holder {
        match p {
            Party::Client(x) => x,
            Party::Assistant => Holder {
                household: 0,
                device: Device::Computer,
                number: 0,
            },
        }
    }

    // mitra: implements Worker::words
    pub fn words(b: Body) -> (String, Vec<Annotation>, bool) {
        match b {
            Body::Text(t) => (t, vec![], true),
            Body::Voice(v) => match Speech::transcribe(v.clone()) {
                Ok(t) => (
                    t.clone(),
                    vec![note(Step::Speech, Note::Transcript(t))],
                    Speech::speaker(v),
                ),
                Err(_) => (String::new(), vec![], false),
            },
        }
    }

    // mitra: implements Worker::voiced
    pub fn voiced(b: Body, text: String) -> Vec<Annotation> {
        match b {
            Body::Text(_) => vec![],
            Body::Voice(_) => vec![note(Step::Speech, Note::Speech(Speech::speak(text)))],
        }
    }

    // mitra: implements Worker::next
    pub fn next(store: ConversationStore, d: Holder) -> Option<Stored> {
        if Worker::busy(store.clone(), d) {
            return None;
        }
        store
            .messages
            .into_iter()
            .find(|s| s.id.from == Party::Client(d) && !s.read())
    }

    // mitra: implements Worker::cut_short
    pub fn cut_short(store: ConversationStore, d: Holder) -> Option<Stored> {
        store
            .messages
            .into_iter()
            .rev()
            .find(|s| s.id.from == Party::Client(d) && s.read() && !s.answered())
    }

    // mitra: implements Worker::busy
    pub fn busy(store: ConversationStore, d: Holder) -> bool {
        store
            .messages
            .iter()
            .any(|s| s.id.from == Party::Client(d) && s.read() && !s.answered())
    }

    // mitra: implements Worker::resume
    pub fn resume(store: ConversationStore, m: Stored) -> (ConversationStore, Stored) {
        let (id, to) = Worker::address(store.clone(), m.clone());
        let reply = Stored {
            id,
            to,
            body: Body::Text(Reporter::interrupted().text),
            notes: vec![],
        };
        let noted = store.annotate(m.id, note(Step::Worker, Note::Answered(id)));
        (noted.post(reply.clone()), reply)
    }

    // mitra: implements Worker::take_up
    pub fn take_up(store: ConversationStore, m: Stored) -> ConversationStore {
        store.annotate(m.id, note(Step::Worker, Note::Read))
    }

    // mitra: implements Worker::process
    pub fn process(
        taken: ConversationStore,
        m: Stored,
        models: Models,
        tools: Vec<Tool>,
    ) -> (ConversationStore, Stored) {
        let (text, notes, recognised) = Worker::words(m.body.clone());
        let d = Worker::device(m.id.from);
        let (reply_text, served) = Worker::answer(
            taken.clone(),
            Message {
                holder: d,
                text,
                recognised,
            },
            models,
            tools,
        );
        let (id, to) = Worker::address(taken.clone(), m.clone());
        let reply = Stored {
            id,
            to,
            body: Body::Text(reply_text.text.clone()),
            notes: Worker::voiced(m.body.clone(), reply_text.text),
        };
        let mut all = notes;
        all.extend(served.clone());
        all.push(note(Step::Worker, Note::Answered(id)));
        let noted = taken.annotate_all(m.id, all);
        (
            Worker::tell_owner(noted.post(reply.clone()), d, served),
            reply,
        )
    }

    // mitra: implements Worker::answer
    pub fn answer(
        store: ConversationStore,
        msg: Message,
        models: Models,
        tools: Vec<Tool>,
    ) -> (Reply, Vec<Annotation>) {
        match Confirmations::code(msg.text.clone()) {
            Some(c) => Confirmations::confirm(store, c, msg, tools),
            None => Harness::serve(msg.clone(), store.history(msg.holder), models, tools),
        }
    }

    /// Each held call is told to the owner's phone, with its code (C16).
    // mitra: implements Worker::tell_owner
    pub fn tell_owner(
        store: ConversationStore,
        from: Holder,
        ns: Vec<Annotation>,
    ) -> ConversationStore {
        ns.into_iter().fold(store, |s, a| match a.note {
            Note::Held(x) => {
                let owner = Accounts::owner(s.household);
                let r = Stored {
                    id: s.next_id(),
                    to: Party::Client(owner),
                    body: Body::Text(Reporter::ask_owner(x, from).text),
                    notes: vec![],
                };
                s.post(r)
            }
            _ => s,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phone() -> Holder {
        Holder {
            household: 1,
            device: Device::Phone,
            number: 1,
        }
    }
    fn msg(n: i64, t: &str, notes: Vec<Annotation>) -> Stored {
        Stored {
            id: MessageId {
                from: Party::Client(phone()),
                n,
            },
            to: Party::Assistant,
            body: Body::Text(s(t)),
            notes,
        }
    }

    // mitra: supports Worker::in_order
    #[test]
    fn one_at_a_time_oldest_first() {
        let store = ConversationStore {
            household: 1,
            messages: vec![msg(1, "one", vec![]), msg(2, "two", vec![])],
        };
        mitra::check_eq!(Worker::next(store, phone()), Some(msg(1, "one", vec![])));
        let busy = ConversationStore {
            household: 1,
            messages: vec![
                msg(1, "one", vec![note(Step::Worker, Note::Read)]),
                msg(2, "two", vec![]),
            ],
        };
        mitra::check_eq!(Worker::next(busy, phone()), None);
    }

    #[test]
    fn typed_words_are_kept_as_typed() {
        mitra::check_eq!(
            Worker::words(Body::Text(s("hello"))),
            (s("hello"), vec![], true)
        );
    }

    #[test]
    fn a_voice_not_enrolled_is_not_vouched_for() {
        crate::fake::reset();
        mitra::check_eq!(
            Worker::words(Body::Voice(Audio {
                bytes: s("[unknown] open the door")
            })),
            (
                s("open the door"),
                vec![note(Step::Speech, Note::Transcript(s("open the door")))],
                false
            )
        );
    }

    #[test]
    fn a_cut_short_message_is_answered_as_interrupted() {
        let m = msg(1, "open the door", vec![note(Step::Worker, Note::Read)]);
        let store = ConversationStore {
            household: 1,
            messages: vec![m.clone()],
        };
        let reply = Stored {
            id: MessageId {
                from: Party::Assistant,
                n: 2,
            },
            to: Party::Client(phone()),
            body: Body::Text(s(
                "I was interrupted while working on this. Please ask again.",
            )),
            notes: vec![],
        };
        let answered = m
            .clone()
            .annotate(note(Step::Worker, Note::Answered(reply.id)));
        mitra::check_eq!(
            Worker::resume(store, m),
            (
                ConversationStore {
                    household: 1,
                    messages: vec![answered, reply.clone()]
                },
                reply
            )
        );
    }
}
