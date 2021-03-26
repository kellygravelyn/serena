use std::fs;
use futures::{FutureExt, StreamExt, executor::ThreadPool};
use tokio_stream::wrappers::BroadcastStream;
use warp::{Filter, reply::Reply, ws::Message};

mod opts;
mod watch;

#[tokio::main]
async fn main() {
    let opts = opts::parse();

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
                        watch::attach_script(&mut html);
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
        pool.spawn_ok(watch::watch_for_file_changes(opts.directory.clone(), refresh_tx));
    }

    warp::serve(watch.or(file))
        .run(([127, 0, 0, 1], opts.port))
        .await;
}
