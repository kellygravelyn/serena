use clap::{Clap, crate_version, crate_authors};

/// Tennis is a very simple static website server for local development.
#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct Opts {
    /// The directory that will act as the root for static files.
    #[clap(default_value = ".")]
    directory: String,

    /// The port on which to run the server.
    #[clap(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    println!(
        "Hosting static files from {} at localhost:{}",
        opts.directory,
        opts.port
    );

    warp::serve(warp::fs::dir(opts.directory))
        .run(([127, 0, 0, 1], opts.port))
        .await;
}
