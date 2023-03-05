use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rev_lines::RevLines;

use crate::model::{Note, Settings};

fn get_notes_file(path: PathBuf) -> File {
    let file = match OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(_) => panic!("Error opening notes file"),
    };
    file
}

pub fn take_note(settings: Settings, note_value: &String, echo: &bool) {
    if settings.verbosity > 0 {
        println!("✏️✏️✏️ Taking note {}", note_value);
    }

    let mut file = get_notes_file(settings.notes_file_path);

    let note = Note::new(note_value.clone());
    if *echo {
        println!("{}", note)
    } else {
        writeln!(file, "{}", note).unwrap();
    }
}

pub fn list_notes(settings: Settings, count: &u8) {
    if settings.verbosity > 0 {
        println!("📓 Printing your last {} notes:", count);
    }

    let file = get_notes_file(settings.notes_file_path);
    let mut reader = RevLines::new(BufReader::new(file)).unwrap();

    for i in 0..*count {
        let val = reader.next().unwrap();
        println!("Note {}: {}", i, val);
    }
}

pub fn search_notes(settings: Settings, search_term: &String, count: &u8) {
    let file = get_notes_file(settings.notes_file_path);
    let reader = RevLines::new(BufReader::new(file)).unwrap();
    let matcher = SkimMatcherV2::default();

    println!("Searching notes with term \"{}\"...", search_term);

    let mut line_matches = Vec::new();
    for line in reader {
        let note = Note::new_from_line(&line);
        let match_res = matcher.fuzzy_match(&note.note_value, search_term);
        match match_res {
            None => {}
            Some(match_score) => line_matches.push((match_score, note)),
        }
    }

    line_matches.sort_by_key(|a| a.0);
    let print_count = min(*count, line_matches.len() as u8);

    if settings.verbosity > 0 {
        println!(
            "Found {} matches, printing max {}.",
            line_matches.len(),
            print_count
        );
    }

    for _ in 0..print_count {
        let val = line_matches.pop().unwrap();
        println!("{} (Score: {})", val.1, val.0)
    }
}

pub fn echo_path(settings: Settings) {
    println!("{}", settings.notes_file_path.display())
}

pub fn echo_under_construction(_settings: Settings) {
    println!("🔨🔨🔨 Currently under construction 🔨🔨🔨")
}
