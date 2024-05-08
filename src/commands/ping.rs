// return latency

use crate::{Context, Error};
use poise::serenity_prelude::Timestamp;

/// Get the response time of Artifice
#[poise::command(slash_command, guild_only)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    // calculate response time
    // Yes we could just use the amount provided by discord, but this is more accurate.
    
    // let time_now = Timestamp::now().timestamp_millis();
    // let time_sent = ctx.created_at().timestamp_millis();
    // let ping = time_now - time_sent;

    // you win, discord.
    let ping = ctx.ping().await.as_millis();
    let output = format!("I'm online!\n({ping}ms)");
    ctx.say(output).await?;
    Ok(())
}
