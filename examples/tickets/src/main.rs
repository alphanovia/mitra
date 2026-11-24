use std::io::Write;
use std::path::Path;
use std::process::exit;

use tickets::{code, render, run, Failure, Outcome};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args, Path::new("tickets.log")) {
        Outcome::Printed(r) => {
            let mut out = std::io::stdout().lock();
            if out.write_all(render(&r).as_bytes()).and_then(|_| out.flush()).is_err() {
                exit(code(Failure::Storage));
            }
        }
        Outcome::Failed(f) => {
            let why = match f {
                Failure::Usage => "usage: tickets add TITLE | done ID | list | show ID",
                Failure::EmptyTitle => "tickets: the title is empty",
                Failure::NoSuchTicket => "tickets: no such ticket",
                Failure::Storage => "tickets: could not read or write tickets.log",
            };
            eprintln!("{}", why);
            exit(code(f));
        }
    }
}
