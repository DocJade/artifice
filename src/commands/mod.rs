mod ping;

use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::captions::caption_media;
use crate::commands::ping::ping;
use crate::job::{Job, JobId, JobType};
use crate::media_helpers;
use crate::media_helpers::find_media;
use crate::media_helpers::Media;
use crate::{Context, Result};

// return the commands in this folder.
pub fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    vec![
        // todo
        ping(),
        register(),
        resize(),
        caption(),
        rotate(),
    ]
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: crate::Context<'_>) -> crate::Result {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

pub async fn handle_job(ctx: Context<'_>, job: Job) -> crate::Result {
    let media = find_media(ctx).await?.ok_or("No media found")?;
    let mut response = ctx
        .reply(format!("Queue Position: {}", ctx.data().queue.len().await))
        .await?;
    ctx.data().queue.wait(job.id, ctx, &mut response).await?;
    let _permit = ctx.data().job_semaphore.acquire().await?;
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
        JobType::Caption { text } => caption_media(text, media, false, (0, 0, 0), (255, 255, 255))?,
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

//  Video, Gif, Image
/// Rotate media.
#[poise::command(slash_command, prefix_command)]
pub async fn rotate(
    ctx: Context<'_>,
    #[description = "What angle?"] choice: crate::media_helpers::Rotation,
) -> Result {
    handle_job(
        ctx,
        Job::new_simple(JobType::Rotate { rotation: choice }, JobId(ctx.id())),
    )
    .await
}

/// Resize media.
#[poise::command(slash_command, prefix_command)]
pub async fn resize(
    ctx: Context<'_>,
    #[description = "How tall?"] height: u16,
    #[description = "How wide?"] width: Option<u16>,
) -> Result {
    handle_job(
        ctx,
        Job::new_simple(
            JobType::Resize {
                width: width.unwrap_or(0),
                height,
            },
            JobId(ctx.id()),
        ),
    )
    .await
}

/// Add a caption to media.
#[poise::command(slash_command)]
pub async fn caption(
    ctx: Context<'_>,
    #[description = "Text to add"] caption: String,
    #[description = "Do you want the caption on the bottom?"] _bottom: Option<bool>,
) -> Result {
    handle_job(
        ctx,
        Job::new_simple(
            JobType::Caption {
                text: caption.clone(),
            },
            JobId(ctx.id()),
        ),
    )
    .await
}