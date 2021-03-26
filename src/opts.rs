use clap::{Clap, crate_version, crate_authors};

/// Tennis is a very simple static website server for local development.
#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Opts {
    /// The directory that will act as the root for static files.
    #[clap(default_value = ".")]
    pub directory: String,

    /// The port on which to run the server.
    #[clap(short, long, default_value = "3000")]
    pub port: u16,

    /// Automatically refresh the page when a change to the files is detected.
    #[clap(short, long)]
    pub watch: bool,
}

pub fn parse() -> Opts {
    Opts::parse()
}
