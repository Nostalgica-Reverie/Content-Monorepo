use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

const CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceLevel {
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceRecord {
    pub sequence: u64,
    pub level: TraceLevel,
    pub module: String,
    pub message: String,
    pub origin: String,
    pub platform_code: Option<i32>,
}

static RECORDS: LazyLock<Mutex<VecDeque<TraceRecord>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(CAPACITY)));
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
static DROPPED: AtomicU64 = AtomicU64::new(0);

pub fn trace(
    level: TraceLevel,
    module: impl Into<String>,
    message: impl Into<String>,
    origin: impl Into<String>,
    platform_code: Option<i32>,
) {
    let record = TraceRecord {
        sequence: SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1,
        level,
        module: module.into(),
        message: message.into(),
        origin: origin.into(),
        platform_code,
    };
    if let Ok(mut records) = RECORDS.lock() {
        if records.len() == CAPACITY {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        } else {
            records.push_back(record);
        }
    } else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn trace_drain() -> Vec<TraceRecord> {
    RECORDS
        .lock()
        .map(|mut records| records.drain(..).collect())
        .unwrap_or_default()
}

pub fn trace_dropped() -> u64 {
    DROPPED.load(Ordering::Relaxed)
}
