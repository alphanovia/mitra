//! The assistant's CLI: `cargo run --bin assistant-cli -- --token cli "hello"`.
//! With no message, it reads lines until the end of input.
//! `--voice` sends the words as a voice message (the fake speech reads
//! them back), `--addr` names the server (127.0.0.1:7878).

use assistant::cli::Cli;
use assistant::vocabulary::*;
use std::io::BufRead;
use std::time::{Duration, Instant};

fn main() {
    let mut addr = "127.0.0.1:7878".to_string();
    let mut token = "cli".to_string();
    let mut voice = false;
    let mut words = vec![];
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--addr" => addr = args.next().unwrap_or(addr),
            "--token" => token = args.next().unwrap_or(token),
            "--voice" => voice = true,
            // Where one of this device's messages stands (W12).
            "--status" => {
                let n = args.next().unwrap_or_default();
                let got = assistant::cli::ask(&addr, &format!("STATUS {token} {n}"));
                match got {
                    Ok(l) => println!("{}", l.join(" ")),
                    Err(f) => eprintln!("{} ({f:?})", Cli::status(false)),
                }
                return;
            }
            _ => words.push(a),
        }
    }
    let mut cli = match Cli::connect(&addr, &token) {
        Ok(c) => c,
        Err(f) => {
            eprintln!("{}", Cli::status(false));
            eprintln!("({f:?})");
            std::process::exit(1);
        }
    };
    if words.is_empty() {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            cli = say(&addr, cli, &line, voice);
        }
    } else {
        say(&addr, cli, &words.join(" "), voice);
    }
}

fn say(addr: &str, cli: Cli, text: &str, voice: bool) -> Cli {
    let input = if voice {
        Input::Spoken(Audio {
            bytes: text.to_string(),
        })
    } else {
        Input::Typed(text.to_string())
    };
    let e = cli.envelope(input);
    let r = cli.send(addr, e);
    let mut cli = cli.made();
    println!("{}", Cli::status(Cli::sent(r)));
    if r.is_err() {
        return cli;
    }
    let start = Instant::now();
    let mut shown_working = false;
    while start.elapsed() < Duration::from_secs(10) {
        if let Ok(new) = cli.poll(addr) {
            if !new.is_empty() {
                for (n, t) in &new {
                    println!("assistant: {t}");
                    cli.got(addr, *n);
                    cli.seen = cli.seen.max(*n);
                }
                return cli;
            }
        }
        if !shown_working && cli.working(addr) {
            println!("working…");
            shown_working = true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    println!("no reply yet");
    cli
}
