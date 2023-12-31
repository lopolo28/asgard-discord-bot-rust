pub mod asgard_events {
    use parsercher::{dom::Tag, parse};
    use raxios::{ContentType, Raxios, RaxiosHeaders, RaxiosOptions};
    use regex::Regex;
    use serenity::client::Context;
    use serenity::model::prelude::Message;
    use std::env;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct ToReturn {}
    #[allow(non_snake_case)]
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct ToSend {
        imdbId: String,
    }

    // array of base urls to replace
    static TWITTER_BASE_URLS: [&'static str; 2] = ["https://twitter.com/", "https://x.com/"];

    pub async fn on_message_twitter(ctx: &Context, msg: &Message) {
        // find first element who match with any of the base urls
        let base_url = TWITTER_BASE_URLS.iter().find(|&i| msg.content.starts_with(i));

        if base_url.is_some() {
            // constant with replace content
            let replace_base_url = "https://vxtwitter.com/";
            // replace msg content with new base url
            let replaced_msg = msg.content.replacen(base_url.unwrap(), replace_base_url, 1);
            msg.reply(ctx, replaced_msg).await.ok();
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

        let mut headers: RaxiosHeaders = RaxiosHeaders::new();
        headers.insert(String::from("discord-id"), msg.author.id.0.to_string());
        let uri = env::var("API_URL").expect("API_URL not found");

        let client = match Raxios::new(&uri, None) {
            Ok(client) => client,
            Err(e) => {
                eprintln!("{}", e);
                msg.react(ctx, '🤖').await.ok();
                return;
            }
        };

        let options: RaxiosOptions = RaxiosOptions {
            headers: Option::from(headers),
            accept: Option::None,
            content_type: Option::from(ContentType::Json),
            params: None,
            deserialize_body: false,
        };
        let request = client.post::<ToReturn, ToSend>(
            "/suggestions",
            Option::from(ToSend { imdbId: link }),
            Option::from(options),
        );
        let response = match request.await {
            Ok(response) => response,
            Err(err) => {
                println!("{}", err);
                msg.react(ctx, '🚨').await.ok();
                return;
            }
        };

        let reaction_emoji = match response.status.as_u16() {
            201 => '💾',
            400..=499 => '🚨',
            _ => '🥵',
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
