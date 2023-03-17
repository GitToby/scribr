use std::cmp::min;
use std::fs;
use std::fs::{create_dir_all, write, File, OpenOptions};
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use git2::opts::reset_search_path;
use rev_lines::RevLines;

use crate::commands::github::{
    get_gh_access_token_oauth, gh_create_scribr_gist, gh_fetch_scribr_gist, gh_push_gist_files,
};
use crate::internal::{
    get_default_init_files, get_scribr_config_file, get_scribr_home_dir, read_file,
};
use crate::model::{File as GhFile, GhFiles, Note, Settings, SCRIBR_CONFIG_FILE_NAME};

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
        if let Some(val) = reader.next() {
            println!("Note {}: {}", i, val);
        }
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

pub fn open_path() {
    let home_dir = get_scribr_home_dir();
    match open::that(&home_dir) {
        Ok(_) => {
            println!("Opening {}", home_dir.display())
        }
        Err(_) => {
            println!("Couldnt automatically open {}", home_dir.display())
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

    let mut access_token = String::new();
    let remote_gist_id = if !*no_gh {
        println!("Setting up GitHub gist for backup...");
        access_token = get_gh_access_token_oauth();
        let mut remote_gist = gh_fetch_scribr_gist(&access_token, gist_id);
        if let None = remote_gist {
            remote_gist = gh_create_scribr_gist(&access_token, get_default_init_files(None));
        };
        if let Some(remote_gist) = remote_gist {
            Some(remote_gist.id)
        } else {
            None
        }
    } else {
        None
    };

    let files = get_default_init_files(remote_gist_id.clone());
    let files2 = get_default_init_files(remote_gist_id.clone());

    if let Some(gist_id) = &remote_gist_id {
        // if the gist id exists we should update the settings file with the new gist ID
        gh_push_gist_files(&access_token, gist_id, files);
    };

    for (file_name, file) in files2 {
        let full_path = &scribr_home_dir.join(file_name);
        let err_msg = format!("failed writing file {}", &full_path.display());
        if full_path.exists() {
            println!("overwriting file {}", full_path.display())
        } else {
            println!("Creating file {}", full_path.display());
        }
        write(full_path, &file.content).expect(&*err_msg);
    }
}

pub fn backup_notes(run_settings: Settings, settings: &bool) {
    let gist_id = run_settings
        .remote
        .and_then(|remote| remote.gist_id)
        .expect("bad result for gist id");

    let path = get_scribr_home_dir();
    let mut files = GhFiles::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file = entry.path();
                let file_name = file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("Could not get file name!");
                let content = read_file(&file).expect("could not extract file content");
                if file_name == SCRIBR_CONFIG_FILE_NAME && !*settings {
                    continue;
                }
                files.insert(file_name.to_string(), GhFile::from(content));
            }
        }
    }
    println!("We will back up the following files");
    for f_name in files.keys() {
        println!("{}", f_name)
    }
    let access_token = get_gh_access_token_oauth();
    gh_push_gist_files(&access_token, &gist_id, files);
}

pub fn echo_under_construction(_settings: Settings) {
    println!("🔨🔨🔨 Currently under construction 🔨🔨🔨")
}
