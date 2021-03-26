use std::time::Duration;
use futures::{SinkExt, StreamExt, executor::ThreadPool};
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use tokio::sync::broadcast::{Receiver, Sender};
use warp::ws::{Message, WebSocket};

static INJECTED_SCRIPT: &str = "
<script>
    (() => {
        let socket = null;
        let wasDisconnected = false;
        let connectionAttempts = 0;
        const openConnection = () => {
            console.log('[Tennis] Connecting to web socket for automatic page reload', `ws://${location.host}/__tennis`);
            socket = new WebSocket(`ws://${location.host}/__tennis`);
            socket.onopen = () => {
                connectionAttempts = 0;
                if (wasDisconnected) {
                    location.reload();
                }
            };
            socket.onmessage = () => location.reload();
            socket.onclose = () => {
                wasDisconnected = true;
                connectionAttempts++;

                if (connectionAttempts < 12) {
                    console.log('[Tennis] Web socket closed. Trying to reconnect in 5 seconds.');
                    setTimeout(openConnection, 5000);
                } else {
                    console.log('[Tennis] Failed to reconnect to the websocket. Will no longer attempt to reconnect. Manually refresh the page once you have restarted the server.');
                }
            };
        };
        openConnection();
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

pub async fn handle_websocket_client(
    websocket: WebSocket, 
    mut refresh_receiver: Receiver<()>
) {
    let (mut tx, _) = websocket.split();
    let _ = refresh_receiver.recv().await;
    let _ = tx.send(Message::text("")).await;
}

pub fn initialize_watching(directory: String, wants_to_watch: bool) -> Sender<()> {
    let pool = ThreadPool::new().unwrap();
    let (refresh_sender, _) = tokio::sync::broadcast::channel::<()>(32);
    let refresh_sender2 = refresh_sender.clone();

    if wants_to_watch {
        println!("Watching {} for changes…", directory);
        pool.spawn_ok(watch_for_file_changes(directory, refresh_sender));
    }

    refresh_sender2
}

pub fn attach_script(html: &mut String) {
    html.push_str(INJECTED_SCRIPT);
}
