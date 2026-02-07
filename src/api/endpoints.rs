use anyhow::Result;

use super::client::ApiClient;
use crate::models::pokemon::{MoveDetail, PokemonDetail, PokemonListResponse};
use crate::models::type_data::TypeInfo;

const BASE_URL: &str = "https://pokeapi.co/api/v2";

impl ApiClient {
    pub async fn fetch_pokemon_list(&self) -> Result<PokemonListResponse> {
        let url = format!("{}/pokemon?limit=10000", BASE_URL);
        self.get_cached(&url).await
    }

    pub async fn fetch_pokemon_detail(&self, id_or_name: &str) -> Result<PokemonDetail> {
        let url = format!("{}/pokemon/{}", BASE_URL, id_or_name);
        self.get_cached(&url).await
    }

    pub async fn fetch_type_info(&self, name: &str) -> Result<TypeInfo> {
        let url = format!("{}/type/{}", BASE_URL, name);
        self.get_cached(&url).await
    }

    pub async fn fetch_move_detail(&self, name: &str) -> Result<MoveDetail> {
        let url = format!("{}/move/{}", BASE_URL, name);
        self.get_cached(&url).await
    }

    pub async fn fetch_sprite_bytes(&self, url: &str) -> Result<Vec<u8>> {
        self.get_bytes_cached(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_fetch_pokemon_list_url_construction() {
        // Test that the URL is constructed correctly
        // In a real scenario with mocking, we'd verify the request goes to the right URL
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let client = ApiClient::new_with_cache_dir(cache_dir.clone());

        // Pre-populate cache with mock data
        let url = "https://pokeapi.co/api/v2/pokemon?limit=10000";
        let cache_key = ApiClient::url_to_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        let mock_response = r#"{"results": [{"name": "bulbasaur", "url": "https://pokeapi.co/api/v2/pokemon/1/"}]}"#;
        std::fs::write(&cache_path, mock_response).unwrap();

        let result = client.fetch_pokemon_list().await;
        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.results.len(), 1);
        assert_eq!(list.results[0].name, "bulbasaur");
    }

    #[tokio::test]
    async fn test_fetch_pokemon_detail_url_construction() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let client = ApiClient::new_with_cache_dir(cache_dir.clone());

        // Pre-populate cache
        let url = "https://pokeapi.co/api/v2/pokemon/1";
        let cache_key = ApiClient::url_to_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        let mock_response = r#"{
            "id": 1,
            "name": "bulbasaur",
            "height": 7,
            "weight": 69,
            "types": [],
            "stats": [],
            "abilities": [],
            "moves": [],
            "sprites": {"front_default": null}
        }"#;
        std::fs::write(&cache_path, mock_response).unwrap();

        let result = client.fetch_pokemon_detail("1").await;
        assert!(result.is_ok());
        let detail = result.unwrap();
        assert_eq!(detail.id, 1);
        assert_eq!(detail.name, "bulbasaur");
    }

    #[tokio::test]
    async fn test_fetch_type_info() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let client = ApiClient::new_with_cache_dir(cache_dir.clone());

        // Pre-populate cache
        let url = "https://pokeapi.co/api/v2/type/fire";
        let cache_key = ApiClient::url_to_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        let mock_response = r#"{
            "id": 10,
            "name": "fire",
            "damage_relations": {
                "double_damage_to": [],
                "half_damage_to": [],
                "no_damage_to": [],
                "double_damage_from": [],
                "half_damage_from": [],
                "no_damage_from": []
            }
        }"#;
        std::fs::write(&cache_path, mock_response).unwrap();

        let result = client.fetch_type_info("fire").await;
        assert!(result.is_ok());
        let type_info = result.unwrap();
        assert_eq!(type_info.id, 10);
        assert_eq!(type_info.name, "fire");
    }

    #[tokio::test]
    async fn test_fetch_move_detail() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let client = ApiClient::new_with_cache_dir(cache_dir.clone());

        // Pre-populate cache
        let url = "https://pokeapi.co/api/v2/move/tackle";
        let cache_key = ApiClient::url_to_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        let mock_response = r#"{
            "id": 33,
            "name": "tackle",
            "power": 40,
            "accuracy": 100,
            "pp": 35,
            "type": {"name": "normal", "url": "https://pokeapi.co/api/v2/type/1/"},
            "damage_class": {"name": "physical", "url": "https://pokeapi.co/api/v2/move-damage-class/2/"}
        }"#;
        std::fs::write(&cache_path, mock_response).unwrap();

        let result = client.fetch_move_detail("tackle").await;
        assert!(result.is_ok());
        let move_detail = result.unwrap();
        assert_eq!(move_detail.id, 33);
        assert_eq!(move_detail.name, "tackle");
        assert_eq!(move_detail.power, Some(40));
    }

    #[tokio::test]
    async fn test_fetch_sprite_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let client = ApiClient::new_with_cache_dir(cache_dir.clone());

        // Pre-populate cache
        let url = "https://example.com/sprite.png";
        let cache_key = ApiClient::url_to_cache_key(url);
        let cache_path = cache_dir.join(&cache_key);
        let mock_bytes = b"fake png data";
        std::fs::write(&cache_path, mock_bytes).unwrap();

        let result = client.fetch_sprite_bytes(url).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), mock_bytes);
    }
}
