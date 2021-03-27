use std::net::SocketAddr;

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
        watch::refresh_events(refresh_receiver).await
    } else {
        statics::transfer_static_file(path, root_dir).await
    }
}

