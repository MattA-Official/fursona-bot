use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// A simple ping command that responds with "Pong!" and the bot's latency.
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let latency = ctx.ping().await;

    let response = format!("Pong! Latency: `{:?}`", latency);

    ctx.say(response).await?;

    Ok(())
}

/// The parent command for fursona-related commands.
#[poise::command(
    slash_command,
    subcommands("view_fursona", "create_fursona", "random_fursona")
)]
pub async fn fursona(_: Context<'_>) -> Result<(), Error> {
    // This will never be called, because `subcommand_required` parameter is set
    Ok(())
}

/// A command to view a user's fursona.
#[poise::command(slash_command, rename = "view")]
pub async fn view_fursona(
    ctx: Context<'_>,
    #[description = "User to view fursona of"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let fursona = {
        let fursonas = ctx.data().fursonas.lock().unwrap();
        let u = user.as_ref().unwrap_or_else(|| ctx.author());

        fursonas.get(&u.id).cloned()
    };

    if let Some(user) = user {
        if let Some(fursona) = fursona {
            let response = format!(
                "{}'s fursona is a {} with a {} body type, {} markings, and the following accessories: {:?}. Their personality is: {}",
                user.name,
                fursona.species,
                fursona.body_type,
                fursona.markings,
                fursona.accessories,
                fursona.personality,
            );

            ctx.say(response).await?;
        } else {
            ctx.say(format!("{} doesn't have a fursona set!", user.name))
                .await?;
        }
    } else {
        if let Some(fursona) = fursona {
            let response = format!(
                "Your fursona is a {} with a {} body type, {} markings, and the following accessories: {:?}. Their personality is: {}",
                fursona.species,
                fursona.body_type,
                fursona.markings,
                fursona.accessories,
                fursona.personality,
            );

            ctx.say(response).await?;
        } else {
            ctx.say("You don't have a fursona set!").await?;
        }
    }

    Ok(())
}

/// A command to create a fursona.
#[poise::command(slash_command, rename = "create")]
pub async fn create_fursona(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// A command to generate a random fursona.
#[poise::command(slash_command, rename = "random")]
pub async fn random_fursona(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
