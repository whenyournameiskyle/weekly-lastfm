use egg_mode::tweet::DraftTweet;
use logs::{error, info};
use serde::Deserialize;
use serde_json::from_str;
use std::{env, io};

#[derive(Debug, Deserialize)]
struct Response {
    weeklyartistchart: Artists,
}
#[derive(Debug, Deserialize)]
struct Artist {
    name: String,
    playcount: String,
}
#[derive(Debug, Deserialize)]
struct Artists {
    artist: Vec<Artist>,
}

#[async_std::main]
async fn main() -> Result<(), io::Error> {
    let tweet: String = get_top_lastfm_tweet().await.expect("to have a value");

    if tweet.is_empty() {
        error!("No artists returned");
        return Ok(());
    }

    info!("{}", tweet);
    send_tweet(tweet).await;

    Ok(())
}

async fn get_top_lastfm_tweet() -> Result<String, io::Error> {
    let lastfm_key =
        env::var("LASTFM_API_KEY").expect("Please set environment variable LASTFM_API_KEY");
    let lastfm_user = "";
    let string = surf::get(format!("http://ws.audioscrobbler.com/2.0/?method=user.getweeklyartistchart&user={}&api_key={}&format=json", lastfm_user, lastfm_key)).recv_string().await.unwrap();
    let resp: Response = from_str(&string)?;
    let artist: Vec<Artist> = resp.weeklyartistchart.artist;

    if artist.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "No artists returned"));
    }

    let mut tweet: String = "My most played artists last week:".to_owned();
    let mut artist_length = 5;

    if artist.len() < 5 {
        artist_length = artist.len();
    }

    for item in &artist[0..artist_length] {
        tweet = tweet + " " + &item.name + " (" + &item.playcount + ")";
    }

    tweet += &format!(" https://www.last.fm/user/{}", lastfm_user);

    Ok(tweet)
}

async fn send_tweet(value: String) {
    let access_token = egg_mode::KeyPair::new(
        env::var("ACCESS_TOKEN_KEY").expect("Please set environment variable ACCESS_TOKEN_KEY"),
        env::var("ACCESS_TOKEN_SECRET")
            .expect("Please set environment variable ACCESS_TOKEN_SECRET"),
    );
    let con_token = egg_mode::KeyPair::new(
        env::var("CONSUMER_KEY").expect("Please set environment variable CONSUMER_KEY"),
        env::var("CONSUMER_SECRET").expect("Please set environment variable CONSUMER_SECRET"),
    );
    let token = egg_mode::Token::Access {
        consumer: con_token,
        access: access_token,
    };

    if let Err(err) = egg_mode::auth::verify_tokens(&token).await {
        error!("We've hit an error using your old tokens: {:?}", err);
        return;
    } else {
        info!("Authenticated successfully.");
    }

    let tweet = DraftTweet::new(value);
    tweet
        .send(&token)
        .await
        .expect("Something went wrong with sending the tweet");

    info!("Tweet sent!");
}
