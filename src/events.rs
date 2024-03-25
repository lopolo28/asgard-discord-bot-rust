pub mod asgard_events {
    use parsercher::{dom::Tag, parse};
    use regex::Regex;
    use serde::Serialize;
    use serenity::builder::{Builder, EditMessage};
    use serenity::client::Context;
    use serenity::model::prelude::Message;

    use crate::PbClient;

    #[allow(non_snake_case)]
    #[derive(Debug, Clone, Serialize)]
    struct NewSuggestion {
        imdbId: String,
        createdByDiscordId: String,
        suggestion: bool,
        deleted: bool,
    }

    // array of base urls to replace
    static TWITTER_BASE_URLS: [&str; 2] = ["https://twitter.com/", "https://x.com/"];
    static REPLACE_BASE_URL: &str = "https://vxtwitter.com/";

    pub async fn on_message_twitter(ctx: &Context, msg: &Message) {
        let mut replaced_msg = msg.content.clone();

        if TWITTER_BASE_URLS
            .iter()
            .find(|i| replaced_msg.contains(*i))
            .is_some()
        {
            replaced_msg = replaced_msg.replace(
                TWITTER_BASE_URLS
                    .iter()
                    .find(|&i| replaced_msg.contains(i))
                    .unwrap(),
                REPLACE_BASE_URL,
            );
            msg.reply(ctx, replaced_msg).await.ok();

            match EditMessage::new()
                .suppress_embeds(true)
                .execute(ctx, (msg.channel_id, msg.id, Some(msg.author.id)))
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }

    pub async fn onmessage(ctx: &Context, msg: &Message) {
        if msg.content.starts_with("https://letterboxd.com/") {
            let response = reqwest::get(&msg.content)
                .await
                .expect("Letterboxd scrapping: No response recieved");
            if response.status().is_success() {
                let body = match response.text().await {
                    Ok(body) => body,
                    Err(e) => {
                        eprintln!("{}", e);
                        msg.react(ctx, '🤖').await.ok();
                        return;
                    }
                };

                let dom = match parse(&body) {
                    Ok(dom) => dom,
                    Err(e) => {
                        match find_imdb_url(&body).await {
                            Ok(url) => {
                                imdb(ctx, msg, url).await;
                                return;
                            }
                            Err(e) => {
                                print!("{}", e);
                            }
                        };
                        eprintln!("{}", e);
                        msg.react(ctx, '🤖').await.ok();
                        return;
                    }
                };

                let mut needle = Tag::new("a");
                needle.set_attr("data-track-action", "IMDb");
                if let Some(tags) = parsercher::search_tag(&dom, &needle) {
                    imdb(
                        ctx,
                        msg,
                        tags.last()
                            .expect("Empty")
                            .get_attr("href")
                            .expect("Error accured during unwraping")
                            .as_str(),
                    )
                    .await;
                }
            }
        } else if msg.content.contains("imdb.com") {
            imdb(ctx, msg, &msg.content).await;
        }
    }
    async fn imdb(ctx: &Context, msg: &Message, imdb_link: &str) {
        let regex =
            Regex::new(r"^(?:http://|https://)?(?:www\.|m\.)?(?:imdb.com/title/)?(tt[0-9]*)")
                .expect("Unable to parse regex pattern");

        let result = regex.captures(imdb_link);

        let link = match result {
            Some(link) => link[1].to_string(),
            None => {
                eprintln!("Link not found");
                msg.react(ctx, '🤖').await.ok();
                return;
            }
        };
        let new_suggestion = NewSuggestion {
            imdbId: link,
            createdByDiscordId: msg.author.id.to_string(),
            suggestion: true,
            deleted: false,
        };

        let rw_lock_client = ctx.data.read().await;

        let client = match rw_lock_client.get::<PbClient>() {
            Some(client) => client,
            None => {
                eprintln!("Client not found");
                msg.react(ctx, '🤖').await.ok();
                return;
            }
        };

        let create_response = client.records("movies").create(&new_suggestion).call();

        let reaction_emoji = match create_response {
            Ok(_) => '💾',
            Err(e) => {
                eprintln!("Error creating suggestion {}", e);
                '🚨'
            }
        };
        msg.react(ctx, reaction_emoji).await.ok();
    }

    async fn find_imdb_url(input: &str) -> Result<&str, &'static str> {
        let mut remaining = input;
        while let Some(a_start) = remaining.find("<a ") {
            remaining = &remaining[a_start..];
            if let Some(action_end) = remaining.find('>') {
                let action_attr = &remaining[..action_end];
                if action_attr.contains(r#"data-track-action="IMDb""#) {
                    if let Some(href_start) = remaining.find(r#"href=""#) {
                        let remaining = &remaining[href_start + 6..];
                        if let Some(href_end) = remaining.find('"') {
                            return Ok(&remaining[..href_end]);
                        }
                    }
                }
            }
            remaining = &remaining[3..];
        }
        Err("Attribute not found")
    }
}
