mod commands;
mod fursona;

use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use std::{
    collections::HashMap,
    env::var,
    sync::{Arc, Mutex},
};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
struct Data {
    fursonas: Arc<Mutex<HashMap<serenity::UserId, fursona::Fursona>>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    let token = var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let guild_id =
        serenity::GuildId::new(var("GUILD_ID").expect("missing GUILD_ID").parse().unwrap());

    let options = poise::FrameworkOptions {
        on_error: |error| Box::pin(on_error(error)),
        commands: vec![commands::ping(), commands::fursona()],
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .options(options)
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
                    .await?;

                Ok(Data {
                    fursonas: Arc::new(Mutex::new(HashMap::new())),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
