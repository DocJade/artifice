mod commands;
mod job;

use poise::serenity_prelude as serenity;
use std::collections::{HashSet, VecDeque};
use tracing::info;

use job::Job;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T = ()> = std::result::Result<T, Error>;

// Custom user data passed to all command functions
pub struct Data {
    pub job_queue: tokio::sync::RwLock<VecDeque<Job>>,
}

// import the commands

mod ffmpeg_babysitter;
mod media_helpers; // for linting reasons // ditto

impl Default for Data {
    fn default() -> Self {
        Self {
            job_queue: Default::default(),
        }
    }
}

impl Data {
    pub async fn queue_push(&self, job: Job) -> crate::Result {
        let mut lock = self.job_queue.write().await;
        lock.push_back(job);
        Ok(())
    }
    pub async fn queue_pop(&self) -> crate::Result<Option<Job>> {
        let mut lock = self.job_queue.write().await;
        Ok(lock.pop_front())
    }
    pub async fn get_position(&self, other: &Job) -> crate::Result<Option<usize>> {
        let lock = self.job_queue.read().await;
        for (i, job) in lock.iter().enumerate() {
            if job == other {
                return Ok(Some(i));
            }
        }
        Ok(None)
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
