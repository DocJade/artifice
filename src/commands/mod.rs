mod ping;
use std::sync::Arc;
use std::time::Duration;

use poise::serenity_prelude::{Builder, EditInteractionResponse};
use poise::{CreateReply, ReplyHandle};

use crate::captions::caption_media;
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
        caption(),
    ]
}

#[poise::command(slash_command, prefix_command)]
pub async fn resize(ctx: Context<'_>, width: u16, height: u16) -> Result {
    let job = Job::new_simple(JobType::Resize { width, height }, JobId(ctx.id()));
    let media = find_media(ctx).await?.ok_or("No media found")?;
    let handle = ctx.data().queue_block(ctx, job.id).await?;
    let _permit = ctx.data().job_semaphore.acquire().await?;
    handle
        .edit(ctx, CreateReply::new().content("Processing..."))
        .await?;
    // </boilerplate>
    let output_media = resize_media(media, width, height)?;
    handle
        .edit(ctx, CreateReply::new().content("Uploading..."))
        .await?;
    ctx.send(
        poise::CreateReply::new().attachment(
            poise::serenity_prelude::CreateAttachment::path(
                output_media.output_tempfile.unwrap().path,
            )
            .await?,
        ),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn caption(ctx: Context<'_>, caption: String) -> Result {
    let job = Job::new_simple(
        JobType::Caption {
            text: caption.clone(),
        },
        JobId(ctx.id()),
    );
    let media = find_media(ctx).await?.ok_or("No media found")?;
    let handle = ctx.data().queue_block(ctx, job.id).await?;
    let _permit = ctx.data().job_semaphore.acquire().await?;
    handle
        .edit(ctx, CreateReply::new().content("Processing..."))
        .await?;
    // </boilerplate>
    let output_media = caption_media(caption, media, false, (0, 0, 0), (255, 255, 255))?;
    handle
        .edit(ctx, CreateReply::new().content("Uploading..."))
        .await?;
    ctx.send(
        poise::CreateReply::new().attachment(
            poise::serenity_prelude::CreateAttachment::path(
                output_media.output_tempfile.unwrap().path,
            )
            .await?,
        ),
    )
    .await?;
    Ok(())
}
