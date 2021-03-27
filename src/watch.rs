use std::time::Duration;
use hyper::{Body, Response, Result};
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use tokio_stream::wrappers::UnboundedReceiverStream;

static INJECTED_SCRIPT: &str = "
<script>
    (() => {
        let eventSource = new EventSource('/__tennis');
        eventSource.onmessage = () => location.reload();
    })();
</script>
";

pub fn attach_script(html: &mut String) {
    html.push_str(INJECTED_SCRIPT);
}

pub async fn refresh_events(refresh_receiver: tokio::sync::broadcast::Receiver<()>) -> Result<Response<Body>> {
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
