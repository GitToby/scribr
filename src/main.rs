use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::Local;
use clap::{Parser, Subcommand};
use dirs::home_dir;
use rev_lines::RevLines;

// https://docs.rs/clap/4.1.8/clap/_derive/index.html
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    notes_file: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Take a note
    Take {
        /// lists test values
        note_value: String,
    },

    /// List your notes. This will list notes chronologically from
    List {
        // Number of notes to list
        #[arg(short = 'n', long, default_value = "10")]
        count: u8,
    },

    /// Search
    Search {
        /// Term to search for.
        term: String,

        // Number of notes to list.
        #[arg(short = 'n', long, default_value = "10")]
        count: u8,
    },
}

#[derive(Debug)]
struct Settings {
    notes_file_path: PathBuf,
    verbosity: u8,
}

impl Settings {
    fn print_to_console(&self) {
        match self.verbosity {
            // warnings only
            0 => {}
            // info level
            1 => {
                println!("Using note file at {}", self.notes_file_path.display());
            }
            // debug level
            _ => {
                println!("{:?}", self);
            }
        }
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

fn take_note(settings: Settings, note_value: &String) {
    println!("taking note {}", note_value);
    let mut file = get_notes_file(settings.notes_file_path);

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    writeln!(file, "{} - {}", timestamp, note_value).unwrap();
}

fn list_notes(settings: Settings, count: &u8) {
    if settings.verbosity > 0 {
        println!("Printing your last {} notes:", count);
    }

    let file = get_notes_file(settings.notes_file_path);
    let reader1 = BufReader::new(file);
    let reader = RevLines::new(reader1).unwrap();
    for (i, note) in reader.enumerate() {
        if i >= *count as usize {
            break;
        }
        match settings.verbosity {
            // warnings only
            0 => {
                println!("{}", note);
            }
            // info level
            _ => {
                println!("Note {}: {}", i, note);
            }
        }
    }
}

fn search_notes(_settings: Settings, search_term: &String, count: &u8) {
    println!(
        "searching notes with term {}, printing {} notes",
        search_term, count
    )
}

fn main() {
    let cli = Cli::parse();
    let verbose = cli.verbose;
    if verbose > 0 {
        println!("Running in verbose level {}.", verbose);
    };

    let note_file = match cli.notes_file {
        Some(notes_file) => notes_file,
        None => home_dir()
            .map(|p| p.join(".notes.txt"))
            .expect("No home dir found!!"),
    };

    let settings = Settings {
        notes_file_path: note_file,
        verbosity: verbose,
    };

    settings.print_to_console();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Take { note_value }) => take_note(settings, note_value),
        Some(Commands::List { count }) => list_notes(settings, count),
        Some(Commands::Search { term, count }) => search_notes(settings, term, count),
        _ => {}
    }
}
