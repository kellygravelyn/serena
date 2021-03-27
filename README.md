# Serena

Fast and lightweight static site server for local development. Because sometimes the best website is just a directory of files.


## Running

The simplest way is to just run the program:

```sh
serena
```

This will start a new server at `localhost:3000` which serves the current directory as a website.

You can also specify options for the directory, port, and a flag to automatically reload browsers when files change:

```sh
serena /path/to/directory --port=8080 --watch
```

Run `serena --help` at any time to see the help guide.


## Development

Serena is written in [Rust](http://rust-lang.org). New feature requests and PRs are welcome, but please keep in mind that the goal is a very simple server for static files for local development.
