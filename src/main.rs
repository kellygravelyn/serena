use std::{convert::Infallible, fs};
use tokio::sync::broadcast::{Receiver, Sender};
use warp::{Filter, reply::Reply};

mod opts;
mod watch;

fn refresh_receiver(sender: Sender<()>) -> impl Filter<Extract = (Receiver<()>, ), Error = Infallible> + Clone {
    warp::any().map(move || sender.subscribe())
}

#[tokio::main]
async fn main() {
    let opts = opts::parse();

    println!(
        "Serving static files from {} at localhost:{}",
        opts.directory,
        opts.port
    );

    let wants_to_watch = opts.watch;
    let refresh_sender = watch::initialize_watching(
        opts.directory.clone(),
        wants_to_watch
    );

    let watch = warp::path("__tennis")
        .and(warp::ws())
        .and(refresh_receiver(refresh_sender.clone()))
        .map(|ws: warp::ws::Ws, refresh_receiver: tokio::sync::broadcast::Receiver<()>| {
            ws.on_upgrade(move |websocket| {
                watch::handle_websocket_client(websocket, refresh_receiver)
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

    warp::serve(watch.or(file))
        .run(([127, 0, 0, 1], opts.port))
        .await;
}
