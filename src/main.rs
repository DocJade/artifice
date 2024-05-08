use std::collections::HashSet;

use poise::serenity_prelude as serenity;
use tracing::info;

type Error = Box<dyn std::error::Error + Send + Sync>;

// Custom user data passed to all command functions
#[derive(Debug)]
pub struct Data {
    pub job_tx: flume::Sender<Job>,
    pub job_rx: flume::Receiver<Job>,
}

impl Default for Data {
    fn default() -> Self {
        let (job_tx, job_rx) = flume::bounded(100);
        Self { job_tx, job_rx }
    }
}

impl Data {
    pub async fn queue_push(&mut self, item: Job) -> Result<(), Error> {
        self.job_tx.send_async(item).await?;
        Ok(())
    }
    pub async fn queue_pop(&mut self) -> Result<Job, Error> {
        Ok(self.job_rx.recv_async().await?)
    }
}

#[derive(Debug)]
pub enum Job {
    // #TODO
}

type Context<'a> = poise::Context<'a, Data, Error>;

// import the commands
mod commands;

#[tokio::main]
async fn main() {
    info!("Artifice is starting...");
    // pull in env variables
    dotenv::dotenv().ok();
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
