use std::fs;
use std::time::Duration;
use clap::{Clap, crate_version, crate_authors};
use futures::{FutureExt, StreamExt, executor::ThreadPool};
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use tokio_stream::wrappers::BroadcastStream;
use warp::{Filter, reply::Reply, ws::Message};

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

async fn watch_for_file_changes(directory: String, refresh: tokio::sync::broadcast::Sender<()>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(200)).unwrap();
    watcher.watch(directory, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(p) => {
                        println!("File changed: {:?}", p);

                        // ignore errors for now
                        let _ = refresh.send(());
                    },
                    DebouncedEvent::Remove(p) => {
                        println!("File removed: {:?}", p);

                        // ignore errors for now
                        let _ = refresh.send(());
                    },
                    DebouncedEvent::Rename(p1, p2) => {
                        println!("File renamed: {:?} -> {:?}", p1, p2);

                        // ignore errors for now
                        let _ = refresh.send(());
                    },
                    DebouncedEvent::Rescan => {
                        println!("Directory had to be rescanned");

                        // ignore errors for now
                        let _ = refresh.send(());
                    },
                    _ => {},
                }
            },
            Err(e) => println!("Error watching: {:?}", e),
        }
    }
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    println!(
        "Serving static files from {} at localhost:{}",
        opts.directory,
        opts.port
    );

    let wants_to_watch = opts.watch;
    let pool = ThreadPool::new().unwrap();
    let (refresh_tx, _) = tokio::sync::broadcast::channel::<()>(32);
    
    let tx2 = refresh_tx.clone();
    let refresh_stream = warp::any().map(move || tx2.subscribe());

    let watch = warp::path("__tennis")
        .and(warp::ws())
        .and(refresh_stream)
        .map(|ws: warp::ws::Ws, refresh_stream: tokio::sync::broadcast::Receiver<()>| {
            ws.on_upgrade(move |websocket| {
                let (tx, _) = websocket.split();
                BroadcastStream::new(refresh_stream).map(|_| Ok(Message::text(""))).forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        });

    let file = warp::fs::dir(opts.directory.clone())
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

    if wants_to_watch {
        println!("Watching {} for changes…", opts.directory);
        pool.spawn_ok(watch_for_file_changes(opts.directory.clone(), refresh_tx));
    }

    warp::serve(watch.or(file))
        .run(([127, 0, 0, 1], opts.port))
        .await;
}
