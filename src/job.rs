#![allow(dead_code)]
use std::sync::Arc;

#[derive(Clone, Copy, Debug, bevy_derive::Deref, PartialEq, Eq)]
pub struct JobId(pub u64);

/// a bunch of work that we need to do to respond to a given interaction
#[derive(Debug, Clone, Eq)]
pub struct Job {
    /// id that initiated this job; used for equality
    pub id: JobId,
    /// if you upload multiple images each gets a JobPart
    pub parts: smallvec::SmallVec<[JobPart; 1]>,
}

impl Job {
    pub fn new_simple(ty: JobType, /* url: Arc<str>, */ id: JobId) -> Job {
        Self {
            parts: [JobPart {
                subparts: [ty].into(),
                // download_url: url,
            }]
            .into(),
            id,
        }
    }
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JobPart {
    /// if you chain multiple actions each gets an entry here
    pub subparts: smallvec::SmallVec<[JobType; 1]>,
    // /// URL to download the first image/whatever from
    // pub download_url: Arc<str>, // bad idea! -Doc
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum JobType {
    Resize {
        width: u16,
        height: u16,
    },
    Caption {
        text: String,
    },
    Rotate {
        rotation: crate::media_helpers::Rotation,
    }, // #TODO
}
