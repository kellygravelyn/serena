# tennis 🎾

A simple static site server for local development. Because sometimes the best website is just a directory of files.


## Running

The simplest way is to just run the program:

```sh
tennis
```

This will start a new server at `localhost:3000` which serves the current directory as a website.

You can also specify options for the directory, port, and a flag to automatically reload browsers when files change:

```sh
tennis /path/to/directory --port=8080 --watch
```

Run `tennis --help` at any time to see the help guide.


## Development

Tennis is written in [Rust](http://rust-lang.org). New feature requests and PRs are welcome, but please keep in mind that the goal is a very simple server for static files for local development.
