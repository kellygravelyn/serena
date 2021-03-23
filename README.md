# tennis 🎾

A simple static site server for local development. Because sometimes the best website is just a directory of files.


## Running

The simplest way is to just run the program:

```sh
tennis
```

This will start a new server at `localhost:3000` which serves the current directory as a website.

Of course you can also run `tennis --help` to see all the options available, including specifying a different directory, changing the port, etc.


## Development

Tennis is written in [Rust](http://rust-lang.org) using [Warp](https://github.com/seanmonstar/warp) as the server framework.

New feature requests and PRs are welcome, but please keep in mind that the goal is a very simple server for static files for local development.
