use crate::{Context, Error};

/// A simple ping command that responds with "Pong!" and the bot's latency.
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let latency = ctx.ping().await;

    let response = format!("Pong! Latency: `{:?}`", latency);

    ctx.say(response).await?;

    Ok(())
}
