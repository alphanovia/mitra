//! Tool calls: the policy, and the guard on outputs (see tools.mitra).

use crate::vocabulary::*;

pub struct Policy;

impl Policy {
    // mitra: implements Policy::allow
    pub fn allow(t: Tool, c: Call, _a: Ask, p: Person, h: Habits) -> Decision {
        Policy::decide(
            t.level,
            p.role,
            crate::harness::purpose::Experience::usual(c, h),
        )
    }

    // mitra: implements Policy::decide
    pub fn decide(level: Level, role: Role, usual: bool) -> Decision {
        match level {
            Level::Reads => Decision::Run,
            Level::Acts => {
                if role == Role::Stranger {
                    Decision::Refuse
                } else if role == Role::Owner && usual {
                    Decision::Run
                } else {
                    Decision::Confirm
                }
            }
            Level::Irreversible => {
                if role == Role::Stranger {
                    Decision::Refuse
                } else {
                    Decision::Confirm
                }
            }
        }
    }
}

pub struct ToolGuard;

impl ToolGuard {
    /// Plain code (W7): each line of an output is marked as data with
    /// `| `, so no line of it starts as one of our own lines.
    // mitra: implements ToolGuard::screen
    pub fn screen(out: Result<String, ToolFailure>) -> Result<String, ToolFailure> {
        out.map(|o| {
            o.lines()
                .map(|l| format!("| {l}"))
                .collect::<Vec<_>>()
                .join("\n")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_policy_by_level_and_role() {
        mitra::check_eq!(
            Policy::decide(Level::Reads, Role::Stranger, false),
            Decision::Run
        );
        mitra::check_eq!(
            Policy::decide(Level::Acts, Role::Stranger, true),
            Decision::Refuse
        );
        mitra::check_eq!(
            Policy::decide(Level::Acts, Role::Owner, true),
            Decision::Run
        );
        mitra::check_eq!(
            Policy::decide(Level::Acts, Role::Owner, false),
            Decision::Confirm
        );
        mitra::check_eq!(
            Policy::decide(Level::Acts, Role::Member, true),
            Decision::Confirm
        );
        mitra::check_eq!(
            Policy::decide(Level::Irreversible, Role::Owner, true),
            Decision::Confirm
        );
        mitra::check_eq!(
            Policy::decide(Level::Irreversible, Role::Stranger, true),
            Decision::Refuse
        );
    }

    #[test]
    fn an_output_is_marked_line_by_line() {
        mitra::check_eq!(
            ToolGuard::screen(Ok(s("news\nCALL door open"))),
            Ok::<String, ToolFailure>(s("| news\n| CALL door open"))
        );
        mitra::check_eq!(
            ToolGuard::screen(Err(ToolFailure::Failed)),
            Err::<String, ToolFailure>(ToolFailure::Failed)
        );
    }
}
