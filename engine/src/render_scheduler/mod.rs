//! GPU/render job scheduler — priority queue (Rule #18).

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::sync::{Arc, Condvar, Mutex};

use uuid::Uuid;

use crate::device_profiler::profile::ThermalLevel;
use crate::error::{CinemaError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Playback = 4,
    Preview = 3,
    Proxy = 2,
    Export = 1,
}

#[derive(Debug, Clone)]
pub struct ScheduledJob {
    pub id: Uuid,
    pub priority: JobPriority,
    pub label: String,
}

struct JobEntry {
    job: ScheduledJob,
    order: u64,
}

impl PartialEq for JobEntry {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

impl Eq for JobEntry {}

impl PartialOrd for JobEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JobEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.job.priority.cmp(&other.job.priority) {
            Ordering::Equal => other.order.cmp(&self.order),
            o => o,
        }
    }
}

struct SchedulerInner {
    queue: BinaryHeap<JobEntry>,
    sequence: u64,
    paused_export: bool,
    paused_proxy: bool,
}

pub struct RenderScheduler {
    inner: Arc<(Mutex<SchedulerInner>, Condvar)>,
}

impl RenderScheduler {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(SchedulerInner {
                    queue: BinaryHeap::new(),
                    sequence: 0,
                    paused_export: false,
                    paused_proxy: false,
                }),
                Condvar::new(),
            )),
        }
    }

    pub fn enqueue(&self, job: ScheduledJob) -> Result<()> {
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().map_err(|e| CinemaError::Storage(format!("scheduler lock: {e}")))?;

        if inner.paused_export && job.priority == JobPriority::Export {
            return Ok(());
        }
        if inner.paused_proxy && job.priority == JobPriority::Proxy {
            return Ok(());
        }

        inner.sequence += 1;
        inner.queue.push(JobEntry {
            job,
            order: inner.sequence,
        });
        cvar.notify_one();
        Ok(())
    }

    pub fn pop_highest(&self) -> Option<ScheduledJob> {
        let (lock, _) = &*self.inner;
        lock.lock().ok()?.queue.pop().map(|e| e.job)
    }

    pub fn adapt_to_thermal(&self, thermal: ThermalLevel) {
        if let Ok(mut inner) = self.inner.0.lock() {
            inner.paused_export = thermal >= ThermalLevel::Hot;
            inner.paused_proxy = thermal == ThermalLevel::Critical;
        }
    }

    pub fn pending_count(&self) -> usize {
        self.inner.0.lock().map(|i| i.queue.len()).unwrap_or(0)
    }
}

impl Default for RenderScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_beats_export() {
        let sched = RenderScheduler::new();
        sched
            .enqueue(ScheduledJob {
                id: Uuid::new_v4(),
                priority: JobPriority::Export,
                label: "export".into(),
            })
            .unwrap();
        sched
            .enqueue(ScheduledJob {
                id: Uuid::new_v4(),
                priority: JobPriority::Playback,
                label: "playback".into(),
            })
            .unwrap();

        let first = sched.pop_highest().unwrap();
        assert_eq!(first.priority, JobPriority::Playback);
    }

    #[test]
    fn thermal_pauses_export() {
        let sched = RenderScheduler::new();
        sched.adapt_to_thermal(ThermalLevel::Hot);
        sched
            .enqueue(ScheduledJob {
                id: Uuid::new_v4(),
                priority: JobPriority::Export,
                label: "export".into(),
            })
            .unwrap();
        assert_eq!(sched.pending_count(), 0);
    }
}
