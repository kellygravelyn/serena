use std::time::Duration;
use futures::executor::ThreadPool;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
use tokio::sync::broadcast::Sender;

static INJECTED_SCRIPT: &str = "
<script>
    (() => {
        let eventSource = null;
        console.log('[Tennis] Connecting to event source for automatic page reload');
        eventSource = new EventSource('/__tennis');
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
