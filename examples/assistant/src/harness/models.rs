//! The two machines (see models.mitra).

use crate::vocabulary::*;
use mitra::Observe;

/// A model behind inference. The fake plays it.
// mitra: implements Llm
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Llm {
    pub window: i64,
    pub reserve: i64,
    pub machine: Machine,
}

impl Llm {
    // mitra: implements Llm::inference
    pub fn inference(self, request: String, tools: Vec<Tool>) -> Result<Answer, LlmFailure> {
        crate::fake::inference(self, request, tools)
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
