use clap::{Clap, crate_version, crate_authors};
use warp::{Filter, reply::Reply};

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

    let route = warp::fs::dir(opts.directory)
        .map(|reply: warp::filters::fs::File| {
            println!("Serving {:?}", reply.path());
            match reply.path().extension() {
                Some(ext) if ext == "html" => {
                    println!("html!");
                    return warp::reply::html("<h1>Intercepted!</h1>").into_response();
                },
                _ => {
                    println!("not html!");
                    return reply.into_response();
                },
            }
        });

    warp::serve(route)
        .run(([127, 0, 0, 1], opts.port))
        .await;
}
