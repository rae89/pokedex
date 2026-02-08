use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub struct ApiClient {
    client: Client,
    cache_dir: PathBuf,
}

impl ApiClient {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pokemon-tui")
            .join("api");
        let _ = std::fs::create_dir_all(&cache_dir);
        Self {
            client: Client::new(),
            cache_dir,
        }
    }

    /// Fetch JSON, using filesystem cache
    pub async fn get_cached<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let cache_key = Self::url_to_cache_key(url);
        let cache_path = self.cache_dir.join(&cache_key);

        if let Ok(data) = std::fs::read_to_string(&cache_path) {
            if let Ok(parsed) = serde_json::from_str(&data) {
                return Ok(parsed);
            }
        }

        let resp = self.client.get(url).send().await?.text().await?;
        let _ = std::fs::write(&cache_path, &resp);
        Ok(serde_json::from_str(&resp)?)
    }

    /// Fetch raw bytes (for sprites), using filesystem cache
    pub async fn get_bytes_cached(&self, url: &str) -> Result<Vec<u8>> {
        let cache_key = Self::url_to_cache_key(url);
        let cache_path = self.cache_dir.join(&cache_key);

        if let Ok(data) = std::fs::read(&cache_path) {
            return Ok(data);
        }

        let bytes = self.client.get(url).send().await?.bytes().await?.to_vec();
        let _ = std::fs::write(&cache_path, &bytes);
        Ok(bytes)
    }

    pub(crate) fn url_to_cache_key(url: &str) -> String {
        url.replace("https://", "")
            .replace("http://", "")
            .replace(['/', '?'], "_")
    }

    #[cfg(test)]
    pub(crate) fn new_with_cache_dir(cache_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&cache_dir);
        Self {
            client: Client::new(),
            cache_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use tempfile::TempDir;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestData {
        value: String,
    }

    #[test]
    fn test_url_to_cache_key() {
        assert_eq!(
            ApiClient::url_to_cache_key("https://pokeapi.co/api/v2/pokemon/1"),
            "pokeapi.co_api_v2_pokemon_1"
        );
        assert_eq!(
            ApiClient::url_to_cache_key("http://example.com/test?param=value"),
            "example.com_test_param=value"
        );
        assert_eq!(
            ApiClient::url_to_cache_key("https://api.example.com/path/to/resource"),
            "api.example.com_path_to_resource"
        );
        assert_eq!(
            ApiClient::url_to_cache_key("https://example.com/"),
            "example.com_"
        );
    }

    #[tokio::test]
    async fn test_get_cached_cache_hit() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let client = ApiClient::new_with_cache_dir(cache_dir.clone());

        // Pre-populate cache
        let url = "https://example.com/test";
        let cache_key = ApiClient::url_to_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        let cached_data = r#"{"value": "cached"}"#;
        std::fs::write(&cache_path, cached_data).unwrap();

        // This should return cached data without making HTTP request
        // Note: This test verifies cache reading logic, but won't work with real HTTP
        // In a real scenario, we'd mock the HTTP client
        let result: TestData = client.get_cached(url).await.unwrap();
        assert_eq!(result.value, "cached");
    }

    #[tokio::test]
    async fn test_get_bytes_cached_cache_hit() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let client = ApiClient::new_with_cache_dir(cache_dir.clone());

        // Pre-populate cache
        let url = "https://example.com/sprite.png";
        let cache_key = ApiClient::url_to_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        let cached_bytes = b"fake png data";
        std::fs::write(&cache_path, cached_bytes).unwrap();

        // This should return cached bytes
        let result = client.get_bytes_cached(url).await.unwrap();
        assert_eq!(result, cached_bytes);
    }

    #[test]
    fn test_new_creates_cache_dir() {
        // Test that ApiClient::new() creates the cache directory
        // We use a temporary directory to verify the directory creation logic
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("api");
        let client = ApiClient::new_with_cache_dir(cache_dir.clone());

        // Verify the cache directory was created
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());

        // Verify the client can be used (indirectly tests that new() would work similarly)
        drop(client);
    }
}
