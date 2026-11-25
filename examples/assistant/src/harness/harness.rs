//! One message through the pipeline, and the agent loop (see harness.mitra).

use crate::harness::care::{Helpline, Tuning};
use crate::harness::conversation::Conversation;
use crate::harness::models::{Llm, Models};
use crate::harness::purpose::{Composer, Experience, Purpose};
use crate::harness::reply::Reporter;
use crate::harness::security::{Commands, Gate, Identity};
use crate::harness::tools::{Policy, ToolGuard};
use crate::vocabulary::*;

pub struct Harness;

fn note(by: Step, note: Note) -> Annotation {
    Annotation { by, note }
}

impl Harness {
    // mitra: implements Harness::serve
    pub fn serve(
        m: Message,
        history: Conversation,
        models: Models,
        tools: Vec<Tool>,
    ) -> (Reply, Vec<Annotation>) {
        let p = Identity::who(m.clone());
        let seen = vec![note(Step::Identity, Note::Sender(p))];
        if Commands::alarm(m.clone()) {
            let mut ns = seen;
            ns.push(note(Step::Commands, Note::Verdict(Verdict::Distress)));
            return (Reporter::helped(Helpline::alert(p, m)), ns);
        }
        if let Some(r) = Commands::answer(m.clone(), p) {
            return (r, seen);
        }
        let v = match Gate::screen(m.clone(), p) {
            Err(f) => return (Reporter::unchecked(f), seen),
            Ok(v) => v,
        };
        let mut judged = seen;
        judged.push(note(Step::Gate, Note::Verdict(v)));
        match Harness::after_gate(v, p.role) {
            Route::Work => match Purpose::pin(m, p, history.clone()) {
                Err(f) => (Reporter::unchecked(f), judged),
                Ok(a) => {
                    let mut asked = judged;
                    asked.push(note(Step::Purpose, Note::Ask(a.clone())));
                    match Harness::after_ask(a.clear) {
                        Route::Work => {
                            let h = Experience::recall(a.clone(), p);
                            let (result, c) = Harness::run(
                                models.clone().pick(a.clone()),
                                models.turns,
                                a.clone(),
                                p,
                                h,
                                history,
                                tools,
                            );
                            asked.extend(Harness::called(c.outcomes(), a.clone()));
                            (Reporter::report(result, a), asked)
                        }
                        _ => (Reporter::ask_back(a), asked),
                    }
                }
            },
            Route::Help => (Reporter::helped(Helpline::alert(p, m)), judged),
            Route::Tune => (Reporter::tuned(Tuning::adjust(m, p)), judged),
            Route::Decline | Route::AskBack => (Reporter::decline(v), judged),
        }
    }

    // mitra: implements Harness::after_gate
    pub fn after_gate(v: Verdict, role: Role) -> Route {
        match v {
            Verdict::Act => Route::Work,
            Verdict::Distress => Route::Help,
            Verdict::Meta => {
                if role == Role::Stranger {
                    Route::Decline
                } else {
                    Route::Tune
                }
            }
            Verdict::Noise | Verdict::Malicious => Route::Decline,
        }
    }

    // mitra: implements Harness::after_ask
    pub fn after_ask(clear: bool) -> Route {
        if clear {
            Route::Work
        } else {
            Route::AskBack
        }
    }

    // mitra: implements Harness::called
    pub fn called(os: Vec<Outcome>, a: Ask) -> Vec<Annotation> {
        os.into_iter()
            .flat_map(|o| {
                let mut ns = vec![note(Step::Policy, Note::Called(o.clone()))];
                ns.extend(Harness::held(o, a.clone()));
                ns
            })
            .collect()
    }

    // mitra: implements Harness::held
    pub fn held(o: Outcome, a: Ask) -> Vec<Annotation> {
        if Harness::awaits(o.clone()) {
            vec![note(
                Step::Policy,
                Note::Held(Hold {
                    code: crate::fake::fresh(),
                    call: o.call,
                    asked: a.text,
                }),
            )]
        } else {
            vec![]
        }
    }

    // mitra: implements Harness::awaits
    pub fn awaits(o: Outcome) -> bool {
        o.output == Err(ToolFailure::AwaitsYes)
    }

    /// The agent loop on one model, at most `turns` answers that still
    /// call a tool (H12).
    // mitra: implements Harness::run
    pub fn run(
        llm: Llm,
        turns: i64,
        a: Ask,
        p: Person,
        h: Habits,
        history: Conversation,
        tools: Vec<Tool>,
    ) -> (Result<Answer, HarnessFailure>, Conversation) {
        if a.text.is_empty() {
            return (Err(HarnessFailure::EmptyPrompt), history);
        }
        let mut c = Conversation {
            turns: history.turns,
        };
        c.turns.push(Turn::User(a.text.clone()));
        let ask = |c: &Conversation| {
            Composer::compose(
                a.clone(),
                Purpose::context(a.clone(), c.clone()),
                h.clone(),
                llm.clone(),
            )
        };
        let mut got = llm.clone().inference(ask(&c), tools.clone());
        let mut k = 1;
        while !Harness::finished(got.clone()) && k < turns {
            let ans = got.clone().expect("not finished, so an answer");
            let after_output = Harness::read_output(c.clone());
            let rs = Harness::perform_all(
                ans.calls.clone(),
                a.clone(),
                p,
                h.clone(),
                tools.clone(),
                after_output,
            );
            c.turns.push(Turn::Model(ans));
            c.turns.push(Turn::Results(rs));
            got = llm.clone().inference(ask(&c), tools.clone());
            k += 1;
        }
        let result = if Harness::finished(got.clone()) {
            Harness::lifted(got)
        } else {
            Err(HarnessFailure::TooManyTurns)
        };
        (result, c)
    }

    // mitra: implements Harness::lifted
    pub fn lifted(got: Result<Answer, LlmFailure>) -> Result<Answer, HarnessFailure> {
        got.map_err(HarnessFailure::Llm)
    }

    // mitra: implements Harness::finished
    pub fn finished(got: Result<Answer, LlmFailure>) -> bool {
        match got {
            Ok(ans) => ans.calls.is_empty(),
            Err(_) => true,
        }
    }

    // mitra: implements Harness::perform_all
    pub fn perform_all(
        calls: Vec<Call>,
        a: Ask,
        p: Person,
        h: Habits,
        tools: Vec<Tool>,
        after_output: bool,
    ) -> Vec<Outcome> {
        calls
            .into_iter()
            .map(|c| {
                let output = match Harness::lookup(c.tool.clone(), tools.clone()) {
                    None => Harness::unknown(),
                    Some(t) => {
                        let d = Policy::allow(
                            t.clone(),
                            c.clone(),
                            a.clone(),
                            p,
                            h.clone(),
                            after_output,
                        );
                        if Harness::runs(d) {
                            ToolGuard::screen(Harness::act(t, c.clone()))
                        } else {
                            Err(Harness::withheld(d))
                        }
                    }
                };
                Outcome { call: c, output }
            })
            .collect()
    }

    // mitra: implements Harness::act
    /// A tool runs only once its call is written down (H13).
    pub fn act(t: Tool, c: Call) -> Result<String, ToolFailure> {
        crate::server::store::Ahead::mark(c.clone());
        t.call(c.input)
    }

    // mitra: implements Harness::read_output
    pub fn read_output(c: Conversation) -> bool {
        c.outcomes().iter().any(|o| o.output.is_ok())
    }

    // mitra: implements Harness::unknown
    pub fn unknown() -> Result<String, ToolFailure> {
        Err(ToolFailure::Unknown)
    }

    // mitra: implements Harness::lookup
    pub fn lookup(name: String, tools: Vec<Tool>) -> Option<Tool> {
        tools.into_iter().rev().find(|t| t.name == name)
    }

    // mitra: implements Harness::runs
    pub fn runs(d: Decision) -> bool {
        d == Decision::Run
    }

    // mitra: implements Harness::withheld
    pub fn withheld(d: Decision) -> ToolFailure {
        match d {
            Decision::Confirm => ToolFailure::AwaitsYes,
            _ => ToolFailure::Refused,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_after_the_gate() {
        mitra::check_eq!(Harness::after_gate(Verdict::Act, Role::Owner), Route::Work);
        mitra::check_eq!(
            Harness::after_gate(Verdict::Malicious, Role::Owner),
            Route::Decline
        );
        mitra::check_eq!(
            Harness::after_gate(Verdict::Meta, Role::Stranger),
            Route::Decline
        );
        mitra::check_eq!(
            Harness::after_gate(Verdict::Meta, Role::Member),
            Route::Tune
        );
        mitra::check_eq!(
            Harness::after_gate(Verdict::Distress, Role::Stranger),
            Route::Help
        );
    }

    #[test]
    fn a_tool_is_looked_up_by_name() {
        let door = Tool {
            name: s("door"),
            description: s("opens the front door"),
            level: Level::Irreversible,
        };
        mitra::check_eq!(Harness::lookup(s("door"), vec![door.clone()]), Some(door));
        mitra::check_eq!(Harness::lookup(s("door"), vec![]), None);
    }

    #[test]
    fn the_loop_stops_on_a_final_answer_or_a_failure() {
        mitra::check_eq!(
            Harness::finished(Ok(Answer {
                text: s("hi"),
                calls: vec![]
            })),
            true
        );
        mitra::check_eq!(Harness::finished(Err(LlmFailure::Unreachable)), true);
    }

    #[test]
    fn a_held_call_gets_a_code() {
        crate::fake::reset();
        crate::fake::codes(vec![424242]);
        let door = Call {
            tool: s("door"),
            input: s("open"),
        };
        mitra::check_eq!(
            Harness::held(
                Outcome {
                    call: door.clone(),
                    output: Err(ToolFailure::AwaitsYes)
                },
                Ask {
                    text: s("open the door"),
                    clear: true,
                    easy: false
                }
            ),
            vec![Annotation {
                by: Step::Policy,
                note: Note::Held(Hold {
                    code: 424242,
                    call: door,
                    asked: s("open the door")
                })
            }]
        );
    }
}
