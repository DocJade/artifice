mod ping;
mod transform;

use poise::CreateReply;

use crate::captions::caption_media;
use crate::commands::ping::ping;
use crate::job::{Job, JobId, JobType};
use crate::media_helpers;
use crate::media_helpers::download_media;
use crate::media_helpers::find_media;
use crate::media_helpers::Media;
use crate::{Context, Result};

// return the commands in this folder.
pub fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    vec![
        // todo
        ping(),
        register(),
        caption(),
        transform::resize(),
        transform::rotate(),
    ]
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: crate::Context<'_>) -> crate::Result {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

pub async fn handle_job(ctx: Context<'_>, job: Job) -> crate::Result {
    let mut response = ctx
        .reply("Searching for media...".to_string())
        .await?;
    let find_media = find_media(ctx).await?.ok_or("No media found")?;
    response
        .edit(ctx, CreateReply::default().content(format!("Queue Position: {}", ctx.data().queue.len().await)))
        .await?;
    ctx.data().queue.wait(job.id, ctx, &mut response).await?;
    let _permit = ctx.data().job_semaphore.acquire().await?;
    // download the media file
    response
        .edit(ctx, CreateReply::default().content("Downloading..."))
        .await?;
    let media = download_media(find_media).await?.expect("Unable to download media!");
    response
        .edit(ctx, CreateReply::default().content("Processing..."))
        .await?;
    let job = job
        .parts
        .first()
        .ok_or("not implemented")?
        .subparts
        .first()
        .ok_or("not implemented")?
        .to_owned();
    let result: Media = match job {
        JobType::Caption { text, bottom } => caption_media(text, media, bottom, (0, 0, 0), (255, 255, 255))?,
        JobType::Resize { width, height } => media_helpers::resize_media(media, width, height)?,
        JobType::Rotate { rotation } => media_helpers::rotate_and_flip(media, rotation).await?,
    };
    response
        .edit(ctx, CreateReply::default().content("Uploading..."))
        .await?;
    response
        .edit(
            ctx,
            CreateReply::default().content("Done!").attachment(
                poise::serenity_prelude::CreateAttachment::path(
                    result.output_tempfile.unwrap().path,
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
    handle_job(
        ctx,
        Job::new_simple(
            JobType::Caption {
                text: caption.clone(),
                bottom: bottom.unwrap_or(false)
            },
            JobId(ctx.id()),
        ),
    )
    .await
}