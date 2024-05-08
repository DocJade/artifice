#![allow(dead_code)]
use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tokio::sync::mpsc;

/// a bunch of work that we need to do to respond to a given interaction
#[derive(Debug, Clone)]
pub struct Job {
    /// if you upload multiple images each gets a JobPart
    parts: smallvec::SmallVec<[JobPart; 1]>,
    /// id that initiated this job; used for equality
    id: u64,
    /// sender that the worker task will use to communicate with the command task
    /// #TODO `JobMessage` or something
    tx: mpsc::Sender<JobMessage>,
}

impl Job {
    /// returns a new Job and the receiver half of the channel
    pub fn new(parts: &[JobPart], id: u64) -> (Self, mpsc::Receiver<JobMessage>) {
        let (tx, rx) = mpsc::channel(3);
        (
            Self {
                parts: parts.into(),
                id,
                tx,
            },
            rx,
        )
    }

    pub fn single(ty: JobType, url: Arc<str>, id: u64) -> (Self, mpsc::Receiver<JobMessage>) {
        Self::new(
            &[JobPart {
                subparts: [ty].into(),
                download_url: url,
            }],
            id,
        )
    }

    /// iterate over the parts
    pub fn iter(&self) -> std::slice::Iter<JobPart> {
        self.parts.iter()
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Job {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JobPart {
    /// if you chain multiple actions each gets an entry here
    pub subparts: smallvec::SmallVec<[JobType; 1]>,
    /// URL to download the first image/whatever from
    pub download_url: Arc<str>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JobType {
    Resize { width: u16, height: u16 },
    Caption { text: String },
    // #TODO
}

pub enum JobMessage {
    /// nonspecific information which will just be sent as a discord message.
    /// mostly for debugging
    Information(String),
}
