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

/// API resource with only a URL (no name), used for fields like evolution_chain
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResource {
    pub url: String,
}

/// Species data from /pokemon-species/{id}
#[derive(Debug, Clone, Deserialize)]
pub struct PokemonSpecies {
    pub id: u32,
    pub name: String,
    pub evolution_chain: ApiResource,
}

/// Evolution chain from /evolution-chain/{id}
#[derive(Debug, Clone, Deserialize)]
pub struct EvolutionChain {
    pub id: u32,
    pub chain: EvolutionChainLink,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvolutionChainLink {
    pub species: NamedResource,
    pub evolution_details: Vec<EvolutionDetail>,
    pub evolves_to: Vec<EvolutionChainLink>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct EvolutionDetail {
    pub trigger: NamedResource,
    pub min_level: Option<u32>,
    pub item: Option<NamedResource>,
    pub held_item: Option<NamedResource>,
    pub min_happiness: Option<u32>,
    pub known_move: Option<NamedResource>,
    pub location: Option<NamedResource>,
    pub time_of_day: Option<String>,
    pub trade_species: Option<NamedResource>,
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

    #[test]
    fn test_pokemon_species_deserialization() {
        let json = r#"{
            "id": 1,
            "name": "bulbasaur",
            "evolution_chain": {"url": "https://pokeapi.co/api/v2/evolution-chain/1/"}
        }"#;

        let species: PokemonSpecies = serde_json::from_str(json).unwrap();
        assert_eq!(species.id, 1);
        assert_eq!(species.name, "bulbasaur");
        assert!(species.evolution_chain.url.contains("evolution-chain/1"));
    }

    #[test]
    fn test_evolution_chain_deserialization() {
        let json = r#"{
            "id": 1,
            "chain": {
                "species": {"name": "bulbasaur", "url": "https://pokeapi.co/api/v2/pokemon-species/1/"},
                "evolution_details": [],
                "evolves_to": [
                    {
                        "species": {"name": "ivysaur", "url": "https://pokeapi.co/api/v2/pokemon-species/2/"},
                        "evolution_details": [
                            {
                                "trigger": {"name": "level-up", "url": ""},
                                "min_level": 16,
                                "item": null,
                                "held_item": null,
                                "min_happiness": null,
                                "known_move": null,
                                "location": null,
                                "time_of_day": "",
                                "trade_species": null
                            }
                        ],
                        "evolves_to": [
                            {
                                "species": {"name": "venusaur", "url": "https://pokeapi.co/api/v2/pokemon-species/3/"},
                                "evolution_details": [
                                    {
                                        "trigger": {"name": "level-up", "url": ""},
                                        "min_level": 32,
                                        "item": null,
                                        "held_item": null,
                                        "min_happiness": null,
                                        "known_move": null,
                                        "location": null,
                                        "time_of_day": "",
                                        "trade_species": null
                                    }
                                ],
                                "evolves_to": []
                            }
                        ]
                    }
                ]
            }
        }"#;

        let chain: EvolutionChain = serde_json::from_str(json).unwrap();
        assert_eq!(chain.id, 1);
        assert_eq!(chain.chain.species.name, "bulbasaur");
        assert_eq!(chain.chain.evolves_to.len(), 1);
        assert_eq!(chain.chain.evolves_to[0].species.name, "ivysaur");
        assert_eq!(chain.chain.evolves_to[0].evolution_details[0].min_level, Some(16));
        assert_eq!(chain.chain.evolves_to[0].evolves_to[0].species.name, "venusaur");
        assert_eq!(chain.chain.evolves_to[0].evolves_to[0].evolution_details[0].min_level, Some(32));
    }

    #[test]
    fn test_evolution_chain_branching_deserialization() {
        let json = r#"{
            "id": 67,
            "chain": {
                "species": {"name": "eevee", "url": "https://pokeapi.co/api/v2/pokemon-species/133/"},
                "evolution_details": [],
                "evolves_to": [
                    {
                        "species": {"name": "vaporeon", "url": "https://pokeapi.co/api/v2/pokemon-species/134/"},
                        "evolution_details": [
                            {
                                "trigger": {"name": "use-item", "url": ""},
                                "min_level": null,
                                "item": {"name": "water-stone", "url": ""},
                                "held_item": null,
                                "min_happiness": null,
                                "known_move": null,
                                "location": null,
                                "time_of_day": "",
                                "trade_species": null
                            }
                        ],
                        "evolves_to": []
                    },
                    {
                        "species": {"name": "jolteon", "url": "https://pokeapi.co/api/v2/pokemon-species/135/"},
                        "evolution_details": [
                            {
                                "trigger": {"name": "use-item", "url": ""},
                                "min_level": null,
                                "item": {"name": "thunder-stone", "url": ""},
                                "held_item": null,
                                "min_happiness": null,
                                "known_move": null,
                                "location": null,
                                "time_of_day": "",
                                "trade_species": null
                            }
                        ],
                        "evolves_to": []
                    }
                ]
            }
        }"#;

        let chain: EvolutionChain = serde_json::from_str(json).unwrap();
        assert_eq!(chain.id, 67);
        assert_eq!(chain.chain.species.name, "eevee");
        assert_eq!(chain.chain.evolves_to.len(), 2);
        assert_eq!(chain.chain.evolves_to[0].species.name, "vaporeon");
        assert_eq!(chain.chain.evolves_to[0].evolution_details[0].item.as_ref().unwrap().name, "water-stone");
        assert_eq!(chain.chain.evolves_to[1].species.name, "jolteon");
        assert_eq!(chain.chain.evolves_to[1].evolution_details[0].item.as_ref().unwrap().name, "thunder-stone");
    }

    #[test]
    fn test_evolution_chain_single_stage_deserialization() {
        let json = r#"{
            "id": 100,
            "chain": {
                "species": {"name": "tauros", "url": "https://pokeapi.co/api/v2/pokemon-species/128/"},
                "evolution_details": [],
                "evolves_to": []
            }
        }"#;

        let chain: EvolutionChain = serde_json::from_str(json).unwrap();
        assert_eq!(chain.id, 100);
        assert_eq!(chain.chain.species.name, "tauros");
        assert!(chain.chain.evolves_to.is_empty());
        assert!(chain.chain.evolution_details.is_empty());
    }

    #[test]
    fn test_evolution_detail_trade_with_held_item() {
        let json = r#"{
            "trigger": {"name": "trade", "url": ""},
            "min_level": null,
            "item": null,
            "held_item": {"name": "metal-coat", "url": ""},
            "min_happiness": null,
            "known_move": null,
            "location": null,
            "time_of_day": "",
            "trade_species": null
        }"#;

        let detail: EvolutionDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.trigger.name, "trade");
        assert!(detail.held_item.is_some());
        assert_eq!(detail.held_item.unwrap().name, "metal-coat");
        assert!(detail.min_level.is_none());
        assert!(detail.item.is_none());
    }
}
