use clap::{Clap, crate_version, crate_authors};
use warp::{Filter, reply::Reply};
use std::fs;

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

static INJECTED_SCRIPT: &str = "
<script>
    console.log('boo');
</script>
";
#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    println!(
        "Hosting static files from {} at localhost:{}",
        opts.directory,
        opts.port
    );

    let route = warp::fs::dir(opts.directory)
        .map(|file: warp::filters::fs::File| {
            match file.path().extension() {
                Some(ext) if ext == "html" => {
                    let mut html = fs::read_to_string(file.path()).unwrap();
                    html.push_str(INJECTED_SCRIPT);
                    return warp::reply::html(html).into_response();
                },
                _ => {
                    return file.into_response();
                },
            }
        });

    warp::serve(route)
        .run(([127, 0, 0, 1], opts.port))
        .await;
}
