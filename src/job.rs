#![allow(dead_code)]
use std::sync::Arc;

use poise::serenity_prelude as serenity;

/// a bunch of work that we need to do to respond to a given interaction
#[derive(Debug, Clone)]
pub struct Job {
    /// if you upload multiple images each gets a JobPart
    parts: smallvec::SmallVec<[JobPart; 1]>,
    /// interaction which initiated this job
    interaction: Arc<serenity::CommandInteraction>,
    /// http object used for replying
    http: Arc<serenity::Http>,
}

impl Job {
    pub fn new(
        parts: &[JobPart],
        interaction: Arc<serenity::CommandInteraction>,
        http: Arc<serenity::Http>,
    ) -> Self {
        Self {
            parts: parts.into(),
            interaction,
            http,
        }
    }

    /// get the [`CommandInteraction`] which initiated this job
    pub fn interaction(&self) -> Arc<serenity::CommandInteraction> {
        self.interaction.clone()
    }

    /// get the http object
    pub fn http(&self) -> Arc<serenity::Http> {
        self.http.clone()
    }

    /// iterate over the parts
    pub fn iter(&self) -> std::slice::Iter<JobPart> {
        self.parts.iter()
    }

    /// `position` counts *down*
    pub async fn post_queue_position(&self, position: usize) -> Result<(), crate::Error> {
        self.interaction()
            .create_response(
                self.http(),
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(format!("Queue Position: {}", position)),
                ),
            )
            .await?;
        Ok(())
    }
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.interaction.id == other.interaction.id
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
