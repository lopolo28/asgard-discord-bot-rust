use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;

mod events;
use events::asgard_events::onmessage;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ToReceive {
    field1: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ToSend {
    field1: String,
}

#[group]
#[commands(ping)]
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
        }
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
