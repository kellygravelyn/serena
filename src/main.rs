use std::{net::SocketAddr, sync::Arc};

use hyper::{
    Body, 
    Error, 
    Request, 
    Response, 
    Result, 
    Server,
    service::{
        make_service_fn, 
        service_fn
    }
};

use crate::{
    file_watcher::FileWatcher, 
    handlers::transfer_file, 
    handlers::refresh_events,
    handlers::not_found,
    opts::Opts
};

mod content_type;
mod file_watcher;
mod opts;
mod handlers;

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));
    
    println!(
        "Serving static files from {} at http://{}",
        opts.directory,
        addr
    );

    let mut watcher = FileWatcher::new(opts.directory.clone());
    if opts.watch {
        watcher.start_watching();
    }
    let watcher = Arc::new(watcher);

    let root_dir = opts.directory.clone();
    let make_service = make_service_fn(move |_| {
        let root_dir = root_dir.clone();
        let watcher = watcher.clone();
        async move { 
            Ok::<_, Error>(service_fn(move |req| {
                handle_request(req, root_dir.clone(), watcher.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>, root_dir: String, watcher: Arc<FileWatcher>) -> Result<Response<Body>> {
    let path = req.uri().path();
    if path == "/__serena" {
        if let Some(receiver) = watcher.subscribe() {
            refresh_events(receiver).await
        } else {
            not_found()
        }
    } else {
        transfer_file(path, root_dir).await
    }
}

