extern crate core;

use clap::{Parser, Subcommand};

use crate::commands::{backup_notes, init, list_notes, open_path, search_notes, take_note};
use crate::internal::{get_scribr_config_file, get_settings_from_disk, scriber_files_setup};

mod commands;
mod internal;
mod model;

// https://docs.rs/clap/4.1.8/clap/_derive/index.html
#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    /// Use a config file
    // todo: remove this to just be an env var
    #[arg(long)]
    force_config_overwrite: bool,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Turn magic all off while searching or listing notes
    #[arg(long)]
    no_magic_commands: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

const PRINT_LEN_DEFAULT: &str = "20";

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
        #[arg(short = 'n', long, default_value = PRINT_LEN_DEFAULT)]
        count: u8,
    },

    /// 🔎 Search your notes with fuzzy matching
    Search {
        /// Term to search for across all notes.
        term: String,

        /// Number of notes to list.
        #[arg(short = 'n', long, default_value = PRINT_LEN_DEFAULT)]
        count: u8,
    },

    /// 📁 Open the notes file path
    Open,

    /// Init or re-init scribr
    Init {
        /// initialize with no backup to github
        #[arg(long)]
        no_gh: bool,
        /// force overwriting existing files - THIS WILL REMOVE ALL YOUR NOTES ON THIS MACHINE
        #[arg(long)]
        force: bool,
        /// use a specific gist for backing up notes
        #[arg(long)]
        gist_id: Option<String>,
    },

    /// ☁️ Interact with the GitHub in the context of scribr
    #[command()]
    Gh {
        #[command(subcommand)]
        command: Option<GhCommand>,
    },
}

#[derive(Subcommand)]
enum GhCommand {
    Init {
        #[arg(long)]
        gist_id: Option<String>,
    },

    /// Back up notes to a GitHub gist
    Backup {
        /// include the settings file in your backup
        #[arg(long)]
        settings: bool,
    },

    /// Restore your notes file from a GitHub gist
    Restore {
        /// Force overwriting your local file with the remote file
        #[arg(short, long)]
        force: bool,

        /// include the settings file in your restore
        #[arg(long)]
        settings: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if !scriber_files_setup() {
        println!("Scribr is not initialized on the machine! run scribr init");
        return;
    }

    let run_settings = get_settings_from_disk(Some(get_scribr_config_file()));

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Take { note, echo }) => take_note(run_settings, note, echo),
        Some(Commands::List { count }) => list_notes(run_settings, count),
        Some(Commands::Search { term, count }) => search_notes(run_settings, term, count),
        Some(Commands::Open {}) => open_path(),
        Some(Commands::Init {
            no_gh,
            force,
            gist_id,
        }) => init(no_gh, force, Option::from(gist_id)),
        Some(Commands::Gh { command }) => match command {
            Some(GhCommand::Backup { settings }) => backup_notes(run_settings, settings),
            // Some(GDhCommand::Restore {
            //     force: _force,
            //     gist_id,
            // }) => echo_under_construction(run_settings),
            _ => {}
        },
        _ => {}
    }
}
