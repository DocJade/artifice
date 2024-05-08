#![allow(dead_code)]
use poise::serenity_prelude as serenity;

#[derive(Debug)]
pub struct Job {
    typ: JobType,
    interaction: serenity::Interaction,
    source_url: String,
}

#[derive(Debug)]
pub enum JobType {
    Resize { width: u16, height: u16 },
    Caption { text: String },
    // #TODO
}

impl Job {
    pub async fn respond(&self) {
        // self.interaction.
    }
}
