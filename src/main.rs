use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::PathBuf;

use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rev_lines::RevLines;
use scan_fmt::scan_fmt;

// https://docs.rs/clap/4.1.8/clap/_derive/index.html
#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Turn magic commands off while searching or listing notes
    #[arg(short, long)]
    no_magic_commands: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Take a note
    Take {
        /// The note value the note should contain.
        note_value: String,

        /// Echo the note to the console rather than write to disk
        #[arg(short, long)]
        echo: bool,
    },

    /// 📑 List your notes chronologically.
    List {
        /// Number of notes to list
        #[arg(short = 'n', long, default_value = "10")]
        count: u8,
    },

    /// 🔎 Search your notes with fuzzy matching
    Search {
        /// Term to search for across all notes.
        term: String,

        /// Number of notes to list.
        #[arg(short = 'n', long, default_value = "10")]
        count: u8,
    },

    /// 📁 Echo the notes file path
    Path,

    /// ☁️ Back up notes to a GitHub gist
    Backup,

    /// ☁️ Restore your notes file from a GitHub gist
    Restore {
        /// Force overwriting your local file with the remote file
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Debug)]
struct Settings {
    notes_file_path: PathBuf,
    verbosity: u8,
    no_magic_commands: bool,
}

impl Settings {
    fn print_to_console(&self) {
        if self.verbosity > 0 {
            println!("Running in verbose level {}.", self.verbosity);
            println!("Using note file at {}", self.notes_file_path.display());
            if self.no_magic_commands {
                println!("Ignoring magic commands.");
            }
        }
        if self.verbosity > 1 {
            println!("Full settings: {:?}", self);
        }
    }
}

struct Note {
    timestamp: DateTime<Local>,
    note_value: String,
}

impl Note {
    fn new(note_value: String) -> Note {
        return Note {
            timestamp: Local::now(),
            note_value,
        };
    }

    fn new_from_line(line: &String) -> Note {
        // This must match the fmt below, as the parse may fail.
        let fmt_res = scan_fmt!(line, "{} - {}", String, String);

        // todo: make this destructuring more solid
        let (timestamp_str, note_value) =
            fmt_res.expect("Format of notes should be as \"{timestamp} - {note}\"");
        let timestamp = DateTime::parse_from_rfc2822(&*timestamp_str)
            .expect("All Datetime should be in the rfc2822 format")
            .with_timezone(&Local);

        return Note {
            timestamp,
            note_value,
        };
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.timestamp.to_rfc2822(), self.note_value)
    }
}

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

fn take_note(settings: Settings, note_value: &String, echo: &bool) {
    if settings.verbosity > 0 {
        println!("✏️ ✏️ ✏️ Taking note {}", note_value);
    }

    let mut file = get_notes_file(settings.notes_file_path);

    let note = Note::new(note_value.clone());
    if *echo {
        println!("{}", note)
    } else {
        writeln!(file, "{}", note).unwrap();
    }
}

fn list_notes(settings: Settings, count: &u8) {
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

fn search_notes(settings: Settings, search_term: &String, count: &u8) {
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

fn echo_path(settings: Settings) {
    println!("{}", settings.notes_file_path.display())
}

fn main() {
    let cli = Cli::parse();

    let note_file = match cli.file {
        Some(notes_file) => notes_file,
        None => home_dir()
            .map(|p| p.join(".notes.txt"))
            .expect("No home dir found!!"),
    };

    let settings = Settings {
        notes_file_path: note_file,
        verbosity: cli.verbose,
        no_magic_commands: cli.no_magic_commands,
    };

    settings.print_to_console();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Take { note_value, echo }) => take_note(settings, note_value, echo),
        Some(Commands::List { count }) => list_notes(settings, count),
        Some(Commands::Search { term, count }) => search_notes(settings, term, count),
        Some(Commands::Path {}) => echo_path(settings),
        Some(Commands::Backup {}) => echo_under_construction(settings),
        Some(Commands::Restore { force }) => echo_under_construction(settings),
        _ => {}
    }
}

fn echo_under_construction(_settings: Settings) {
    println!("🔨🔨🔨 Currently under construction 🔨🔨🔨")
}
