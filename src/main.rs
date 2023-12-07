use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Configuration, StandardFramework};
use serenity::gateway::ActivityData;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::env;

mod events;
use events::asgard_events::onmessage;

// #[derive(serde::Serialize, serde::Deserialize, Debug)]
// struct ToReceive {
//     field1: String,
// }

// #[derive(serde::Serialize, serde::Deserialize, Debug)]
// struct ToSend {
//     field1: String,
// }

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
        if msg.channel_id == moviechannel {
            onmessage(&ctx, &msg).await;
        }
    }
    async fn ready(&self, ctx: Context, _: serenity::model::prelude::Ready) {
        ctx.set_activity(Some(ActivityData::competing("YOUR MOM")));
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
    let mut framework = StandardFramework::new();
    framework.configure(
        Configuration::new()
            .with_whitespace(true)
            .prefix("!") // set the bot's prefix to "!"
            .ignore_bots(true),
    );
    framework.group_add(&GENERAL_GROUP);

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
        "Here is source code: https://github.com/4sgard-dev/asgard-discord-bot-rust\r\n💾 - Bot Happy\r\n🤖 - Bot Error\r\n🚨 - Server Error\r\n🥵 - Duplicate Suggestion",
    )
    .await?;
    Ok(())
}