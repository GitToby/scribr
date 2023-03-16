use std::cmp::min;
use std::fs::{create_dir_all, write, File, OpenOptions};
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rev_lines::RevLines;

use crate::commands::github::{
    get_gh_access_token_oauth, gh_create_scribr_gist, gh_fetch_scribr_gist,
    gh_search_existing_scribr_gist,
};
use crate::internal::{get_default_init_files, get_scribr_home_dir};
use crate::model::{Note, Settings};

mod github;

fn get_notes_file(notes_file: PathBuf) -> File {
    let file = match OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&notes_file)
    {
        Ok(file) => file,
        Err(_) => panic!("Error opening notes file"),
    };
    file
}

pub fn take_note(settings: Settings, note: &String, echo: &bool) {
    let verbosity = settings.verbosity;
    if verbosity > 0 {
        println!("✏️✏️✏️ Taking note {}", note);
    }

    let mut file = get_notes_file(settings.get_default_notebook_path());

    let full_note = Note::new(note.clone());
    if *echo {
        println!("{}", full_note)
    } else {
        writeln!(file, "{}", full_note).unwrap();
    }
}

pub fn list_notes(settings: Settings, count: &u8) {
    if settings.verbosity > 0 {
        println!("📓 Printing your last {} notes:", count);
    }

    let file = get_notes_file(settings.get_default_notebook_path());
    let mut reader = RevLines::new(BufReader::new(file)).unwrap();

    for i in 0..*count {
        let val = reader.next().unwrap();
        println!("Note {}: {}", i, val);
    }
}

pub fn search_notes(settings: Settings, term: &String, count: &u8) {
    let file = get_notes_file(settings.get_default_notebook_path());
    let reader = RevLines::new(BufReader::new(file)).unwrap();
    let matcher = SkimMatcherV2::default();

    println!("Searching notes with term \"{}\"...", term);

    let mut line_matches = Vec::new();
    for line in reader {
        let note = Note::new_from_line(&line);
        let match_res = matcher.fuzzy_match(&note.note_value, term);
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
    match open::that(&settings.get_default_notebook_path()) {
        Ok(_) => {
            println!("Opening {}", settings.get_default_notebook_path().display())
        }
        Err(_) => {
            println!(
                "Couldnt automatically open {}",
                settings.get_default_notebook_path().display()
            )
        }
    };
}

pub fn init(no_gh: &bool, force: &bool, gist_id: Option<&String>) {
    let scribr_home_dir = get_scribr_home_dir();
    if scribr_home_dir.exists() && !*force {
        panic!("Dir already exists at {} - aborting init.", {
            scribr_home_dir.display()
        })
    }

    println!("Creating scribr dir in {}", scribr_home_dir.display());

    let err_msg = format!(
        "Could not create scribr dir at {}",
        scribr_home_dir.display()
    );
    create_dir_all(&scribr_home_dir).expect(&*err_msg);

    // todo: make the borrowing here less difficult...
    let files1 = get_default_init_files();
    let files2 = get_default_init_files();

    if !*no_gh {
        println!("Setting up GitHub gist for backup...");
        let access_token = get_gh_access_token_oauth();
        let gist_result = gh_fetch_scribr_gist(&access_token, gist_id);
        match gist_result {
            Some(gist) => gist,
            None => gh_create_scribr_gist(&access_token, files1)
                .expect("Could not create gist for scribr"),
        };
    };

    for (file_name, file) in files2 {
        let full_path = &scribr_home_dir.join(file_name);
        let err_msg = format!("failed writing file {}", &full_path.display());
        if full_path.exists() {
            println!("overwriting file {}", full_path.display())
        } else {
            println!("Creating file {}", full_path.display());
        }
        write(full_path, file.content).expect(&*err_msg);
    }
}

pub fn backup_notes(_settings: Settings, gist_id: &Option<String>) {
    let access_token = get_gh_access_token_oauth();
    let gist = {
        if let Some(_) = gist_id {
            gh_search_existing_scribr_gist(&access_token)
        } else {
            gh_search_existing_scribr_gist(&access_token)
        }
    };

    dbg!(gist);
}

pub fn echo_under_construction(_settings: Settings) {
    println!("🔨🔨🔨 Currently under construction 🔨🔨🔨")
}
