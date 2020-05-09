use chrono::{Local, DateTime};
use std::io::{stdin, Write, stdout};
use serde::{Deserialize, Serialize};
use ansi_term::Colour::{Blue, Yellow};

#[derive(Debug, Deserialize, Serialize)]
pub struct Note {
    pub body: String,
    pub record_time: DateTime<Local>,
}

impl Note {
    pub fn new(body: String, record_time: DateTime<Local>) -> Self {
        Note { body, record_time }
    }

    /// Loops round prompting the user for a note. Once they're happy it will return the note object
    pub fn from_prompt() -> Self {
        loop {
            let time = Local::now();
            println!("{} | what did you do today?", time.format("%a %d %b %Y"));
            let mut input = String::new();
            let _ = stdout().flush();
            stdin().read_line(&mut input).expect("Did not enter a correct string");
            if let Some('\n') = input.chars().next_back() {
                input.pop();
            }
            if let Some('\r') = input.chars().next_back() {
                input.pop();
            }
            if fetch_user_confirm("Persist this?") {
                return Note::new(
                    input,
                    time,
                );
            }
        }
    }

    pub fn from_string(body: String) -> Self {
        return Note::new(body, Local::now());
    }

    pub fn get_display_format(&self) -> String {
        format!("Note for {}:\n{}", self.record_time.format("%a %d %b %Y at %H:%M"), self.body)
    }
}

fn fetch_user_confirm(prompt: &str) -> bool {
    let mut s = String::new();
    while !["y", "n"].contains(&s.trim()) {
        println!("{} (y/n)", Blue.bold().paint(prompt));
        stdin().read_line(&mut s).expect("Did not enter a correct string");
    }
    return match s.trim() {
        "y" | "yes" => true,
        _ => false,
    };
}