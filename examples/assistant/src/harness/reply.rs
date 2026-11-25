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
    pub fn interrupted(begun: Vec<Call>) -> Reply {
        if begun.is_empty() {
            return Reply {
                text: s("I was interrupted while working on this. Please ask again."),
            };
        }
        Reply {
            text: format!(
                "I was interrupted while working on this. These had begun and may have run: {}. Please check before asking again.",
                calls(&begun)
            ),
        }
    }

    // mitra: implements Reporter::begun
    /// The owner is told that calls may have run for another device (H13).
    pub fn begun(cs: Vec<Call>, from: Holder) -> Reply {
        Reply {
            text: format!(
                "A restart cut short a message from {} {}. These had begun and may have run: {}.",
                format!("{:?}", from.device).to_lowercase(),
                from.number,
                calls(&cs)
            ),
        }
    }

    /// The owner's phone is told of a held call (C16).
    // mitra: implements Reporter::ask_owner
    /// The sender's own words, and the call the assistant chose, apart
    /// (K2). The call's input is shown only when it is short and plain:
    /// a long one is the model's prose, which can be an attacker's.
    pub fn ask_owner(h: Hold, from: Holder) -> Reply {
        let input = if h.call.input.len() <= 32
            && h.call
                .input
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || " .,:-".contains(c))
        {
            format!("\"{}\"", h.call.input)
        } else {
            format!("an input of {} characters, not shown", h.call.input.len())
        };
        Reply {
            text: format!(
                "Your {:?} {} sent: \"{}\". To answer it, the assistant wants to run {} with {}. Reply \"yes {}\" to allow it.",
                from.device,
                from.number,
                crate::harness::tools::one_line(&h.asked),
                h.call.tool,
                input,
                h.code
            ),
        }
    }
}
/// Calls named for a person: the tool, and its input when short and
/// plain (as in `ask_owner`).
fn calls(cs: &[Call]) -> String {
    cs.iter()
        .map(|c| {
            if c.input.len() <= 32
                && c.input
                    .chars()
                    .all(|x| x.is_ascii_alphanumeric() || " .,:-".contains(x))
            {
                format!("{} ({})", c.tool, c.input)
            } else {
                c.tool.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
