use std::time::Duration;
use futures::executor::ThreadPool;
use hyper::{Body, Response, Result};
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use tokio::{sync::{broadcast::Sender, mpsc::UnboundedSender}, time::sleep};
use tokio_stream::wrappers::UnboundedReceiverStream;

static INJECTED_SCRIPT: &str = "
<script>
    (() => {
        let eventSource = new EventSource('/__tennis');
        eventSource.onmessage = () => location.reload();
    })();
</script>
";

async fn watch_for_file_changes(directory: String, refresh: tokio::sync::broadcast::Sender<()>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(200)).unwrap();
    watcher.watch(directory, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(p) => {
                        println!("File changed: {:?}", p);

                        // ignore errors for now
                        let _ = refresh.send(());
                    },
                    DebouncedEvent::Remove(p) => {
                        println!("File removed: {:?}", p);

                        // ignore errors for now
                        let _ = refresh.send(());
                    },
                    DebouncedEvent::Rename(p1, p2) => {
                        println!("File renamed: {:?} -> {:?}", p1, p2);

                        // ignore errors for now
                        let _ = refresh.send(());
                    },
                    DebouncedEvent::Rescan => {
                        println!("Directory had to be rescanned");

                        // ignore errors for now
                        let _ = refresh.send(());
                    },
                    _ => {},
                }
            },
            Err(e) => println!("Error watching: {:?}", e),
        }
    }
}

pub fn initialize_watching(directory: String, wants_to_watch: bool) -> Sender<()> {
    let pool = ThreadPool::new().unwrap();
    let (refresh_sender, _) = tokio::sync::broadcast::channel::<()>(32);

    if wants_to_watch {
        println!("Watching {} for changes…", directory);
        pool.spawn_ok(watch_for_file_changes(directory, refresh_sender.clone()));
    }

    refresh_sender
}

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
