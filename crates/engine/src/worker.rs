//! Background work so the UI thread never touches the disk (AGENTS.md: UI スレッドを止めない).
//!
//! One I/O thread lists folders and reads metadata; a pool decodes images, newest request first,
//! and drops requests the UI no longer wants (scrolled away) before decoding them.

use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex, PoisonError};

use crate::decode::{Rgba, decode_scaled};
use crate::meta::{MetaSummary, read_summary};
use crate::model::Asset;

/// Longest side of a decoded thumbnail, in pixels (the largest thumbnail cell on a 2× display).
pub const THUMB_SIDE: u32 = 512;
/// Longest side of a decoded preview, in pixels.
pub const PREVIEW_SIDE: u32 = 2048;
/// Decoder threads at most, so a big machine still leaves cores for the UI and other apps.
const MAX_DECODE_THREADS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageSize {
    Thumb,
    Preview,
}

impl ImageSize {
    fn max_side(self) -> u32 {
        match self {
            Self::Thumb => THUMB_SIDE,
            Self::Preview => PREVIEW_SIDE,
        }
    }
}

pub enum Event {
    Folder {
        generation: u64,
        path: PathBuf,
        result: Result<Vec<Asset>, String>,
    },
    Subfolders {
        path: PathBuf,
        result: Result<Vec<PathBuf>, String>,
    },
    Image {
        path: PathBuf,
        size: ImageSize,
        result: Result<Rgba, String>,
    },
    /// The request was dropped because the UI stopped wanting it; ask again when it is wanted.
    Skipped {
        path: PathBuf,
        size: ImageSize,
    },
    Meta {
        path: PathBuf,
        summary: Box<MetaSummary>,
    },
}

enum IoJob {
    Folder { generation: u64, path: PathBuf },
    Subfolders { path: PathBuf },
    Meta { path: PathBuf },
}

type Key = (PathBuf, ImageSize);

struct ImageQueue {
    jobs: Mutex<VecDeque<Key>>,
    ready: Condvar,
    wanted: Mutex<HashSet<Key>>,
    shutdown: AtomicBool,
}

pub struct Worker {
    io: Sender<IoJob>,
    images: Arc<ImageQueue>,
    events: Receiver<Event>,
    pending: Arc<AtomicUsize>,
}

type Wake = Arc<dyn Fn() + Send + Sync>;

impl Worker {
    /// `wake` is called after each event is queued (the UI passes its repaint request).
    pub fn new(wake: Wake) -> Self {
        let (event_tx, events) = channel();
        let (io, io_rx) = channel();
        let pending = Arc::new(AtomicUsize::new(0));
        let images = Arc::new(ImageQueue {
            jobs: Mutex::new(VecDeque::new()),
            ready: Condvar::new(),
            wanted: Mutex::new(HashSet::new()),
            shutdown: AtomicBool::new(false),
        });

        let tx = event_tx.clone();
        let w = wake.clone();
        let p = pending.clone();
        std::thread::Builder::new().name("pier-io".into()).spawn(move || io_loop(&io_rx, &tx, &w, &p)).ok();

        let threads = std::thread::available_parallelism().map_or(2, |n| n.get().saturating_sub(1)).clamp(1, MAX_DECODE_THREADS);
        for i in 0..threads {
            let (q, tx, w, p) = (images.clone(), event_tx.clone(), wake.clone(), pending.clone());
            std::thread::Builder::new().name(format!("pier-decode-{i}")).spawn(move || decode_loop(&q, &tx, &w, &p)).ok();
        }
        Self { io, images, events, pending }
    }

    pub fn list_folder(&self, generation: u64, path: PathBuf) {
        self.send_io(IoJob::Folder { generation, path });
    }

    pub fn list_subfolders(&self, path: PathBuf) {
        self.send_io(IoJob::Subfolders { path });
    }

    pub fn read_meta(&self, path: PathBuf) {
        self.send_io(IoJob::Meta { path });
    }

    fn send_io(&self, job: IoJob) {
        self.pending.fetch_add(1, Ordering::SeqCst);
        if self.io.send(job).is_err() {
            self.pending.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// Replaces the set of images the UI is showing; queued requests outside it are dropped.
    pub fn set_wanted(&self, wanted: HashSet<(PathBuf, ImageSize)>) {
        *self.images.wanted.lock().unwrap_or_else(PoisonError::into_inner) = wanted;
    }

    pub fn request_image(&self, path: PathBuf, size: ImageSize) {
        self.pending.fetch_add(1, Ordering::SeqCst);
        self.images.jobs.lock().unwrap_or_else(PoisonError::into_inner).push_back((path, size));
        self.images.ready.notify_one();
    }

    pub fn try_recv(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }

    /// No job queued or running (tests and offscreen snapshots wait for this).
    pub fn is_idle(&self) -> bool {
        self.pending.load(Ordering::SeqCst) == 0
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.images.shutdown.store(true, Ordering::SeqCst);
        self.images.ready.notify_all();
    }
}

fn emit(tx: &Sender<Event>, wake: &Wake, pending: &AtomicUsize, event: Event) {
    let _ = tx.send(event);
    pending.fetch_sub(1, Ordering::SeqCst);
    wake();
}

fn io_loop(rx: &Receiver<IoJob>, tx: &Sender<Event>, wake: &Wake, pending: &AtomicUsize) {
    while let Ok(job) = rx.recv() {
        let event = match job {
            IoJob::Folder { generation, path } => {
                let result = crate::fs::list_folder(&path);
                Event::Folder { generation, path, result }
            }
            IoJob::Subfolders { path } => {
                let result = crate::fs::list_subfolders(&path);
                Event::Subfolders { path, result }
            }
            IoJob::Meta { path } => {
                let summary = Box::new(read_summary(&path));
                Event::Meta { path, summary }
            }
        };
        emit(tx, wake, pending, event);
    }
}

fn decode_loop(q: &ImageQueue, tx: &Sender<Event>, wake: &Wake, pending: &AtomicUsize) {
    loop {
        let job = {
            let mut jobs = q.jobs.lock().unwrap_or_else(PoisonError::into_inner);
            loop {
                if q.shutdown.load(Ordering::SeqCst) {
                    return;
                }
                if let Some(job) = jobs.pop_back() {
                    break job;
                }
                jobs = q.ready.wait(jobs).unwrap_or_else(PoisonError::into_inner);
            }
        };
        let (path, size) = job;
        let wanted = q.wanted.lock().unwrap_or_else(PoisonError::into_inner).contains(&(path.clone(), size));
        let event = if wanted {
            let result = decode_scaled(&path, size.max_side());
            Event::Image { path, size, result }
        } else {
            Event::Skipped { path, size }
        };
        emit(tx, wake, pending, event);
    }
}
