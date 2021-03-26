use std::{net::SocketAddr, path::Path};

use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use hyper::{Error, service::{make_service_fn, service_fn}};
use hyper::{Body, Method, Request, Response, Result, Server, StatusCode};

mod opts;
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

    let root_dir = opts.directory.clone();
    let make_service = make_service_fn(move |_| {
        let root_dir = root_dir.clone();
        async move { 
            Ok::<_, Error>(service_fn(move |req| {
                routes(req, root_dir.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn routes(req: Request<Body>, root_dir: String) -> Result<Response<Body>> {
    let path = req.uri().path();
    if path == "/__tennis" {
        Ok(Response::new("Websocket eventually".into()))
    } else {
        transfer_static_file(path, root_dir).await
    }
}

static NOTFOUND: &[u8] = b"Not Found";
fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

async fn transfer_static_file(path: &str, root_dir: String) -> Result<Response<Body>> {
    let trimmed_characters: &[_] = &['/', '.'];
    let mut filepath = Path::new(&root_dir).join(path.trim_start_matches(trimmed_characters));
    if filepath.is_dir() {
        filepath = filepath.join("index.html");
    }

    if let Ok(file) = File::open(filepath).await {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        return Ok(Response::new(body));
    }

    Ok(not_found())
}
