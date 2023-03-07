extern crate core;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dirs::home_dir;

use crate::commands::{
    echo_path, echo_under_construction, gh_fetch_gists, list_notes, search_notes, take_note,
};
use crate::model::Settings;

mod commands;
mod model;

// https://docs.rs/clap/4.1.8/clap/_derive/index.html
#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    /// Sets a custom note output file
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Turn magic all off while searching or listing notes
    #[arg(short, long)]
    no_magic_commands: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum GhCommand {
    /// Back up notes to a GitHub gist
    Backup {
        #[arg(long)]
        gist_id: String,
    },

    /// Restore your notes file from a GitHub gist
    Restore {
        /// Force overwriting your local file with the remote file
        #[arg(short, long)]
        force: bool,

        #[arg(long)]
        gist_id: String,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Take a note
    Take {
        /// The note value the note should contain.
        note: String,

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

    /// ☁️ Interact with the GitHub in the context of scribr
    #[command()]
    Gh {
        #[command(subcommand)]
        command: Option<GhCommand>,
    },
}

fn main() {
    let cli = Cli::parse();

    let note_file = match cli.file {
        Some(notes_file) => notes_file,
        None => {
            let note_path = home_dir()
                .map(|p| p.join(".notes.txt"))
                .expect("No home dir found!!");
            println!("Setting custom note path {}", note_path.display());
            note_path
        }
    };

    let settings = Settings {
        scribr_data_dir: note_file,
        verbosity: cli.verbose,
        no_magic_commands: cli.no_magic_commands,
    };

    settings.print_to_console();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Take { note, echo }) => take_note(settings, note, echo),
        Some(Commands::List { count }) => list_notes(settings, count),
        Some(Commands::Search { term, count }) => search_notes(settings, term, count),
        Some(Commands::Path {}) => echo_path(settings),
        Some(Commands::Gh { command }) => match command {
            Some(GhCommand::Backup { gist_id }) => echo_under_construction(settings),
            Some(GhCommand::Restore {
                force: _force,
                gist_id,
            }) => echo_under_construction(settings),
            _ => {}
        },
        _ => {}
    }
}
