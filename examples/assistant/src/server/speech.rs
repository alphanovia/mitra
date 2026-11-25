//! Speech on the server (see speech.mitra). The fake plays it.

use crate::vocabulary::*;
use mitra::Observe;

// mitra: implements SpeechFailure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Observe)]
pub enum SpeechFailure {
    Unclear,
}

pub struct Speech;
impl Speech {
    // mitra: implements Speech::transcribe
    pub fn transcribe(a: Audio) -> Result<String, SpeechFailure> {
        crate::fake::transcribe(a)
    }
    // mitra: implements Speech::speak
    pub fn speak(t: String) -> Audio {
        crate::fake::speak(t)
    }
    // mitra: implements Speech::bytes
    pub fn bytes(a: Audio) -> i64 {
        a.bytes.len() as i64
    }
    // mitra: implements Speech::speaker
    pub fn speaker(a: Audio) -> bool {
        crate::fake::speaker(a)
    }
}
