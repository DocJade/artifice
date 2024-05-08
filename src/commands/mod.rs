mod ping;
use std::sync::Arc;
use std::time::Duration;

use poise::ReplyHandle;

use crate::commands::ping::ping;
use crate::job::{Job, JobPart, JobType};
use crate::{Context, Result};

// return the commands in this folder.
pub fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    vec![
        // todo
        ping(),
        resize(),
    ]
}

pub async fn poll_queue(
    handle: ReplyHandle<'_>,
    ctx: Context<'_>,
    data: &crate::Data,
    id: u64,
) -> Result {
    let mut timer = tokio::time::interval(Duration::from_secs(1));
    loop {
        if let Some(position) = data.get_position(id).await? {
            handle
                .edit(
                    ctx,
                    poise::CreateReply::default().content({
                        if position == 0 {
                            "front of queue, but job hasn't started yet...".to_string()
                        } else {
                            format!("queue: {}", position)
                        }
                    }),
                )
                .await?;
            timer.tick().await;
        } else {
            return Ok(());
        }
    }
}

#[poise::command(slash_command, prefix_command)]
pub async fn resize(ctx: Context<'_>, width: u16, height: u16, url: String) -> Result {
    let url = url.into();
    let (job, mut rx) = Job::single(JobType::Resize { width, height }, url, ctx.id());
    let job = Arc::new(job);
    ctx.data().queue_push(job.clone()).await?;
    let handle = ctx.reply("working...").await?;
    poll_queue(handle, ctx.clone(), ctx.data(), job.id()).await?;
    Ok(())
}
