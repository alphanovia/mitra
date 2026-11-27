//! The two machines (see models.mitra).

use crate::vocabulary::*;
use mitra::Observe;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A model behind inference.
// mitra: implements Llm
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Llm {
    pub window: i64,
    pub reserve: i64,
    pub machine: Machine,
}

/// What answers a machine's requests: a model server (`ollama`), or the
/// fake world (`fake::Fake`), which tests script. The window is not the
/// backend's to keep: `Llm::inference` keeps it for all of them.
pub trait Backend {
    fn answer(&self, llm: &Llm, request: &str, tools: &[Tool]) -> Result<Answer, LlmFailure>;
}

thread_local! {
    /// The backend of each machine, in this thread: the server sets its
    /// worker's at start; each test sets its own. None: the fake.
    static SERVED: RefCell<HashMap<Machine, Rc<dyn Backend>>> = RefCell::new(HashMap::new());
}

/// From now on, in this thread, `b` answers machine `m`.
pub fn serve(m: Machine, b: impl Backend + 'static) {
    SERVED.with(|s| s.borrow_mut().insert(m, Rc::new(b)));
}

/// Every machine back to the fake, in this thread.
pub fn unserve_all() {
    SERVED.with(|s| s.borrow_mut().clear());
}

fn backend(m: Machine) -> Rc<dyn Backend> {
    SERVED
        .with(|s| s.borrow().get(&m).cloned())
        .unwrap_or_else(|| Rc::new(crate::fake::Fake))
}

impl Llm {
    /// A request that leaves less than the reserve is refused here, before
    /// any backend (`windowed`); else the machine's backend answers. Each
    /// answer is a measurement for `mitra explain`.
    // mitra: implements Llm::inference
    pub fn inference(self, request: String, tools: Vec<Tool>) -> Result<Answer, LlmFailure> {
        let got = if request.len() as i64 + self.reserve > self.window {
            Err(LlmFailure::TooLong)
        } else {
            backend(self.machine).answer(&self, &request, &tools)
        };
        crate::fake::gave!(Llm::inference(self, request, tools), got)
    }
}

// mitra: implements Models
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Models {
    pub cloud: Llm,
    pub local: Llm,
    pub turns: i64,
}

/// Settings. Bytes stand for tokens. Our model's window holds a message
/// of the size limit with room to spare (H15).
pub const CLOUD_WINDOW: i64 = 100_000;
pub const CLOUD_RESERVE: i64 = 4_000;
pub const LOCAL_WINDOW: i64 = 8_000;
pub const LOCAL_RESERVE: i64 = 500;
pub const TURNS: i64 = 8;

impl Models {
    pub fn settings() -> Models {
        Models {
            cloud: Llm {
                window: CLOUD_WINDOW,
                reserve: CLOUD_RESERVE,
                machine: Machine::Cloud,
            },
            local: Llm {
                window: LOCAL_WINDOW,
                reserve: LOCAL_RESERVE,
                machine: Machine::Local,
            },
            turns: TURNS,
        }
    }

    // mitra: implements Models::machine
    pub fn machine(easy: bool) -> Machine {
        if easy {
            Machine::Local
        } else {
            Machine::Cloud
        }
    }

    // mitra: implements Models::llm
    pub fn llm(self, m: Machine) -> Llm {
        match m {
            Machine::Local => self.local,
            Machine::Cloud => self.cloud,
        }
    }

    // mitra: implements Models::pick
    pub fn pick(self, a: Ask) -> Llm {
        self.llm(Models::machine(a.easy))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_easy_ask_stays_local() {
        mitra::check_eq!(Models::machine(true), Machine::Local);
        mitra::check_eq!(Models::machine(false), Machine::Cloud);
    }
}
