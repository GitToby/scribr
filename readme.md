# `scribr` - A CLI Note Taker in rust

## What

Introducing `scribr`, the all-in-one note-taking tool that allows you to effortlessly jot down notes, and securely save
them to a file. With `scribr`, you can easily keep track of all your notes and view them on-demand, in the console in
the language you took them in. It's not a complicated tool, just the core note-taking tooling that can be run by a
terminal system, so you can focus on the note and its context rather than the padding around the note.

🔨👷 We will also support backups to GitHub Gists via Oauth. 

🔨👷 We will also support refs & magic commands. 

## Why

When working on 100 things at once as a dev, I want to be asked to take notes on my day-to-day goings-on. `scribr` 
tries to create an easier environment to take notes if you operate in the terminal every day.

## `$ scribr --help`

```
Take notes in the CLI and back them up to GitHub! 📓🚀

Usage: scribr [OPTIONS] [COMMAND]

Commands:
  take     Take a note
  list     📑 List your notes chronologically
  search   🔎 Search your notes with fuzzy matching
  path     📁 Echo the notes file path
  backup   Back up notes to a GitHub gist
  restore  Restore your notes file from a GitHub gist
  help     Print this message or the help of the given subcommand(s)

Options:
  -f, --file <FILE>        Sets a custom config file
  -v, --verbose...         Turn debugging information on
  -n, --no-magic-commands  Turn magic commands off while searching or listing notes
  -h, --help               Print help
  -V, --version            Print version
```