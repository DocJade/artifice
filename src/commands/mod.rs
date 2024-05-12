mod ping;
use std::sync::Arc;
use std::time::Duration;

use poise::serenity_prelude::{Builder, EditInteractionResponse};
use poise::ReplyHandle;

use crate::commands::ping::ping;
use crate::job::{Job, JobId, JobPart, JobType};
use crate::media_helpers::{find_media, resize_media};
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
pub async fn resize(ctx: Context<'_>, width: u16, height: u16) -> Result {
    // TODO: show place in queue.
    let job = Job::new_simple(JobType::Resize { width, height }, JobId(ctx.id()));
    ctx.data().queue_push(job.id).await?;
    let handle = ctx.reply("working...").await?;
    poll_queue(handle.clone(), ctx.clone(), ctx.data(), job.id).await?;
    let _permit = ctx.data().job_semaphore.acquire().await?;
    assert_eq!(Some(job.id), ctx.data().queue_pop().await?);

    // get the media
    // TODO: more concrete error handling
    let input_media = find_media(ctx).await?;
    // did we get some
    if input_media.is_none() {
        // nope!
        // TODO: move this into its own function as well
        handle
        .edit(ctx, poise::CreateReply::default().content("No media found!"))
        .await?;
    return Ok(())
}

    // we've got media! resize that mf

    // TODO: more concrete error handling
    let output_media = resize_media(input_media.unwrap(), width, height)?;
    
    // make the attachment
    // TODO: more concrete error handling
    let attachment = poise::serenity_prelude::CreateAttachment::path(output_media.output_tempfile.unwrap().path).await?;

    // reply!
    // TODO: how do we update the original message with media?
    // maybe something to do with EditInteractionResponse::new_attachment())
    ctx.reply("Uploading...").await?.edit(ctx, poise::CreateReply::default().content("").attachment(attachment)).await?;
    Ok(())
}
