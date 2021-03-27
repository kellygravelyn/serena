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
    common::not_found, 
    file_watcher::FileWatcher, 
    statics::transfer_static_file, 
    watch::refresh_events
};

mod common;
mod file_watcher;
mod opts;
mod statics;
mod watch;

#[tokio::main]
async fn main() {
    let opts = opts::parse();
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
                routes(req, root_dir.clone(), watcher.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn routes(req: Request<Body>, root_dir: String, watcher: Arc<FileWatcher>) -> Result<Response<Body>> {
    let path = req.uri().path();
    if path == "/__tennis" {
        if let Some(receiver) = watcher.subscribe() {
            refresh_events(receiver).await
        } else {
            not_found()
        }
    } else {
        transfer_static_file(path, root_dir).await
    }
}

