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
    #[description = "How tall?"]
    #[min = 10]
    #[max = 8000]
    height: u16,
    #[description = "How wide?"]
    #[min = 10]
    #[max = 8000]
    width: Option<u16>,
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
