extern crate clap;
extern crate chrono;
extern crate csv;
extern crate serde;
extern crate ansi_term;

use clap::{App, Arg, crate_version, crate_authors, crate_description};
use chrono::{Local, DateTime};

use crate::note::Note;
use std::path::Path;
use std::fs::{File, OpenOptions};
use csv::Error;

mod note;
mod setup;

/* CLI Commands */
const TAKE_COMMAND: &str = "take";
const TAKE_ARG_MESSAGE: &str = "message";

const PRINT_COMMAND: &str = "print";

const SETUP_COMMAND: &str = "setup";

const OUT_FILE_LOCATION: &str = "./out.csv";

fn main() {
    let enabled = ansi_term::enable_ansi_support();
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

    let f = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(Path::new(OUT_FILE_LOCATION))
        .expect("error opening file");

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .comment(Some(b'#'))
        .from_reader(
            f.try_clone().expect("Error with file opening for read")
        );

    match matches.subcommand() {
        (TAKE_COMMAND, Some(arg)) => {
            let note_value: Note;
            if arg.is_present(TAKE_ARG_MESSAGE) {
                note_value = Note::from_string(arg.value_of(TAKE_ARG_MESSAGE).unwrap().to_string())
            } else {
                note_value = Note::from_prompt();
            };

            let write_header_row = match csv_reader.deserialize::<Note>().next() {
                // if there's nothing in the file we can assume there's no headers, we should write them
                None => true,
                // if there is something there probably been written before and we can ignore
                Some(_) => false
            };

            let mut writer = csv::WriterBuilder::new()
                .has_headers(write_header_row)
                .from_writer(
                    f.try_clone().expect("error")
                );
            writer.serialize(note_value);
        }
        (PRINT_COMMAND, Some(arg)) => {
            println!("print");
            let records_iter = csv_reader.deserialize::<Note>();
            for r in records_iter {
                let record: Note = r.unwrap();
                println!("{:?}", record);
                //https://docs.rs/csv/1.1.3/csv/tutorial/index.html#setup
            };
        }
        _ => {
            println!("none")
        }
    }
}
