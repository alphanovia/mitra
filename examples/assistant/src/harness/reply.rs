//! The reply: fixed words for every way it went (see reply.mitra).

use crate::vocabulary::*;

pub struct Reporter;
impl Reporter {
    // mitra: implements Reporter::report
    pub fn report(r: Result<Answer, HarnessFailure>, _a: Ask) -> Reply {
        let text = match r {
            Ok(ans) => ans.text,
            Err(HarnessFailure::EmptyPrompt) => s("I didn't get what you asked. Could you say it again?"),
            Err(HarnessFailure::Llm(LlmFailure::Unreachable)) => {
                s("I can't reach the model that does this work right now. Nothing was done. Please try again later.")
            }
            Err(HarnessFailure::Llm(LlmFailure::TooLong)) => s("That is too long for me to work on. Nothing was done."),
            Err(HarnessFailure::Tool(f)) => format!("A tool failed ({f:?}). Nothing more was done."),
            Err(HarnessFailure::TooManyTurns) => {
                s("I stopped: this took more steps than I am allowed. What was done so far is in the journal.")
            }
        };
        Reply { text }
    }

    // mitra: implements Reporter::decline
    pub fn decline(v: Verdict) -> Reply {
        Reply {
            text: s(match v {
                Verdict::Noise => "I didn't catch that.",
                Verdict::Malicious => "I won't do that.",
                _ => "I won't act on that.",
            }),
        }
    }

    // mitra: implements Reporter::ask_back
    pub fn ask_back(_a: Ask) -> Reply {
        Reply {
            text: s("Which one do you mean?"),
        }
    }

    // mitra: implements Reporter::helped
    pub fn helped(sent: bool) -> Reply {
        Reply {
            text: s(if sent {
                "I have called for help."
            } else {
                "I could not reach anyone. Call 112 now."
            }),
        }
    }

    // mitra: implements Reporter::tuned
    pub fn tuned(_s: Preferences) -> Reply {
        Reply {
            text: s("Thank you, noted."),
        }
    }

    // mitra: implements Reporter::unchecked
    pub fn unchecked(f: LlmFailure) -> Reply {
        Reply {
            text: s(match f {
                LlmFailure::TooLong => {
                    "That message is too long for me to check. Nothing was done."
                }
                LlmFailure::Unreachable => {
                    "I can't check your message right now. If you need help, say help."
                }
            }),
        }
    }

    // mitra: implements Reporter::confirmed
    pub fn confirmed(o: Outcome) -> Reply {
        Reply {
            text: match o.output {
                Ok(out) => format!("Done: {} {}.\n{}", o.call.tool, o.call.input, out),
                Err(f) => format!("{} {} did not run ({f:?}).", o.call.tool, o.call.input),
            },
        }
    }

    // mitra: implements Reporter::not_allowed
    pub fn not_allowed() -> Reply {
        Reply {
            text: s("That code is not valid, or only the owner can allow it."),
        }
    }

    // mitra: implements Reporter::interrupted
    pub fn interrupted() -> Reply {
        Reply {
            text: s("I was interrupted while working on this. Please ask again."),
        }
    }

    /// The owner's phone is told of a held call (C16).
    // mitra: implements Reporter::ask_owner
    pub fn ask_owner(h: Hold, from: Holder) -> Reply {
        Reply {
            text: format!(
                "Your {:?} {} asks to run {} {}. Reply \"yes {}\" to allow it.",
                from.device, from.number, h.call.tool, h.call.input, h.code
            ),
        }
    }
}
