# `scribr` - A CLI Note Taker in rust - https://gittoby.github.io/scribr/

## Drive

When working on 100 things at once as a dev, I want to be asked to take notes on my day to day goings on.
NoteAble should be able to prompt the user for a note, and write that note to a output file for safe keeping along with all the necessary tags.
It shouldn't be an orchestration tool, just the core note taking prompt and config tool that can be run by another platform, such as an OS.

## Usage

Pull help with

```bash
scribr --help
```

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