mod commands;
mod job;
mod queue;

use poise::serenity_prelude as serenity;
use std::collections::HashSet;

use job::JobId;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T = ()> = std::result::Result<T, Error>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    pub queue: queue::JobQueue,
    pub job_semaphore: tokio::sync::Semaphore,
}

// import the commands

mod captions;
mod ffmpeg_babysitter;
mod media_helpers; // for linting reasons // ditto

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
            owners: HashSet::from([415004648555151380.into()]),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    queue: queue::JobQueue::default(),
                    job_semaphore: tokio::sync::Semaphore::new(2),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
