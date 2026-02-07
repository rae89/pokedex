use serde::Deserialize;

/// Lightweight entry from the /pokemon?limit=151 list endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct PokemonListResponse {
    pub results: Vec<PokemonEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PokemonEntry {
    pub name: String,
    pub url: String,
}

/// Full detail from /pokemon/{id}
#[derive(Debug, Clone, Deserialize)]
pub struct PokemonDetail {
    pub id: u32,
    pub name: String,
    pub height: u32,
    pub weight: u32,
    pub types: Vec<PokemonTypeSlot>,
    pub stats: Vec<StatEntry>,
    pub abilities: Vec<AbilitySlot>,
    pub moves: Vec<MoveEntry>,
    pub sprites: Sprites,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PokemonTypeSlot {
    pub slot: u32,
    #[serde(rename = "type")]
    pub type_info: NamedResource,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatEntry {
    pub base_stat: u32,
    pub stat: NamedResource,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AbilitySlot {
    pub ability: NamedResource,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoveEntry {
    #[serde(rename = "move")]
    pub move_info: NamedResource,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sprites {
    pub front_default: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamedResource {
    pub name: String,
    pub url: String,
}

/// Summary used in the list screen (built from list + individual fetches)
#[derive(Debug, Clone)]
pub struct PokemonSummary {
    pub id: u32,
    pub name: String,
    pub types: Vec<String>,
}

/// Move detail from /move/{id}
#[derive(Debug, Clone, Deserialize)]
pub struct MoveDetail {
    pub id: u32,
    pub name: String,
    pub power: Option<u32>,
    pub accuracy: Option<u32>,
    pub pp: Option<u32>,
    #[serde(rename = "type")]
    pub move_type: NamedResource,
    pub damage_class: Option<NamedResource>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pokemon_summary_creation() {
        let summary = PokemonSummary {
            id: 25,
            name: "pikachu".to_string(),
            types: vec!["electric".to_string()],
        };

        assert_eq!(summary.id, 25);
        assert_eq!(summary.name, "pikachu");
        assert_eq!(summary.types.len(), 1);
        assert_eq!(summary.types[0], "electric");
    }

    #[test]
    fn test_pokemon_list_response_deserialization() {
        let json = r#"{
            "results": [
                {"name": "bulbasaur", "url": "https://pokeapi.co/api/v2/pokemon/1/"},
                {"name": "ivysaur", "url": "https://pokeapi.co/api/v2/pokemon/2/"}
            ]
        }"#;

        let response: PokemonListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].name, "bulbasaur");
        assert_eq!(response.results[1].name, "ivysaur");
    }

    #[test]
    fn test_pokemon_detail_deserialization() {
        let json = r#"{
            "id": 25,
            "name": "pikachu",
            "height": 4,
            "weight": 60,
            "types": [
                {"slot": 1, "type": {"name": "electric", "url": "https://pokeapi.co/api/v2/type/13/"}}
            ],
            "stats": [
                {"base_stat": 35, "stat": {"name": "hp", "url": "https://pokeapi.co/api/v2/stat/1/"}}
            ],
            "abilities": [
                {"ability": {"name": "static", "url": "https://pokeapi.co/api/v2/ability/9/"}, "is_hidden": false}
            ],
            "moves": [
                {"move": {"name": "tackle", "url": "https://pokeapi.co/api/v2/move/33/"}}
            ],
            "sprites": {"front_default": "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/25.png"}
        }"#;

        let detail: PokemonDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.id, 25);
        assert_eq!(detail.name, "pikachu");
        assert_eq!(detail.height, 4);
        assert_eq!(detail.weight, 60);
        assert_eq!(detail.types.len(), 1);
        assert_eq!(detail.types[0].type_info.name, "electric");
        assert_eq!(detail.stats.len(), 1);
        assert_eq!(detail.abilities.len(), 1);
        assert_eq!(detail.moves.len(), 1);
        assert!(detail.sprites.front_default.is_some());
    }

    #[test]
    fn test_move_detail_deserialization() {
        let json = r#"{
            "id": 33,
            "name": "tackle",
            "power": 40,
            "accuracy": 100,
            "pp": 35,
            "type": {"name": "normal", "url": "https://pokeapi.co/api/v2/type/1/"},
            "damage_class": {"name": "physical", "url": "https://pokeapi.co/api/v2/move-damage-class/2/"}
        }"#;

        let move_detail: MoveDetail = serde_json::from_str(json).unwrap();
        assert_eq!(move_detail.id, 33);
        assert_eq!(move_detail.name, "tackle");
        assert_eq!(move_detail.power, Some(40));
        assert_eq!(move_detail.accuracy, Some(100));
        assert_eq!(move_detail.pp, Some(35));
        assert_eq!(move_detail.move_type.name, "normal");
        assert!(move_detail.damage_class.is_some());
    }

    #[test]
    fn test_move_detail_with_optional_fields() {
        let json = r#"{
            "id": 1,
            "name": "pound",
            "power": null,
            "accuracy": null,
            "pp": null,
            "type": {"name": "normal", "url": "https://pokeapi.co/api/v2/type/1/"},
            "damage_class": null
        }"#;

        let move_detail: MoveDetail = serde_json::from_str(json).unwrap();
        assert_eq!(move_detail.id, 1);
        assert_eq!(move_detail.name, "pound");
        assert_eq!(move_detail.power, None);
        assert_eq!(move_detail.accuracy, None);
        assert_eq!(move_detail.pp, None);
        assert!(move_detail.damage_class.is_none());
    }
}
