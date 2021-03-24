use std::fs;
use clap::{Clap, crate_version, crate_authors};
use futures::{FutureExt, StreamExt};
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

    /// Automatically refresh the page when a change to the files is detected.
    #[clap(short, long)]
    watch: bool,
}

static INJECTED_SCRIPT: &str = "
<script>
    const socket = new WebSocket(`ws://${location.host}/__tennis`);
    socket.onmessage = (e) => location.reload();
    socket.onclose = (e) => {
        // TODO: Try to reconnect over time to support cases where
        // the server is stopped and then restarted so the page
        // automatically reloads when the server starts up again.
    };
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

    let wants_to_watch = opts.watch;

    let watch = warp::path("__tennis")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(|websocket| {
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        });

    let file = warp::fs::dir(opts.directory)
        .map(move |file: warp::filters::fs::File| {
            if wants_to_watch {
                match file.path().extension() {
                    Some(ext) if ext == "html" => {
                        let mut html = fs::read_to_string(file.path()).unwrap();
                        html.push_str(INJECTED_SCRIPT);
                        warp::reply::html(html).into_response()
                    },
                    _ => {
                        file.into_response()
                    },
                }
            } else {
                file.into_response()
            }
        });


    warp::serve(watch.or(file))
        .run(([127, 0, 0, 1], opts.port))
        .await;
}
