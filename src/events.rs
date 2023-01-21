pub mod asgard_events {
    use parsercher::{dom::Tag, parse};
    use raxios::{ContentType, Raxios, RaxiosHeaders, RaxiosOptions};
    use regex::Regex;
    use serenity::client::Context;
    use serenity::model::prelude::Message;
    use std::env;

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct ToSend {
        imdbId: String,
    }

    pub async fn onmessage(ctx: &Context, msg: &Message) {
        if msg.content.starts_with("https://letterboxd.com/") {
            let response = reqwest::get(&msg.content)
                .await
                .expect("Letterboxd scrapping: No response recieved");
            if response.status().is_success() {
                let body = response.text().await.expect("Missing response body");

                let dom = parse(&body).expect("HTML string too long");

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

        let link = &result.expect("No result of Regex")[1];

        let mut headers: RaxiosHeaders = RaxiosHeaders::new();
        headers.insert(String::from("discord-id"), msg.author.id.0.to_string());
        let uri = env::var("API_URL").expect("API_URL not found");

        let client = Raxios::new(&uri, None).ok();

        let options: RaxiosOptions = RaxiosOptions {
            headers: Option::from(headers),
            accept: Option::from(ContentType::Json),
            content_type: Option::from(ContentType::Json),
            params: None,
            deserialize_body: true,
        };

        let response = client
            .expect("Creating of client failed")
            .post::<u32, ToSend>(
                "/suggestions",
                Option::from(ToSend {
                    imdbId: String::from(link),
                }),
                Option::from(options),
            )
            .await
            .expect("Processing of response failed");

        let reaction_emoji = match response.status.as_u16() {
            201 => '💾',
            400 => '🚨',
            _ => '🥵',
        };
        msg.react(ctx, reaction_emoji).await.ok();
    }
}
