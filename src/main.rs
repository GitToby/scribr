use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use config::{Config, ConfigBuilder, File, FileFormat};
use dirs::{data_dir, home_dir};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config_file: Option<PathBuf>,

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
        #[arg(short, long)]
        note_value: String,
    },

    /// List your notes
    List { count: u8 },

    /// Search
    Search {
        /// Term to search for
        #[arg(short, long)]
        term: String,
    },
}

fn take_note(note_value: &String, _verbose: u8) {
    println!("taking note {}", note_value)
}

fn list_notes(count: &u8, _verbose: u8) {
    println!("printing {} notes", count)
}

fn search_notes(search_term: &String, _verbose: u8) {
    println!("searching notes with term {}", search_term)
}

fn main() {
    let cli = Cli::parse();
    let verbose = cli.verbose;
    if verbose > 0 {
        println!("verbose mode is on");
    };

    let config_path = match cli.config_file {
        None => {
            //use default config path
            home_dir()
                .map(|p| p.join("notes.txt"))
                .expect("No data dir found!!")
        }
        Some(config_path) => config_path,
    };

    if verbose > 0 {
        println!("Using config at {}", config_path.display());
    }

    let builder = Config::builder()
        .set_default("default", "1")
        .unwrap()
        .add_source(File::from(config_path))
        //  .add_async_source(...)
        .set_override("override", "1")
        .unwrap()
        .build();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Take { note_value }) => take_note(note_value, verbose),
        Some(Commands::List { count }) => list_notes(count, verbose),
        Some(Commands::Search { term }) => search_notes(term, verbose),
        _ => {}
    }
}
