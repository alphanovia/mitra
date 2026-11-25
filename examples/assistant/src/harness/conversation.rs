//! The conversation kept with the model (see conversation.mitra).

use crate::vocabulary::*;
use mitra::Observe;

// mitra: implements Conversation
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Conversation {
    pub turns: Vec<Turn>,
}

impl Conversation {
    /// The naive request: every turn written out, one line each.
    // mitra: implements Conversation::encode
    pub fn encode(self) -> String {
        self.turns.iter().map(|t| format!("{t:?}\n")).collect()
    }

    // mitra: implements Conversation::texts
    pub fn texts(self) -> i64 {
        self.turns
            .iter()
            .map(|t| match t {
                Turn::User(p) => p.len() as i64,
                Turn::Model(a) => a.text.len() as i64,
                Turn::Results(rs) => rs
                    .iter()
                    .map(|o| o.output.as_ref().map(|s| s.len() as i64).unwrap_or(0))
                    .sum(),
            })
            .sum()
    }

    // mitra: implements Conversation::outcomes
    pub fn outcomes(self) -> Vec<Outcome> {
        self.turns
            .into_iter()
            .flat_map(|t| match t {
                Turn::Results(rs) => rs,
                _ => vec![],
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_count_bytes() {
        mitra::check_eq!(
            Conversation::texts(Conversation {
                turns: vec![Turn::User(s("hello"))]
            }),
            5
        );
        mitra::check_eq!(
            Conversation::texts(Conversation {
                turns: vec![
                    Turn::User(s("hi")),
                    Turn::Model(Answer {
                        text: s("ok"),
                        calls: vec![]
                    }),
                    Turn::Results(vec![])
                ]
            }),
            4
        );
    }

    #[test]
    fn the_naive_request_holds_every_text() {
        let c = Conversation {
            turns: vec![
                Turn::User(s("hello")),
                Turn::Model(Answer {
                    text: s("hi"),
                    calls: vec![],
                }),
            ],
        };
        mitra::check_eq!(
            Conversation::encode(c),
            s("User(\"hello\")\nModel(Answer { text: \"hi\", calls: [] })\n")
        );
    }
}
