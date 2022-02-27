use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use hyper::{header::HeaderValue, Body, Request, Response, Result};
use tokio::{
    fs::File,
    sync::{broadcast::Receiver, mpsc::UnboundedSender},
    time::sleep,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{content_type::content_type_from_path, file_watcher::FileWatcher};

pub async fn handle_request(
    req: Request<Body>,
    root_dir: String,
    watcher: Option<Arc<FileWatcher>>,
) -> Result<Response<Body>> {
    if let Some(refresh_receiver) = is_refresh_request(&req, watcher) {
        return refresh_events(refresh_receiver).await;
    } else {
        transfer_file(req.uri().path(), root_dir).await
    }
}

fn is_refresh_request(
    req: &Request<Body>,
    watcher: Option<Arc<FileWatcher>>,
) -> Option<Receiver<()>> {
    if req.uri().path() == "/__serena" {
        if let Some(watcher) = watcher {
            return watcher.subscribe();
        }
    }

    None
}

fn not_found() -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(404)
        .body(Body::from(""))
        .unwrap())
}

async fn transfer_file(path: &str, root_dir: String) -> Result<Response<Body>> {
    let filepath = build_file_path(&path, &root_dir);
    if let Ok(file) = File::open(&filepath).await {
        if is_html_file(&filepath) {
            html_response(&filepath).await
        } else {
            file_stream_response(&filepath, file).await
        }
    } else {
        not_found()
    }
}

async fn html_response(filepath: &Path) -> Result<Response<Body>> {
    if let Ok(mut html) = read_to_string(filepath) {
        html.push_str(INJECTED_SCRIPT);
        Ok(Response::new(Body::from(html)))
    } else {
        Ok(Response::builder()
            .status(500)
            .body(Body::from("Failed to read file"))
            .unwrap())
    }
}

async fn file_stream_response(filepath: &Path, file: File) -> Result<Response<Body>> {
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);

    let mut response = Response::new(body);
    if let Some(content_type) = content_type_from_path(filepath) {
        response.headers_mut().insert(
            "content-type",
            HeaderValue::from_str(&content_type[..]).unwrap(),
        );
    }
    Ok(response)
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

static INJECTED_SCRIPT: &str = "
<script>
    (() => {
        let eventSource = new EventSource('/__serena');
        eventSource.onmessage = () => location.reload();
    })();
</script>
";

async fn refresh_events(refresh_receiver: Receiver<()>) -> Result<Response<Body>> {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Result<String>>();

    keep_alive(sender.clone());
    map_refresh_events(sender, refresh_receiver);

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .header("connection", "keep-alive")
        .body(Body::wrap_stream(UnboundedReceiverStream::new(receiver)))
        .unwrap())
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

fn map_refresh_events(sender: UnboundedSender<Result<String>>, mut refresh_receiver: Receiver<()>) {
    tokio::spawn(async move {
        loop {
            match refresh_receiver.recv().await {
                Ok(_) => {
                    if let Err(_) = sender.send(Ok("data: reload\n\n".to_string())) {
                        break;
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    });
}
