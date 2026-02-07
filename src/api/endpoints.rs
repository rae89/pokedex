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
