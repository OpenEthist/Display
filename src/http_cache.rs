use std::fs;
use std::hash::Hasher;
use std::path::{Path, PathBuf};

const CACHE_DIR: &str = ".http_cache";

// TODO: Cache size limits

pub async fn get(url: &str) -> Result<PathBuf, reqwest::Error> {
    let cache_dir = Path::new(CACHE_DIR);
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir).expect("could not create cache dir");
    }

    let encoded_url = encode(url);
    let path = cache_dir.join(&encoded_url);

    if path.exists() {
        return Ok(path);
    }

    println!("Downloading...");

    let res = reqwest::get(url).await?.error_for_status()?;

    let data = res.bytes().await?;
    fs::write(&path, data).expect("could not write chunk to http cached file");

    println!("Downloaded!");

    Ok(path)
}

fn encode(url: &str) -> String {
    let mut hasher = std::hash::DefaultHasher::new();
    hasher.write(url.as_bytes());
    hasher.finish().to_string()
}
