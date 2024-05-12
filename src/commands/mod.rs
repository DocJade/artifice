mod ping;
use std::sync::Arc;
use std::time::Duration;

use poise::ReplyHandle;

use crate::commands::ping::ping;
use crate::job::{Job, JobId, JobPart, JobType};
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
    id: JobId,
) -> Result {
    let mut timer = tokio::time::interval(Duration::from_secs(1));
    loop {
        if let Some(position) = data.get_position(id).await? {
            if position == 0 {
                return Ok(());
            }
            handle
                .edit(
                    ctx,
                    poise::CreateReply::default().content(format!("queue: {}", position)),
                )
                .await?;
            timer.tick().await;
        } else {
            return Err("Job popped from queue unexpectedly".into());
        }
    }
}

#[poise::command(slash_command, prefix_command)]
pub async fn resize(ctx: Context<'_>, width: u16, height: u16, url: String) -> Result {
    let url = url.into();
    let job = Job::new_simple(JobType::Resize { width, height }, url, JobId(ctx.id()));
    ctx.data().queue_push(job.id).await?;
    let handle = ctx.reply("working...").await?;
    poll_queue(handle.clone(), ctx.clone(), ctx.data(), job.id).await?;
    let _permit = ctx.data().job_semaphore.acquire().await?;
    assert_eq!(Some(job.id), ctx.data().queue_pop().await?);
    // ... do ffmpeg stuff ...
    handle
        .edit(ctx, poise::CreateReply::default().content("Done!"))
        .await?;
    Ok(())
}
