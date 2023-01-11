use std::ops::IndexMut;
use std::{env, vec};

use http::StatusCode;
use parsercher::dom::{tag, Tag};
use raxios::{ContentType, Raxios, RaxiosConfig, RaxiosHeaders, RaxiosOptions};
use regex::Regex;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

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
        onmessage(&ctx, &msg).await;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
        .block_on(bot())
}

async fn bot() -> Result<(), Box<dyn std::error::Error>> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = "OTAzMjAyNzk0MDMzNTMyOTY4.G0QBTH.bhaN1EtIKmKdr4Y3APyTxmuL1_hk9-qI1y9cDA";
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

async fn onmessage(ctx: &Context, msg: &Message) -> CommandResult {
    let mut imdb_link = &msg.content;
    
    if imdb_link.starts_with(&"https://letterboxd.com/") {
        let mut p: RaxiosHeaders = RaxiosHeaders::new();
        p.insert(String::from("User-Agent"), String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.63 Safari/537.36"));

        let config: RaxiosConfig = RaxiosConfig {
            timeout_ms: Option::from(5000),
            headers: Option::from(p),
            accept: ContentType::ApplicationXml,
            content_type: ContentType::ApplicationXml,
        };
        //TODO: Complete imdb grab from letterBox
        let response = Raxios::new(&imdb_link, Option::from(config))?;

        let client = Raxios::new("", None)?;
        let data_to_send = ToSend {
            field1: String::from("Hello World"),
        };
        let result = client
            .post::<ToReceive, ToSend>("/endpoint", Some(data_to_send), None)
            .await?;
        if StatusCode::is_success(&result.status){
            let body = result.body.unwrap();
            let dom = parsercher::parse(&body.field1.as_str()).unwrap();
            let tag = parsercher::search_attrs(
                &dom,&vec!["data-tract-action=\"IMDB\""]);
        }
    }
    msg.reply(ctx,imdb_link);
    if msg.content.contains(&"imdb.com") {

        let regex =
            Regex::new(r"^(?:http://|https://)?(?:www\.|m\.)?(?:imdb.com/title/)?(tt[0-9]*)")
                .unwrap();

        let result = regex.captures(&msg.content);

        let link = &result.unwrap()[1];

        let mut headers: RaxiosHeaders = RaxiosHeaders::new();
        headers.insert(String::from("imdbId"), String::from(link));
        let uri = env::var("API_URL").unwrap() + "/suggestions";
        let client = Raxios::new("", None)?;

        let options: RaxiosOptions = RaxiosOptions {
            headers: Option::from(headers),
            accept: Option::from(ContentType::Json),
            content_type: Option::from(ContentType::Json),
            params: None,
            deserialize_body: true,
        };
        // TODO: Implement response to 4sgard movie db
        let response = client
            .post::<u32, &str>(&uri, Option::from(link), Option::from(options))
            .await?;

        let mut reaction_emoji = '🥵';

        println!("{}", link);
        match response.status.as_u16() {
            201 => reaction_emoji = '💾',
            400 => reaction_emoji = '🚨',
            _ => reaction_emoji = '🥵',
        }

        msg.react(ctx, reaction_emoji).await?;
    }
    Ok(())
}
