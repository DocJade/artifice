use super::handle_job;
use crate::{Context, Job, JobId, JobType, Result};

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
