use std::cmp::min;
use std::fs;
use std::fs::{create_dir_all, write, File, OpenOptions};
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rev_lines::RevLines;

use crate::commands::github::{
    get_gh_access_token_oauth, gh_create_scribr_gist, gh_fetch_scribr_gist, gh_pull_gist_files,
    gh_push_gist_files,
};
use crate::internal::{get_default_init_files, get_scribr_home_dir, read_file};
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

pub fn take_note(settings: Settings, note: &str, echo: &bool) {
    let verbosity = settings.verbosity;
    if verbosity > 0 {
        println!("✏️✏️✏️ Taking note {}", note);
    }

    let mut file = get_notes_file(settings.get_default_notebook_path());

    let full_note = Note::new(note);
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

pub fn search_notes(settings: Settings, term: &str, count: &u8) {
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

pub fn init(no_gh: &bool, force: &bool, gist_id: &Option<&str>) {
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

    let files = if !*no_gh {
        println!("Setting up GitHub gist for backup...");
        let access_token = get_gh_access_token_oauth();
        let remote_gist = gh_fetch_scribr_gist(&access_token, gist_id)
            .unwrap_or_else(|| gh_create_scribr_gist(&access_token, get_default_init_files(None)));

        let remote_gist_id = &*remote_gist.id;
        let files = get_default_init_files(Some(remote_gist_id));
        gh_push_gist_files(&access_token, remote_gist_id, files.clone());
        files
    } else {
        get_default_init_files(None)
    };

    for (file_name, file) in files {
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

pub fn backup_notes(run_settings: Settings, include_settings: &bool) {
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
                if file_name == SCRIBR_CONFIG_FILE_NAME && !*include_settings {
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

pub fn restore_notes(run_settings: Settings, force: &bool, include_settings: &bool) {
    let gist_id = run_settings
        .remote
        .and_then(|remote| remote.gist_id)
        .expect("bad result for gist id");
    let access_token = get_gh_access_token_oauth();
    let files = gh_pull_gist_files(&access_token, &gist_id);
    let home_dir = get_scribr_home_dir();

    for (file_name, file_data) in files {
        let full_path = home_dir.join(&file_name);
        if full_path.exists() && !*force {
            println!(
                "Not overwriting file {} as --force was not applied",
                full_path.display()
            );
            continue;
        }
        if !*force && !*include_settings && &file_name == SCRIBR_CONFIG_FILE_NAME {
            println!(
                "not overwriting settings file {}  as --include-settings was not passed",
                full_path.display()
            );
            continue;
        }
        println!("Overwriting file {}", full_path.display());
        write(full_path, file_data.content).unwrap();
    }
}

pub fn echo_under_construction(_settings: Settings) {
    println!("🔨🔨🔨 Currently under construction 🔨🔨🔨")
}
