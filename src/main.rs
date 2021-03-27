use std::{net::SocketAddr, time::Duration};

use tokio::{sync::mpsc::UnboundedSender, time::sleep}; 
use tokio_stream::wrappers::UnboundedReceiverStream;
use hyper::{Body, Error, Request, Response, Result, Server, service::{make_service_fn, service_fn}};

mod opts;
mod watch;
mod statics;

#[tokio::main]
async fn main() {
    let opts = opts::parse();
    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    
    println!(
        "Serving static files from {} at http://{}",
        opts.directory,
        addr
    );

    let refresh_sender = watch::initialize_watching(opts.directory.clone(), opts.watch);

    let root_dir = opts.directory.clone();
    let make_service = make_service_fn(move |_| {
        let root_dir = root_dir.clone();
        let refresh_receiver = refresh_sender.clone();
        async move { 
            Ok::<_, Error>(service_fn(move |req| {
                routes(req, root_dir.clone(), refresh_receiver.subscribe())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn routes(req: Request<Body>, root_dir: String, refresh_receiver: tokio::sync::broadcast::Receiver<()>) -> Result<Response<Body>> {
    let path = req.uri().path();
    if path == "/__tennis" {
        refresh_events(refresh_receiver).await
    } else {
        statics::transfer_static_file(path, root_dir).await
    }
}

async fn refresh_events(refresh_receiver: tokio::sync::broadcast::Receiver<()>) -> Result<Response<Body>> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Result<String>>();

    keep_alive(sender.clone());
    map_refresh_events(sender, refresh_receiver);

    Ok(
        Response::builder()
            .status(200)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("connection", "keep-alive")
            .version(hyper::Version::HTTP_2)
            .body(Body::wrap_stream(UnboundedReceiverStream::new(receiver)))
            .unwrap()
    )
}

fn keep_alive(sender: UnboundedSender<Result<String>>) {
    tokio::spawn(async move {
        loop {
            // If we fail to send to the client, exit the task
            if let Err(_) = sender.send(Ok(":\n\n".to_string())) {
                break;
            }
            sleep(Duration::from_secs(15)).await;
        }
    });
}

fn map_refresh_events(sender: UnboundedSender<Result<String>>, mut refresh_receiver: tokio::sync::broadcast::Receiver<()>) {
    tokio::spawn(async move { 
        loop {
            match refresh_receiver.recv().await {
                Ok(_) => { 
                    if let Err(_) = sender.send(Ok("data: reload\n\n".to_string())) {
                        break;
                    } 
                },
                Err(_) => { break; },
            }
        }  
    });
}
