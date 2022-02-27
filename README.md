# Serena

Fast and lightweight static site server for local development. Because sometimes the best website is just a directory of files.

Run `serena` in your terminal of choice to serve your current directory as a website at `localhost:3000`. Serena automatically refreshes your browser any time a file in the directory changes so iterating on your website is super fast. You can also pass a directory path or change the port if you need to: `serena some/other/directory --port 8080`. Run `serena --help` for full details on options.


## Installing

If you're just looking for a compiled executable to run, take a look at the [Releases](https://github.com/kellygravelyn/serena/releases) page.

If you're a Rust developer, you can clone the repo and just build it with `cargo build --release` like any other Rust project.


## Development

Serena is written in [Rust](http://rust-lang.org). New feature requests and PRs are welcome, but please keep in mind that the goal is a very simple server for static files for local development.
