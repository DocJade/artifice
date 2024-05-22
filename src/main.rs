mod commands;
mod job;

use poise::{serenity_prelude as serenity, CreateReply};
use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

use job::JobId;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T = ()> = std::result::Result<T, Error>;

// Custom user data passed to all command functions
pub struct Data {
    pub job_queue: tokio::sync::RwLock<VecDeque<JobId>>,
    pub job_semaphore: tokio::sync::Semaphore,
}

// import the commands

mod captions;
mod ffmpeg_babysitter;
mod media_helpers; // for linting reasons // ditto

impl Default for Data {
    fn default() -> Self {
        Self {
            job_queue: Default::default(),
            job_semaphore: tokio::sync::Semaphore::new(2),
        }
    }
}

impl Data {
    pub async fn queue_push(&self, job: JobId) -> crate::Result {
        let mut lock = self.job_queue.write().await;
        lock.push_back(job);
        Ok(())
    }
    pub async fn queue_pop(&self) -> crate::Result<Option<JobId>> {
        let mut lock = self.job_queue.write().await;
        Ok(lock.pop_front())
    }
    pub async fn get_position(&self, other: JobId) -> crate::Result<Option<usize>> {
        let lock = self.job_queue.read().await;
        for (i, job) in lock.iter().enumerate() {
            if *job == other {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// blocks till `job` is at the front of the queue;
    /// posts a message and edits it with the queue position.
    /// returns the `ReplyHandle` i was repeatedly editing
    pub async fn queue_block<'ctx>(
        &self,
        ctx: crate::Context<'ctx>,
        job: JobId,
    ) -> crate::Result<poise::ReplyHandle<'ctx>> {
        self.queue_push(job).await?;
        let mut timer = tokio::time::interval(std::time::Duration::from_secs(1));
        let handle = ctx
            .reply(Self::format_queue_position(
                self.job_queue.read().await.len(),
            ))
            .await?;
        loop {
            let position = self.get_position(job).await?;
            if let Some(position) = position {
                handle
                    .edit(
                        ctx,
                        CreateReply::new().content(Self::format_queue_position(position)),
                    )
                    .await?;
                timer.tick().await;
                if position == 0 {
                    break;
                }
            } else {
                return Err("Job removed from queue unexpectedly!".into());
            }
        }
        if let Some(popped) = self.queue_pop().await? {
            if popped == job {
                Ok(handle)
            } else {
                Err(
                    "Was at the front of the queue just a second ago, but wrong job was popped!"
                        .into(),
                )
            }
        } else {
            Err("Was at the front of the queue just a second ago, but queue is now empty!".into())
        }
    }

    pub fn format_queue_position(position: usize) -> String {
        format!("Queue Position: {}", position)
    }
}

type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // pull in env variables
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive("artifice=info".parse().unwrap())
                .with_default_directive("poise=warn".parse().unwrap())
                .with_default_directive("serenity=warn".parse().unwrap())
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    tracing::info!("Artifice is starting...");
    // Automatically set up FFMPEG
    ffmpeg_sidecar::download::auto_download().unwrap();
    let token = std::env::var("TOKEN").expect("missing $TOKEN");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;
    let framework = poise::Framework::<Data, Error>::builder()
        .options(poise::FrameworkOptions {
            commands: commands::commands(),
            prefix_options: {
                poise::PrefixFrameworkOptions {
                    mention_as_prefix: true,
                    ..Default::default()
                }
            },
            owners: HashSet::from([397226869495169037.into()]),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::default())
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
