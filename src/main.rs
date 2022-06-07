use std::{net::SocketAddr, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Error, Server,
};

use webbrowser;

use crate::{file_watcher::FileWatcher, handlers::handle_request, opts::Opts};

mod file_watcher;
mod handlers;
mod opts;

#[tokio::main]
async fn main() {
    let opts = Opts::parse();
    let addr = SocketAddr::from(([127, 0, 0, 1], opts.port));

    println!(
        "Serving static files from {} at http://{}",
        opts.directory, addr
    );

    let watcher = if opts.no_auto_refresh {
        None
    } else {
        Some(Arc::new(FileWatcher::new(opts.directory.clone())))
    };

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

    if opts.open {
        if !webbrowser::open(format!("http://localhost:{}", opts.port).as_str()).is_ok() {
            println!("Failed to open browser");
        }
    }

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
