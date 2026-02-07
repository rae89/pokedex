use serde::Deserialize;

use super::pokemon::NamedResource;

#[derive(Debug, Clone, Deserialize)]
pub struct TypeInfo {
    pub id: u32,
    pub name: String,
    pub damage_relations: DamageRelations,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DamageRelations {
    pub double_damage_to: Vec<NamedResource>,
    pub half_damage_to: Vec<NamedResource>,
    pub no_damage_to: Vec<NamedResource>,
    pub double_damage_from: Vec<NamedResource>,
    pub half_damage_from: Vec<NamedResource>,
    pub no_damage_from: Vec<NamedResource>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_info_deserialization() {
        let json = r#"{
            "id": 10,
            "name": "fire",
            "damage_relations": {
                "double_damage_to": [
                    {"name": "grass", "url": "https://pokeapi.co/api/v2/type/12/"},
                    {"name": "ice", "url": "https://pokeapi.co/api/v2/type/15/"}
                ],
                "half_damage_to": [
                    {"name": "fire", "url": "https://pokeapi.co/api/v2/type/10/"},
                    {"name": "water", "url": "https://pokeapi.co/api/v2/type/11/"}
                ],
                "no_damage_to": [],
                "double_damage_from": [
                    {"name": "water", "url": "https://pokeapi.co/api/v2/type/11/"},
                    {"name": "ground", "url": "https://pokeapi.co/api/v2/type/5/"}
                ],
                "half_damage_from": [
                    {"name": "fire", "url": "https://pokeapi.co/api/v2/type/10/"},
                    {"name": "grass", "url": "https://pokeapi.co/api/v2/type/12/"}
                ],
                "no_damage_from": []
            }
        }"#;

        let type_info: TypeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(type_info.id, 10);
        assert_eq!(type_info.name, "fire");
        assert_eq!(type_info.damage_relations.double_damage_to.len(), 2);
        assert_eq!(type_info.damage_relations.half_damage_to.len(), 2);
        assert_eq!(type_info.damage_relations.double_damage_from.len(), 2);
        assert_eq!(type_info.damage_relations.half_damage_from.len(), 2);
    }

    #[test]
    fn test_damage_relations_empty() {
        let json = r#"{
            "id": 1,
            "name": "normal",
            "damage_relations": {
                "double_damage_to": [],
                "half_damage_to": [],
                "no_damage_to": [],
                "double_damage_from": [],
                "half_damage_from": [],
                "no_damage_from": []
            }
        }"#;

        let type_info: TypeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(type_info.id, 1);
        assert_eq!(type_info.name, "normal");
        assert!(type_info.damage_relations.double_damage_to.is_empty());
    }
}
