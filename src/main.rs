use std::{net::SocketAddr, path::{Path, PathBuf}};

use tokio::{fs::File, io::AsyncReadExt}; 
use tokio_util::codec::{BytesCodec, FramedRead};
use hyper::{Body, Error, Request, Response, Result, Server, StatusCode, service::{make_service_fn, service_fn}};

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

fn not_found() -> Result<Response<Body>> {
    Ok(
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(""))
            .unwrap()
    )
}

async fn transfer_static_file(path: &str, root_dir: String) -> Result<Response<Body>> {
    let filepath = build_file_path(&path, &root_dir);
    if let Ok(file) = File::open(&filepath).await {
        if is_html_file(&filepath) {
            html_response(file).await
        } else {
            file_stream_response(file).await
        }
    } else {
        not_found()
    }
}

async fn html_response(mut file: File) -> Result<Response<Body>> {
    let mut html = String::new();
    let _ = file.read_to_string(&mut html).await;
    watch::attach_script(&mut html);
    Ok(Response::new(Body::from(html)))
}

async fn file_stream_response(file: File) -> Result<Response<Body>> {
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);
    Ok(Response::new(body))
}

fn is_html_file(filepath: &Path) -> bool {
    if let Some(ext) = filepath.extension() {
        ext == "html"
    } else {
        false
    }
}

fn build_file_path(path: &str, root_dir: &String) -> PathBuf {
    let trimmed_characters: &[_] = &['/', '.'];
    let mut filepath = Path::new(&root_dir).join(path.trim_start_matches(trimmed_characters));
    if filepath.is_dir() {
        filepath = filepath.join("index.html");
    }
    filepath
}
