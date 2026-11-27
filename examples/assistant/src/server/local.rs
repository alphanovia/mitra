//! The server in one process: one store per household, the entry's
//! calls, and the worker run after each post. The binary serves it over
//! HTTP; tests drive it directly.

use crate::harness::models::Models;
use crate::server::gateway::{Accounts, Gateway};
use crate::server::store::{ConversationStore, Stored};
use crate::server::worker::Worker;
use crate::vocabulary::*;
use std::collections::BTreeMap;

pub struct Local {
    pub stores: BTreeMap<i64, ConversationStore>,
    pub models: Models,
    pub tools: Vec<Tool>,
}

pub fn tool(name: &str, description: &str, level: Level) -> Tool {
    Tool {
        name: s(name),
        description: s(description),
        level,
    }
}

/// The tools of tonight's assistant. All fake: they only note that they ran.
pub fn tools() -> Vec<Tool> {
    vec![
        tool("web", "reads the news and the web", Level::Reads),
        tool("lights", "turns the lights on or off", Level::Acts),
        tool("door", "opens the front door", Level::Irreversible),
        tool("bank", "sends money", Level::Irreversible),
    ]
}

pub fn holder(household: i64, device: Device, number: i64) -> Holder {
    Holder {
        household,
        device,
        number,
    }
}

impl Local {
    /// Two households. In the first, the owner's phone, a computer with
    /// the CLI, and a wearable; in the second, one phone.
    pub fn new() -> Local {
        Local::pair();
        let mut stores = BTreeMap::new();
        stores.insert(1, ConversationStore::new(1));
        stores.insert(2, ConversationStore::new(2));
        Local {
            stores,
            models: Models::settings(),
            tools: tools(),
        }
    }

    /// The devices, paired in this thread's world (the accounts are the
    /// fake's, one world per thread): each thread that serves calls on
    /// this server pairs them.
    pub fn pair() {
        crate::fake::pair("owner-phone", holder(1, Device::Phone, 1), Role::Owner);
        crate::fake::pair("cli", holder(1, Device::Computer, 1), Role::Member);
        crate::fake::pair("wearable", holder(1, Device::Wearable, 1), Role::Member);
        crate::fake::pair("other-phone", holder(2, Device::Phone, 1), Role::Owner);
    }

    fn token(t: &str) -> Token {
        Token { secret: s(t) }
    }

    fn household(t: &str) -> Option<i64> {
        Accounts::holder(Local::token(t)).map(|h| h.household)
    }

    /// POST: refused unless the token names a household; else the entry
    /// decides, and the worker runs.
    pub fn post(&mut self, t: &str, e: Envelope) -> Result<Receipt, NetFailure> {
        let receipt = self.accept(t, e);
        if receipt.is_ok() {
            self.work();
        }
        receipt
    }

    /// The entry alone: the message is recorded (or refused), and the
    /// worker has not run yet. The HTTP server answers the sender first,
    /// then calls `work`.
    pub fn accept(&mut self, t: &str, e: Envelope) -> Result<Receipt, NetFailure> {
        let Some(h) = Local::household(t) else {
            return Err(NetFailure::Refused);
        };
        // Taken out, not copied: a post costs no copy of the store (K21).
        let store = self
            .stores
            .remove(&h)
            .unwrap_or_else(|| ConversationStore::new(h));
        let (store, receipt) = Gateway::post(store, Local::token(t), e);
        self.stores.insert(h, store);
        receipt
    }

    /// Posts a text with the device's next number.
    pub fn say(&mut self, t: &str, text: &str) -> Result<Receipt, NetFailure> {
        let e = self.next_envelope(t, Body::Text(s(text)));
        self.post(t, e)
    }
    pub fn speak(&mut self, t: &str, audio: &str) -> Result<Receipt, NetFailure> {
        let e = self.next_envelope(t, Body::Voice(Audio { bytes: s(audio) }));
        self.post(t, e)
    }

    pub fn next_envelope(&self, t: &str, body: Body) -> Envelope {
        let h = Accounts::holder(Local::token(t)).unwrap_or(holder(0, Device::Computer, 0));
        Envelope {
            id: MessageId {
                from: Party::Client(h),
                n: self.count(t) + 1,
            },
            body,
        }
    }

    /// The highest number this device has used.
    pub fn count(&self, t: &str) -> i64 {
        let Some(h) = Accounts::holder(Local::token(t)) else {
            return 0;
        };
        let Some(store) = self.stores.get(&h.household) else {
            return 0;
        };
        store
            .messages
            .iter()
            .filter(|m| m.id.from == Party::Client(h))
            .map(|m| m.id.n)
            .max()
            .unwrap_or(0)
    }

    /// The highest number of a reply this device said it got: a client
    /// that starts again shows what it has not shown yet.
    pub fn last_delivered(&self, t: &str) -> i64 {
        self.fetch(t, 0)
            .iter()
            .filter(|m| m.notes.iter().any(|a| a.note == Note::Delivered))
            .map(|m| m.id.n)
            .max()
            .unwrap_or(0)
    }

    pub fn fetch(&self, t: &str, after: i64) -> Vec<Stored> {
        match Local::household(t).and_then(|h| self.stores.get(&h)) {
            Some(store) => Gateway::fetch(store.clone(), Local::token(t), after),
            None => vec![],
        }
    }

    /// The texts of the replies to this device, oldest first.
    pub fn replies(&self, t: &str) -> Vec<String> {
        self.fetch(t, 0)
            .into_iter()
            .map(|m| match m.body {
                Body::Text(x) => x,
                Body::Voice(_) => s(""),
            })
            .collect()
    }
    pub fn last_text(&self, t: &str) -> String {
        self.replies(t).pop().unwrap_or_default()
    }

    pub fn typing(&self, t: &str) -> bool {
        match Local::household(t).and_then(|h| self.stores.get(&h)) {
            Some(store) => Gateway::typing_for(store.clone(), Local::token(t)),
            None => false,
        }
    }

    /// Where this device's message `n` stands (W12).
    pub fn receipt(&self, t: &str, n: i64) -> Option<Receipt> {
        let store = self.stores.get(&Local::household(t)?)?;
        Gateway::receipt(store.clone(), Local::token(t), n)
    }

    pub fn ack(&mut self, t: &str, id: MessageId) {
        if let Some(h) = Local::household(t) {
            if let Some(store) = self.stores.get(&h).cloned() {
                self.stores
                    .insert(h, Gateway::delivered(store, Local::token(t), id));
            }
        }
    }

    /// The worker: each device's messages, one at a time, oldest first.
    /// Taking a message up is its own store write, before any model (W1).
    pub fn work(&mut self) {
        let households: Vec<i64> = self.stores.keys().copied().collect();
        for h in households {
            while let Some(t) = self.take_next(h) {
                let done = Local::process(&t, self.models.clone(), self.tools.clone());
                self.finish(t, done);
            }
        }
    }

    /// The next message of a household, taken up (its own write), if any.
    pub fn take_next(&mut self, h: i64) -> Option<Taken> {
        let store = self.stores.get(&h)?.clone();
        let m = store
            .holders()
            .into_iter()
            .find_map(|d| Worker::next(store.clone(), d))?;
        let taken = Worker::take_up(store, m.clone());
        self.stores.insert(h, taken.clone());
        Some(Taken {
            household: h,
            store: taken,
            message: m,
        })
    }

    /// The work on a message taken up, off the server: it reads and gives
    /// back the store as it was taken, so it may run while calls go on.
    pub fn process(t: &Taken, models: Models, tools: Vec<Tool>) -> ConversationStore {
        Worker::process(t.store.clone(), t.message.clone(), models, tools).0
    }

    /// The work's writes, onto the store as it is now.
    pub fn finish(&mut self, t: Taken, done: ConversationStore) {
        let now = self
            .stores
            .remove(&t.household)
            .unwrap_or_else(|| ConversationStore::new(t.household));
        self.stores.insert(t.household, merged(now, &t.store, done));
    }

    /// A crash right after a message was taken up: it is written as read,
    /// and nothing else happens.
    pub fn take_up_only(&mut self, t: &str, e: Envelope) {
        let Some(h) = Local::household(t) else { return };
        let (store, receipt) = Gateway::post(self.stores[&h].clone(), Local::token(t), e.clone());
        if receipt.is_ok() {
            let m = store
                .messages
                .iter()
                .find(|m| m.id == e.id)
                .cloned()
                .expect("stored");
            self.stores.insert(h, Worker::take_up(store, m));
        }
    }

    /// A restart: each message cut short is answered as interrupted.
    pub fn restart(&mut self) {
        let households: Vec<i64> = self.stores.keys().copied().collect();
        for h in households {
            let store = self.stores[&h].clone();
            for d in store.holders() {
                let store = self.stores[&h].clone();
                if let Some(m) = Worker::cut_short(store.clone(), d) {
                    let (done, _) = Worker::resume(store, m);
                    self.stores.insert(h, done);
                }
            }
        }
        self.work();
    }

    /// The notes on this device's last message: what each step decided.
    pub fn notes_of_last(&self, t: &str) -> Vec<Annotation> {
        let Some(h) = Accounts::holder(Local::token(t)) else {
            return vec![];
        };
        self.stores[&h.household]
            .messages
            .iter()
            .rfind(|m| m.id.from == Party::Client(h))
            .map(|m| m.notes.clone())
            .unwrap_or_default()
    }

    /// The code of the last call held in a household, if any.
    pub fn last_code(&self, household: i64) -> Option<i64> {
        self.stores[&household]
            .messages
            .iter()
            .flat_map(|m| m.notes.iter())
            .filter_map(|a| match &a.note {
                Note::Held(x) => Some(x.code),
                _ => None,
            })
            .next_back()
    }
}

/// A message taken up, with the store as it was then.
pub struct Taken {
    pub household: i64,
    pub store: ConversationStore,
    pub message: Stored,
}

/// What the work wrote onto `taken`, written onto `now`. The store is only
/// ever added to (`Stored::append_only`): the work adds notes to messages
/// it was given and new messages (its replies); meanwhile the entry adds
/// new messages and notes. Each keeps what the other added. The work's
/// replies are numbered from `taken`, and no one else numbers the
/// assistant's messages, so the numbers stay distinct.
pub fn merged(
    now: ConversationStore,
    taken: &ConversationStore,
    done: ConversationStore,
) -> ConversationStore {
    let mut out = now;
    for d in done.messages {
        match taken.messages.iter().find(|t| t.id == d.id) {
            Some(before) => {
                let added = d.notes.get(before.notes.len()..).unwrap_or(&[]).to_vec();
                if let Some(m) = out.messages.iter_mut().find(|m| m.id == d.id) {
                    m.notes.extend(added);
                }
            }
            None => out.messages.push(d),
        }
    }
    out
}

impl Default for Local {
    fn default() -> Self {
        Local::new()
    }
}
