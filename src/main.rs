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
}

static INJECTED_SCRIPT: &str = "
<script>
    const socket = new WebSocket(`ws://${location.host}/__tennis`);

    socket.onopen = (e) => {
      socket.send('Hello, world!');
    };

    socket.onmessage = (e) => {
      alert(`[message] Data received from server: ${event.data}`);
    };

    socket.onclose = (e) => {
      if (event.wasClean) {
        alert(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
      } else {
        alert('[close] Connection died');
      }
    };

    socket.onerror = (error) => {
      alert(`[error] ${error.message}`);
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

    warp::serve(watch.or(file))
        .run(([127, 0, 0, 1], opts.port))
        .await;
}