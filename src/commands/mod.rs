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
        rotate()
    ]
}

//  Video, Gif, Image
/// Rotate media.
#[poise::command(slash_command, prefix_command)]
pub async fn rotate(
    ctx: Context<'_>,
    #[description = "What angle?"] choice: crate::media_helpers::Rotation,
) -> Result {
    // rotation time!
    let job = Job::new_simple(
        JobType::Rotate {
            rotation: choice
        },
        JobId(ctx.id()),
    );
    let media = find_media(ctx).await?.ok_or("No media found")?;
    let handle = ctx.data().queue_block(ctx, job.id).await?;
    let _permit = ctx.data().job_semaphore.acquire().await?;
    handle
        .edit(ctx, CreateReply::default().content("Processing..."))
        .await?;
    // </boilerplate>
    let output_media = crate::media_helpers::rotate_and_flip(media, choice).await?;
    handle
        .edit(ctx, CreateReply::default().content("Uploading..."))
        .await?;

    handle
        .edit(
            ctx,
            CreateReply::default().content("Done!").attachment(
                poise::serenity_prelude::CreateAttachment::path(
                    output_media.output_tempfile.unwrap().path,
                )
                .await?,
            ),
        )
        .await?;
    Ok(())
}


/// Resize media.
#[poise::command(slash_command, prefix_command)]
pub async fn resize(
    ctx: Context<'_>,
    #[description = "How tall?"] height: u16,
    #[description = "How wide?"] width: Option<u16>,
) -> Result {
    let job = Job::new_simple(
        JobType::Resize {
            width: width.unwrap_or(0),
            height,
        },
        JobId(ctx.id()),
    );
    let media = find_media(ctx).await?.ok_or("No media found")?;
    let handle = ctx.data().queue_block(ctx, job.id).await?;
    let _permit = ctx.data().job_semaphore.acquire().await?;
    handle
        .edit(ctx, CreateReply::default().content("Processing..."))
        .await?;
    // </boilerplate>
    let output_media = resize_media(media, width.unwrap_or(0), height)?;
    handle
        .edit(ctx, CreateReply::default().content("Uploading..."))
        .await?;

    handle
        .edit(
            ctx,
            CreateReply::default().content("Done!").attachment(
                poise::serenity_prelude::CreateAttachment::path(
                    output_media.output_tempfile.unwrap().path,
                )
                .await?,
            ),
        )
        .await?;
    Ok(())
}

/// Add a caption to media.
#[poise::command(slash_command)]
pub async fn caption(
    ctx: Context<'_>,
    #[description = "Text to add"] caption: String,
    #[description = "Do you want the caption on the bottom?"] bottom: Option<bool>,
) -> Result {
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
        .edit(ctx, CreateReply::default().content("Processing..."))
        .await?;
    // </boilerplate>
    let output_media = caption_media(
        caption,
        media,
        bottom.unwrap_or(false),
        (0, 0, 0),
        (255, 255, 255),
    )?;

    // tell the user we are uploading
    handle
        .edit(ctx, CreateReply::default().content("Uploading..."))
        .await?;

    // actually upload the file
    handle
        .edit(
            ctx,
            CreateReply::default().content("Done!").attachment(
                poise::serenity_prelude::CreateAttachment::path(
                    output_media.output_tempfile.unwrap().path,
                )
                .await?,
            ),
        )
        .await?;

    Ok(())
}
