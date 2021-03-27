use std::path::{Path, PathBuf};

use hyper::{Body, Response, Result};
use tokio::{fs::File, io::AsyncReadExt};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{common::not_found, watch::attach_script};

pub async fn transfer_static_file(path: &str, root_dir: String) -> Result<Response<Body>> {
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
    if let Ok(_) = file.read_to_string(&mut html).await {
        attach_script(&mut html);
        Ok(Response::new(Body::from(html)))
    } else {
        Ok(
            Response::builder()
                .status(500)
                .body(Body::from("Failed to read file"))
                .unwrap()
        )
    }
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
