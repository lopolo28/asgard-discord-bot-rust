pub mod asgard_events {
    use raxios::{ContentType, Raxios, RaxiosHeaders, RaxiosOptions};
    use regex::Regex;
    use serenity::client::Context;
    use serenity::model::prelude::Message;
    use std::env;
    use parsercher::{parse,dom::Tag};
    pub async fn onmessage(ctx: &Context, msg: &Message) {
        if msg.content.starts_with("https://letterboxd.com/") {
            let response = reqwest::get(&msg.content).await.unwrap();
            println!("Got Response");
            if response.status().is_success() {
                let body = response.text().await.unwrap();
                let dom = parse(&body).expect("HTML string too long");
                let mut needle = Tag::new("a");
                needle.set_attr("data-track-action", "IMDb");
                if let Some(tags) = parsercher::search_tag(&dom, &needle) {
                    imdb(ctx, msg, tags.last().expect("Empty").get_attr("href").expect("Error accured during unwraping").as_str()).await;
                }
            }
        } else if msg.content.contains("imdb.com") {
            imdb(ctx, msg, &msg.content).await;
        }
    }
    async fn imdb(ctx: &Context, msg: &Message, imdb_link: &str) {
        let regex =
        Regex::new(r"^(?:http://|https://)?(?:www\.|m\.)?(?:imdb.com/title/)?(tt[0-9]*)")
        .unwrap();
        
        let result = regex.captures(imdb_link);
        
        let link = &result.unwrap()[1];
        println!("{}",link);

        let mut headers: RaxiosHeaders = RaxiosHeaders::new();
        headers.insert(String::from("imdbId"), String::from(link));
        let uri = env::var("API_URL").unwrap() + "/suggestions";
        let client = Raxios::new(&uri, None).ok();

        let options: RaxiosOptions = RaxiosOptions {
            headers: Option::from(headers),
            accept: Option::from(ContentType::Json),
            content_type: Option::from(ContentType::Json),
            params: None,
            deserialize_body: true,
        };

        let response = client
            .expect("Error")
            .post::<u32, &str>(&uri, Option::from(link), Option::from(options))
            .await
            .expect("Error 2");
        let reaction_emoji = match response.status.as_u16() {
            201 => '💾',
            400 => '🚨',
            _ => '🥵',
        };
        msg.react(ctx, reaction_emoji).await.ok();
    }
}
