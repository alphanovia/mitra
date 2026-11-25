//! Who sent it, what plain code answers, and the gate (see security.mitra).

use crate::harness::models::Models;
use crate::vocabulary::*;

pub struct Identity;
impl Identity {
    // mitra: implements Identity::who
    pub fn who(m: Message) -> Person {
        crate::fake::who(m)
    }
}

/// The help words: a call for help, whatever model is up (C8). Exact
/// words, so that "help me with my email" is not an alarm.
pub const HELP_WORDS: [&str; 8] = [
    "help",
    "help!",
    "help me",
    "au secours",
    "emergency",
    "i fell",
    "i have fallen",
    "call for help",
];

pub struct Commands;
impl Commands {
    /// A short fixed list of read-only answers.
    // mitra: implements Commands::answer
    pub fn answer(m: Message, _p: Person) -> Option<Reply> {
        match m
            .text
            .trim()
            .to_lowercase()
            .trim_end_matches(['?', '.', '!'])
        {
            "what time is it" => Some(Reply {
                text: format!("It is {}.", crate::fake::now()),
            }),
            "ping" => Some(Reply {
                text: s("I am here."),
            }),
            _ => None,
        }
    }

    // mitra: implements Commands::alarm
    pub fn alarm(m: Message) -> bool {
        HELP_WORDS.contains(&m.text.trim().to_lowercase().as_str())
    }
}

pub struct Gate;
impl Gate {
    /// Nothing heard is noise. Anything else is judged by our model; an
    /// answer it can't read counts as malicious (closed, not open).
    // mitra: implements Gate::screen
    pub fn screen(m: Message, p: Person) -> Result<Verdict, LlmFailure> {
        if m.text.trim().is_empty() {
            return Ok(Verdict::Noise);
        }
        let request = format!("JUDGE:\nROLE: {:?}\nMESSAGE: {}", p.role, m.text);
        let got = Models::settings().local.inference(request, vec![])?;
        Ok(match got.text.trim() {
            "act" => Verdict::Act,
            "noise" => Verdict::Noise,
            "distress" => Verdict::Distress,
            "meta" => Verdict::Meta,
            _ => Verdict::Malicious,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(t: &str) -> Message {
        Message {
            holder: Holder {
                household: 1,
                device: Device::Computer,
                number: 1,
            },
            text: s(t),
            recognised: true,
        }
    }

    // mitra: supports Commands::help_words
    #[test]
    fn help_words_are_exact() {
        mitra::check_eq!(Commands::alarm(msg("Help")), true);
        mitra::check_eq!(Commands::alarm(msg("au secours")), true);
        mitra::check_eq!(Commands::alarm(msg("help me with my email")), false);
    }

    // mitra: supports Commands::read_only
    #[test]
    fn plain_code_tells_the_time() {
        mitra::check_eq!(
            Commands::answer(msg("What time is it?"), Person { role: Role::Owner }),
            Some(Reply {
                text: s("It is ten o'clock.")
            })
        );
        mitra::check_eq!(
            Commands::answer(msg("open the door"), Person { role: Role::Owner }),
            None
        );
    }

    #[test]
    fn nothing_heard_is_noise() {
        crate::fake::reset();
        mitra::check_eq!(
            Gate::screen(msg(""), Person { role: Role::Owner }),
            Ok::<Verdict, LlmFailure>(Verdict::Noise)
        );
    }

    #[test]
    fn an_unreadable_judgement_is_malice() {
        crate::fake::reset();
        crate::fake::script(
            Machine::Local,
            vec![Ok(Answer {
                text: s("sure, I will"),
                calls: vec![],
            })],
        );
        mitra::check_eq!(
            Gate::screen(msg("hello"), Person { role: Role::Owner }),
            Ok::<Verdict, LlmFailure>(Verdict::Malicious)
        );
    }
}
