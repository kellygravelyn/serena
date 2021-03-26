use std::time::Duration;
use futures::{SinkExt, StreamExt};
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use warp::ws::{Message, WebSocket};

static INJECTED_SCRIPT: &str = "
<script>
    const socket = new WebSocket(`ws://${location.host}/__tennis`);
    socket.onmessage = (e) => location.reload();
    socket.onclose = (e) => {
        // TODO: Try to reconnect over time to support cases where
        // the server is stopped and then restarted so the page
        // automatically reloads when the server starts up again.
    };
</script>
";

pub async fn watch_for_file_changes(directory: String, refresh: tokio::sync::broadcast::Sender<()>) {
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

pub async fn handle_websocket_client(
    websocket: WebSocket, 
    mut refresh_receiver: tokio::sync::broadcast::Receiver<()>
) {
    let (mut tx, _) = websocket.split();
    let _ = refresh_receiver.recv().await;
    let _ = tx.send(Message::text("")).await;
}

pub fn attach_script(html: &mut String) {
    html.push_str(INJECTED_SCRIPT);
}
