use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

use boost_fs::watch::{WatchBackend, WatchError, WatchOptions};
use oak_core::vfs::{DiskWatcher, VfsEvent, VfsWatchError, VfsWatcher};

const INTERVAL: Duration = Duration::from_millis(20);
const TIMEOUT: Duration = Duration::from_secs(10);

fn options() -> WatchOptions {
    WatchOptions::new().queue_capacity(256).backend(WatchBackend::Polling { interval: INTERVAL })
}

fn root_of(folder: &boost_fs::temp::TempFolder) -> PathBuf {
    normalize(fs::canonicalize(folder.path()).unwrap())
}

#[cfg(windows)]
fn normalize(path: PathBuf) -> PathBuf {
    use std::path::Prefix;

    let mut components = path.components();
    let Some(std::path::Component::Prefix(prefix)) = components.next()
    else {
        return path;
    };
    let rest: PathBuf = components.collect();
    match prefix.kind() {
        Prefix::VerbatimDisk(drive) => {
            let mut normalized = PathBuf::from(format!("{}:\\", drive as char));
            normalized.push(rest);
            normalized
        }
        _ => path,
    }
}

#[cfg(not(windows))]
fn normalize(path: PathBuf) -> PathBuf {
    path
}

fn wait_for(rx: &Receiver<VfsEvent>, predicate: impl Fn(&VfsEvent) -> bool) -> bool {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }
        match rx.recv_timeout(remaining) {
            Ok(event) if predicate(&event) => return true,
            Ok(_) => {}
            Err(_) => return false,
        }
    }
}

/// Asserts that no matching event arrives within `window`.
fn quiet_for(rx: &Receiver<VfsEvent>, window: Duration, predicate: impl Fn(&VfsEvent) -> bool) -> bool {
    let deadline = Instant::now() + window;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return true;
        }
        match rx.recv_timeout(remaining) {
            Ok(event) if predicate(&event) => return false,
            Ok(_) => {}
            Err(_) => return true,
        }
    }
}

#[test]
fn polling_reports_uris_relative_to_the_root() {
    let folder = boost_fs::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = DiskWatcher::with_options(&root, options(), move |event| drop(events.send(event))).unwrap();
    watcher.watch(root.to_str().unwrap()).unwrap();

    let file = root.join("a.txt");
    fs::write(&file, b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, VfsEvent::Created(uri) if uri == "a.txt")));

    fs::write(&file, b"longer").unwrap();
    assert!(wait_for(&received, |event| matches!(event, VfsEvent::Changed(uri) if uri == "a.txt")));

    fs::remove_file(&file).unwrap();
    assert!(wait_for(&received, |event| matches!(event, VfsEvent::Removed(uri) if uri == "a.txt")));

    watcher.close().unwrap();
}

#[test]
fn unwatch_all_unsubscribes_every_root() {
    let folder = boost_fs::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let (events, received) = mpsc::channel();
    let mut watcher = DiskWatcher::with_options(&root, options(), move |event| drop(events.send(event))).unwrap();
    watcher.watch("").unwrap();

    fs::write(root.join("before.txt"), b"one").unwrap();
    assert!(wait_for(&received, |event| matches!(event, VfsEvent::Created(uri) if uri == "before.txt")));

    watcher.unwatch_all().unwrap();
    fs::write(root.join("after.txt"), b"two").unwrap();
    assert!(quiet_for(&received, Duration::from_millis(300), |event| matches!(event, VfsEvent::Created(uri) if uri == "after.txt")));

    watcher.close().unwrap();
}

#[test]
fn missing_and_unsubscribed_roots_report_errors() {
    let folder = boost_fs::temp::create_folder().unwrap();
    let root = root_of(&folder);
    let missing = root.join("missing");
    let mut watcher = DiskWatcher::with_options(&root, options(), |_| {}).unwrap();

    let missing_uri = missing.to_str().unwrap();
    assert!(matches!(watcher.watch(missing_uri), Err(VfsWatchError::Watch(WatchError::NotFound(_)))));
    assert!(matches!(watcher.watch(root.to_str().unwrap()), Ok(())));
    assert!(matches!(watcher.unwatch(missing_uri), Err(VfsWatchError::Watch(WatchError::NotWatched(_)))));

    watcher.close().unwrap();
}
