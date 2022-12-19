use super::config::Config;
use super::VideoType;
use crate::Error;
use derive_more::Constructor;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TwitchToken {
    access_token: String,
}

#[derive(Constructor, Debug)]
pub(super) struct TokenPackage {
    pub(super) token: String,
    pub(super) client: Client,
    pub(super) client_id: String,
    // pub(super) client_secret: String,
}

pub(super) fn get(video_type: &VideoType, config: &Config) -> Result<TokenPackage, Error> {
    match video_type {
        VideoType::Vod | VideoType::Highlight | VideoType::Clip => get_twitch_token(config),
        VideoType::YouTube => get_youtube_token(config),
    }
}

fn get_twitch_token(config: &Config) -> Result<TokenPackage, Error> {
    let client_id = config.twitch_client_id.clone();
    let client_secret = config.twitch_secret.clone();
    if client_id.is_empty() || client_secret.is_empty() {
        return Err(Error::Token(
            "No Twitch client ID or secret found.".to_string(),
        ));
    }
    let token_url = format!("https://id.twitch.tv/oauth2/token?client_id={client_id}&client_secret={client_secret}&grant_type=client_credentials");
    match post(&token_url) {
        Ok((text, client)) => match serde_json::from_str::<TwitchToken>(&text) {
            Ok(json) => {
                if json.access_token.is_empty() {
                    return Err(Error::Token("No Twitch access token found.".to_string()));
                }
                Ok(TokenPackage::new(
                    json.access_token,
                    client,
                    client_id,
                    // client_secret,
                ))
            }
            Err(_) => Err(Error::Token(
                "Could not parse Twitch token response.".to_string(),
            )),
        },
        Err(_) => Err(Error::Token("Request for Twitch token failed.".to_string())),
    }
}

fn get_youtube_token(config: &Config) -> Result<TokenPackage, Error> {
    if config.youtube_key.is_empty() {
        return Err(Error::Token("No YouTube API key found.".to_string()));
    }
    let client = Client::new();
    Ok(TokenPackage::new(
        config.youtube_key.clone(),
        client,
        String::new(),
        // String::new(),
    ))
}

fn post(url: &str) -> Result<(String, Client), Error> {
    let client = Client::new();
    let response = client.post(url).send()?;
    let text = response.text()?;
    Ok((text, client))
}
