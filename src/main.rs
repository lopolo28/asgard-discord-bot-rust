use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;

mod events;
use crate::events::asgard_events::on_message_twitter;
use events::asgard_events::onmessage;

#[group]
#[commands(about, ping)]
struct General;

#[group]
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let envid = env::var("CHANNEL_ID").expect("CHANNEL_ID not found");
        let moviechannel = str::parse::<u64>(&envid).expect("Unable to parse CHANNEL_ID to u64");
        if msg.channel_id.0 == moviechannel {
            onmessage(&ctx, &msg).await;
        } else {
            on_message_twitter(&ctx, &msg).await;
        }
    }
    async fn ready(&self, ctx: Context, _: serenity::model::prelude::Ready) {
        ctx.set_activity(serenity::model::gateway::Activity::watching("Stuff"))
            .await;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .enable_all()
        .build()
        .unwrap()
        .block_on(bot())
}

async fn bot() -> Result<(), Box<dyn std::error::Error>> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!").ignore_bots(true)) // set the bot's prefix to "!"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN not found");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(err) = client.start().await {
        println!("An error occurred while running the client: {:?}", err);
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}
#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(
        ctx,
        r#"Here is source code: https://github.com/4sgard-dev/asgard-discord-bot-rust
💾 - Bot Happy
🤖 - Bot Error
🚨 - Server Error
🥵 - Duplicate Suggestion"#,
    )
    .await?;
    Ok(())
}