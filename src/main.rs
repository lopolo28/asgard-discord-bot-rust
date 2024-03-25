use serenity::all::{CreateAttachment, GuildId};
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Configuration, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::cmp::min;
use std::env;

mod events;
use crate::events::asgard_events::on_message_twitter;
use events::asgard_events::onmessage;

#[group]
#[commands(about, ping)]
struct General;

#[group]
#[commands(swap, moveemoji, countemoji, emojiposition)]
struct EmojiControl;

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
        on_message_twitter(&ctx, &msg).await;
    }
    async fn ready(&self, ctx: Context, _: serenity::model::prelude::Ready) {
        ctx.set_activity(Some(serenity::gateway::ActivityData::watching("Stuff")));
    }
}

struct PbClient {
    client: pocketbase_sdk::client::Client<pocketbase_sdk::client::Auth>,
}

impl TypeMapKey for PbClient {
    type Value = pocketbase_sdk::client::Client<pocketbase_sdk::client::Auth>;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
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
    framework.group_add(&EMOJICONTROL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN not found");
    let pb_uri = env::var("PB_URI").expect("PB_URI not found");
    let pb_ident = env::var("PB_IDENT").expect("PB_IDENT not found");
    let pb_secret = env::var("PB_SECRET").expect("PB_SECRET not found");

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let pocketbase_client = match pocketbase_sdk::client::Client::new(pb_uri.as_str())
        .auth_with_password("users", pb_ident.as_str(), pb_secret.as_str())
    {
        Ok(client) => {
            println!("Connected to pocketbase");
            client
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return Ok(());
        }
    };

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .type_map_insert::<PbClient>(pocketbase_client)
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
#[command]
#[num_args(2)]
async fn moveemoji(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild_id.unwrap();

    let mut emojis = guild.emojis(ctx).await.unwrap();

    let args: Vec<&str> = msg.content.split(' ').skip(1).collect();

    let indexies: Vec<&serenity::all::Emoji> = emojis
        .iter()
        .filter(|emoji| emoji.to_string() == args[0])
        .collect();

    if indexies.len() != 1 {
        msg.reply(ctx, "Emoji not found on this server").await?;
        return Ok(());
    }

    let index0 = match emojis.iter().position(|emoji| emoji.id == indexies[0].id) {
        Some(index) => index,
        None => {
            msg.reply(ctx, "1st emoji is not on this server").await?;
            return Ok(());
        }
    };

    let index1 = match args[1].parse::<usize>() {
        Ok(index) => index - 1,
        Err(_) => {
            msg.reply(ctx, "Index must be positive 1..=Number of emojis")
                .await?;
            return Ok(());
        }
    };

    if index1 >= emojis.len() {
        msg.reply(ctx, "Cannot move emoji outside of bounds")
            .await?;
        return Ok(());
    }
    if index0 == index1 {
        msg.reply(ctx, "Cannot move emoji to same position").await?;
        return Ok(());
    }
    reupload_emoji(ctx, msg, &guild, index0, index1, &mut emojis).await
}

#[command]
#[description = "Swap two emojis"]
#[num_args(2)]
async fn swap(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild_id.unwrap();

    let mut emojis = guild.emojis(ctx).await.unwrap();

    let args: Vec<&str> = msg.content.split(' ').skip(1).collect();

    let indexies: Vec<&serenity::all::Emoji> = emojis
        .iter()
        .filter(|emoji| emoji.to_string() == args[0] || emoji.to_string() == args[1])
        .collect();

    if indexies.len() != 2 {
        msg.reply(ctx, "Emoji not found on this server").await?;
        return Ok(());
    }

    let index0 = match emojis.iter().position(|emoji| emoji.id == indexies[0].id) {
        Some(index) => index,
        None => {
            msg.reply(ctx, "1st emoji is not on this server").await?;
            return Ok(());
        }
    };

    let index1 = match emojis.iter().position(|emoji| emoji.id == indexies[1].id) {
        Some(index) => index,
        None => {
            msg.reply(ctx, "2nd emoji is not on this server").await?;
            return Ok(());
        }
    };
    reupload_emoji(ctx, msg, &guild, index0, index1, &mut emojis).await
}

async fn reupload_emoji(
    ctx: &Context,
    msg: &Message,
    guild: &GuildId,
    index0: usize,
    index1: usize,
    emojis: &mut Vec<serenity::all::Emoji>,
) -> CommandResult {
    emojis.swap(index0, index1);

    for emoji in emojis[min(index0, index1)..].iter() {
        guild.delete_emoji(ctx, emoji.id).await?;
        let bytes = reqwest::get(emoji.url()).await?.bytes().await?;

        let attachment = CreateAttachment::bytes(bytes, emoji.name.clone());

        match guild
            .create_emoji(ctx, &emoji.name, attachment.to_base64().as_str())
            .await
        {
            Ok(_) => println!("Created: {}", emoji.name),
            Err(err) => {
                println!("Error: {:?}", err);
                msg.reply(ctx, err.to_string()).await?;
                return Ok(());
            }
        }
    }
    msg.react(ctx, '💾').await?;
    Ok(())
}

#[command]
async fn countemoji(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(
        ctx,
        msg.guild_id
            .unwrap()
            .emojis(ctx)
            .await
            .unwrap()
            .len()
            .to_string(),
    )
    .await?;
    Ok(())
}

#[command]
#[num_args(1)]
async fn emojiposition(ctx: &Context, msg: &Message) -> CommandResult {
    let emojis = msg.guild_id.unwrap().emojis(ctx).await.unwrap();

    let args = msg.content.split(' ').last().unwrap();

    let index = emojis
        .iter()
        .map(|emoji| emoji.to_string())
        .position(|emoji| emoji == args);
    match index {
        Some(index) => {
            msg.reply(ctx, (index + 1).to_string()).await?;
        }
        None => {
            msg.reply(ctx, "Emoji not found").await?;
        }
    }

    Ok(())
}
