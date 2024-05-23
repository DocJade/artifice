use std::{
    collections::VecDeque,
    sync::{Arc, Weak},
};

use poise::{CreateReply, ReplyHandle};

type JobHandle = crate::JobId;

pub struct JobQueue {
    queue: std::sync::Arc<tokio::sync::RwLock<VecDeque<JobHandle>>>,
}

impl Default for JobQueue {
    fn default() -> Self {
        use tokio::sync::Semaphore;
        Self {
            queue: Default::default(),
        }
    }
}

impl JobQueue {
    /// add `job` to the queue, post updates about the position, and block till it's done
    pub async fn wait<'ctx>(
        &self,
        job: JobHandle,
        ctx: crate::Context<'ctx>,
        message: &mut ReplyHandle<'ctx>,
    ) -> crate::Result {
        self.queue.write().await.push_back(job);
        let _guard = QueueGuard {
            queue: Arc::downgrade(&self.queue),
            job: job,
        };
        loop {
            if let Some(position) = self.poll(job).await {
                message
                    .edit(
                        ctx,
                        CreateReply::default().content(format!("Queue Position: {}", position)),
                    )
                    .await?;
                if position == 0 {
                    return Ok(());
                }
            } else {
                return Err("Unexpectedly popped".into());
            }
        }
    }

    /// get position of `job`, or None if not in the queue
    async fn poll(&self, job: JobHandle) -> Option<usize> {
        let lock = self.queue.read().await;
        for (i, other) in lock.iter().enumerate() {
            if *other == job {
                return Some(i);
            }
        }
        None
    }

    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }
}

/// ensures that the job is removed when dropped
struct QueueGuard {
    queue: Weak<tokio::sync::RwLock<VecDeque<JobHandle>>>,
    job: JobHandle,
}

impl Drop for QueueGuard {
    fn drop(&mut self) {
        if let Some(queue) = self.queue.upgrade() {
            let job = self.job;
            tokio::spawn(async move {
                let mut lock = queue.write().await;
                lock.retain(|other| *other != job);
            });
        }
    }
}
