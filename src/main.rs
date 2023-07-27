use serde::Deserialize;
use std::collections::HashMap;
use reqwest::StatusCode;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;

#[derive(Deserialize, Debug)]
struct Album {
    album_type: String,
    artists: Vec<Artist>,
    // available_markets: Vec<String>,
    external_urls: HashMap<String, String>,
    href: String,
    id: String,
    images: Vec<Image>,
    name: String,
    release_date: String,
    release_date_precision: String,
    total_tracks: u32,
    #[serde(rename = "type")]
    spotify_type: String,
    uri: String,
}

#[derive(Deserialize, Debug)]
struct Artist {
    external_urls: HashMap<String, String>,
    href: String,
    id: String,
    name: String,
    #[serde(rename = "type")]
    artist_type: String,
    uri: String,
}

#[derive(Deserialize, Debug)]
struct Image {
    height: u32,
    url: String,
    width: u32,
}

#[derive(Deserialize, Debug)]
struct SpotifySearchResult {
    albums: AlbumsResponse,
}

#[derive(Deserialize, Debug)]
struct AlbumsResponse {
    href: String,
    items: Vec<Album>,
    limit: u32,
    next: Option<String>,
    offset: u32,
    previous: Option<String>,
    total: u32,
}

#[derive(Deserialize, Debug)]
struct SpotifyError {
    error: SpotifyErrorMessage,
}

#[derive(Deserialize, Debug)]
struct SpotifyErrorMessage {
    status: u16,
    message: String,
}

#[derive(Deserialize, Debug)]
struct Auth {
    access_token: String,
    token_type: String,
    expires_in: usize
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = "276282a6a7794ef7950a99e6d5c667ca";
    let client_secret = "b88ac9fc83404d79bd7038f49548fca2";
    let auth_value = format!("{}:{}", client_id, client_secret);
    let auth_value = format!("Basic {}", BASE64_STANDARD.encode(auth_value));

    let params = [("grant_type", "client_credentials")];

    let client = reqwest::Client::new();
    let res = client.post("https://accounts.spotify.com/api/token")
        .header("Authorization", auth_value)
        .form(&params)
        .send()
        .await?;

    let auth: Auth = res.json().await?;

    println!("Access Token: {}", auth.access_token);

    let query = "artist:Stupeflip";

    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("q", query);
    map.insert("type", "album");
    map.insert("limit", "5");

    let res = client.get("https://api.spotify.com/v1/search")
        .header("Authorization", format!("Bearer {}", auth.access_token))
        .query(&map)
        .send()
        .await?;

    match res.status() {
        StatusCode::OK => {
            let response = res.text().await?;

            let search_result: SpotifySearchResult = serde_json::from_str(&response)?;

            for album in search_result.albums.items {
                println!("Album: {:?}", album);
            }
        },
        StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::NOT_FOUND => {
            let response = res.text().await?;
            let error_response: SpotifyError = serde_json::from_str(&response)?;
            println!("Erreur Spotify: {}, Message: {}", error_response.error.status, error_response.error.message);
        },
        _ => println!("Erreur HTTP."),
    }

    Ok(())
}
