extern crate clap;
extern crate chrono;
extern crate csv;

use clap::{App, Arg, crate_version, crate_authors, crate_description};
use chrono::{Local, DateTime};

use crate::note::Note;
use std::path::Path;

mod note;
mod config;

/* CLI Commands */
const TAKE_COMMAND: &str = "take";
const TAKE_ARG_MESSAGE: &str = "message";

const PRINT_COMMAND: &str = "print";

const SETUP_COMMAND: &str = "setup";


fn main() {
    let matches = App::new("NOTEABLE - The CLI Note App")
        .author(crate_authors!())
        .version(format!("v{}", crate_version!()).as_str())
        .about(crate_description!())
        .subcommand(App::new(SETUP_COMMAND))
        .subcommand(App::new(TAKE_COMMAND)
            .about("Take a new note")
            .arg(Arg::with_name("append")
                .short('a')
                .long("append")
                .about("Appends this note to any notes you've taken today.
                  If there are 0 notes this will take a single note as default.
                  If there are 2+ notes, this will append to the first note for the day.")
            )
            .arg(Arg::with_name(TAKE_ARG_MESSAGE)
                .short('m')
                .long("message")
                .about("Takes a note without the prompt")
                .takes_value(true)
            )
        )
        .subcommand(App::new(PRINT_COMMAND)
            .about("Print notes")
        )
        // todo: build setup comand that builds out setup for all OSs
        // .arg(Arg::with_name(CONFIG_PATH_OPTION)
        //     .about("The path to your config .yml file")
        //     .long("config")
        //     .short('c')
        //     .default_value(CONFIG_DEFAULT_PATH)
        // )
        .get_matches();
    let command_time: DateTime<Local> = Local::now();
    // csv::ReaderBuilder::from_path()

    match matches.subcommand() {
        (TAKE_COMMAND, Some(arg)) => {
            let note_value: Note;
            if arg.is_present(TAKE_ARG_MESSAGE) {
                note_value = Note::from_string(arg.value_of(TAKE_ARG_MESSAGE).unwrap().to_string())
            } else {
                note_value = Note::from_prompt();
            };


        }
        (PRINT_COMMAND, Some(arg)) => {
            println!("print")
        }
        _ => {
            println!("none")
        }
    }
}
