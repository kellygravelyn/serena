use std::{path::{Path, PathBuf}, time::Duration};

use hyper::{Body, Response, Result};
use tokio::{fs::File, io::AsyncReadExt, sync::{broadcast::Receiver, mpsc::UnboundedSender}, time::sleep};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::codec::{BytesCodec, FramedRead};

pub fn not_found() -> Result<Response<Body>> {
    Ok(
        Response::builder()
            .status(404)
            .body(Body::from(""))
            .unwrap()
    )
}

pub async fn transfer_file(path: &str, root_dir: String) -> Result<Response<Body>> {
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
        html.push_str(INJECTED_SCRIPT);
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

static INJECTED_SCRIPT: &str = "
<script>
    (() => {
        let eventSource = new EventSource('/__tennis');
        eventSource.onmessage = () => location.reload();
    })();
</script>
";

pub async fn refresh_events(refresh_receiver: Receiver<()>) -> Result<Response<Body>> {
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

fn map_refresh_events(
    sender: UnboundedSender<Result<String>>, 
    mut refresh_receiver: Receiver<()>
) {
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
