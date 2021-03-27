use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::{thread::JoinHandle, time::Duration};
use tokio::sync::broadcast::{Receiver, Sender};

pub struct FileWatcher {
    thread: Option<JoinHandle<()>>,
    sender: Option<Sender<()>>,
    directory: String,
}

impl FileWatcher {
    pub fn new(directory: String) -> FileWatcher {
        FileWatcher {
            thread: None,
            sender: None,
            directory,
        }
    }

    pub fn start_watching(&mut self) {
        let (refresh_sender, _) = tokio::sync::broadcast::channel::<()>(32);

        let dir = self.directory.clone();
        let thread_sender = refresh_sender.clone();
        self.thread = Some(std::thread::spawn(move || {
            watch_for_file_changes(dir, thread_sender)
        }));

        self.sender = Some(refresh_sender);
    }

    pub fn stop_watching(&mut self) {
        if let Some(h) = self.thread.take() {
            h.join().unwrap();
        }
        self.sender = None;
    }

    pub fn subscribe(&self) -> Option<Receiver<()>> {
        if let Some(s) = &self.sender {
            Some(s.subscribe())
        } else {
            None
        }
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.stop_watching();
    }
}

fn watch_for_file_changes(directory: String, refresh: Sender<()>) {
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
                    }
                    DebouncedEvent::Remove(p) => {
                        println!("File removed: {:?}", p);

                        // ignore errors for now
                        let _ = refresh.send(());
                    }
                    DebouncedEvent::Rename(p1, p2) => {
                        println!("File renamed: {:?} -> {:?}", p1, p2);

                        // ignore errors for now
                        let _ = refresh.send(());
                    }
                    DebouncedEvent::Rescan => {
                        println!("Directory had to be rescanned");

                        // ignore errors for now
                        let _ = refresh.send(());
                    }
                    _ => {}
                }
            }
            Err(e) => println!("Error watching: {:?}", e),
        }
    }
}
