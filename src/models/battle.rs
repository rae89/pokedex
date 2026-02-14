use rand::Rng;

/// A Pokémon prepared for battle with base stats and moves.
#[derive(Debug, Clone)]
pub struct BattlePokemon {
    pub name: String,
    pub types: Vec<String>,
    pub max_hp: u32,
    pub current_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub special: u32, // Gen 1 combined Special
    pub speed: u32,
    pub level: u32,
    pub moves: Vec<BattleMove>,
}

#[derive(Debug, Clone)]
pub struct BattleMove {
    pub name: String,
    pub move_type: String,
    pub power: u32,
    pub accuracy: u32,
    pub is_special: bool, // true = special, false = physical
}

#[derive(Debug, Clone)]
pub struct BattleLogEntry {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BattlePhase {
    /// Picking first Pokémon (player 1)
    SelectPokemon1,
    /// Picking second Pokémon (player 2 / AI)
    SelectPokemon2,
    /// Player picks a move
    SelectMove,
    /// Animating a turn (ticks drain HP bars)
    Animating,
    /// Battle is over
    Finished,
}

#[derive(Debug, Clone)]
pub struct BattleState {
    pub phase: BattlePhase,
    pub pokemon1: Option<BattlePokemon>,
    pub pokemon2: Option<BattlePokemon>,
    pub log: Vec<BattleLogEntry>,
    pub selected_move: usize,
    pub turn: u32,
    pub winner: Option<u8>, // 1 or 2
    // picker state
    pub picker_selected: usize,
    pub picker_search: String,
    pub picker_search_mode: bool,
    // animation
    pub anim_ticks_remaining: u8,
    pub anim_p1_target_hp: u32,
    pub anim_p2_target_hp: u32,
}

impl Default for BattleState {
    fn default() -> Self {
        Self {
            phase: BattlePhase::SelectPokemon1,
            pokemon1: None,
            pokemon2: None,
            log: vec![BattleLogEntry {
                text: "Welcome to the Battle Simulator! Pick your Pokémon.".into(),
            }],
            selected_move: 0,
            turn: 0,
            winner: None,
            picker_selected: 0,
            picker_search: String::new(),
            picker_search_mode: false,
            anim_ticks_remaining: 0,
            anim_p1_target_hp: 0,
            anim_p2_target_hp: 0,
        }
    }
}

impl BattleState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

// ── Gen 1 type effectiveness ────────────────────────────────────────

pub fn type_effectiveness(attack_type: &str, defend_type: &str) -> f64 {
    match (attack_type, defend_type) {
        // Normal
        ("normal", "rock") | ("normal", "steel") => 0.5,
        ("normal", "ghost") => 0.0,
        // Fire
        ("fire", "grass") | ("fire", "ice") | ("fire", "bug") | ("fire", "steel") => 2.0,
        ("fire", "fire") | ("fire", "water") | ("fire", "rock") | ("fire", "dragon") => 0.5,
        // Water
        ("water", "fire") | ("water", "ground") | ("water", "rock") => 2.0,
        ("water", "water") | ("water", "grass") | ("water", "dragon") => 0.5,
        // Electric
        ("electric", "water") | ("electric", "flying") => 2.0,
        ("electric", "electric") | ("electric", "grass") | ("electric", "dragon") => 0.5,
        ("electric", "ground") => 0.0,
        // Grass
        ("grass", "water") | ("grass", "ground") | ("grass", "rock") => 2.0,
        ("grass", "fire")
        | ("grass", "grass")
        | ("grass", "poison")
        | ("grass", "flying")
        | ("grass", "bug")
        | ("grass", "dragon")
        | ("grass", "steel") => 0.5,
        // Ice
        ("ice", "grass") | ("ice", "ground") | ("ice", "flying") | ("ice", "dragon") => 2.0,
        ("ice", "fire") | ("ice", "water") | ("ice", "ice") | ("ice", "steel") => 0.5,
        // Fighting
        ("fighting", "normal")
        | ("fighting", "ice")
        | ("fighting", "rock")
        | ("fighting", "dark")
        | ("fighting", "steel") => 2.0,
        ("fighting", "poison")
        | ("fighting", "flying")
        | ("fighting", "psychic")
        | ("fighting", "bug")
        | ("fighting", "fairy") => 0.5,
        ("fighting", "ghost") => 0.0,
        // Poison
        ("poison", "grass") | ("poison", "fairy") => 2.0,
        ("poison", "poison") | ("poison", "ground") | ("poison", "rock") | ("poison", "ghost") => {
            0.5
        }
        ("poison", "steel") => 0.0,
        // Ground
        ("ground", "fire")
        | ("ground", "electric")
        | ("ground", "poison")
        | ("ground", "rock")
        | ("ground", "steel") => 2.0,
        ("ground", "grass") | ("ground", "bug") => 0.5,
        ("ground", "flying") => 0.0,
        // Flying
        ("flying", "grass") | ("flying", "fighting") | ("flying", "bug") => 2.0,
        ("flying", "electric") | ("flying", "rock") | ("flying", "steel") => 0.5,
        // Psychic
        ("psychic", "fighting") | ("psychic", "poison") => 2.0,
        ("psychic", "psychic") | ("psychic", "steel") => 0.5,
        ("psychic", "dark") => 0.0,
        // Bug
        ("bug", "grass") | ("bug", "psychic") | ("bug", "dark") => 2.0,
        ("bug", "fire")
        | ("bug", "fighting")
        | ("bug", "poison")
        | ("bug", "flying")
        | ("bug", "ghost")
        | ("bug", "steel")
        | ("bug", "fairy") => 0.5,
        // Rock
        ("rock", "fire") | ("rock", "ice") | ("rock", "flying") | ("rock", "bug") => 2.0,
        ("rock", "fighting") | ("rock", "ground") | ("rock", "steel") => 0.5,
        // Ghost
        ("ghost", "psychic") | ("ghost", "ghost") => 2.0,
        ("ghost", "dark") => 0.5,
        ("ghost", "normal") => 0.0,
        // Dragon
        ("dragon", "dragon") => 2.0,
        ("dragon", "steel") => 0.5,
        ("dragon", "fairy") => 0.0,
        // Dark
        ("dark", "psychic") | ("dark", "ghost") => 2.0,
        ("dark", "fighting") | ("dark", "dark") | ("dark", "fairy") => 0.5,
        // Steel
        ("steel", "ice") | ("steel", "rock") | ("steel", "fairy") => 2.0,
        ("steel", "fire") | ("steel", "water") | ("steel", "electric") | ("steel", "steel") => 0.5,
        // Fairy
        ("fairy", "fighting") | ("fairy", "dragon") | ("fairy", "dark") => 2.0,
        ("fairy", "fire") | ("fairy", "poison") | ("fairy", "steel") => 0.5,
        // Default neutral
        _ => 1.0,
    }
}

/// Calculate combined type effectiveness against a defender with 1-2 types.
pub fn combined_effectiveness(attack_type: &str, defender_types: &[String]) -> f64 {
    defender_types
        .iter()
        .map(|t| type_effectiveness(attack_type, t))
        .product()
}

/// Effectiveness message
pub fn effectiveness_message(multiplier: f64) -> Option<&'static str> {
    if multiplier >= 2.0 {
        Some("It's super effective!")
    } else if multiplier > 0.0 && multiplier < 1.0 {
        Some("It's not very effective...")
    } else if multiplier == 0.0 {
        Some("It doesn't affect the opponent...")
    } else {
        None
    }
}

/// Gen 1 damage formula (simplified):
///   ((2*Level/5+2) * Power * A/D) / 50 + 2) * STAB * Type * random(0.85..1.0)
pub fn calculate_damage(
    attacker: &BattlePokemon,
    defender: &BattlePokemon,
    battle_move: &BattleMove,
) -> (u32, f64) {
    if battle_move.power == 0 {
        return (0, 1.0);
    }

    let level = attacker.level as f64;
    let power = battle_move.power as f64;

    let (atk_stat, def_stat) = if battle_move.is_special {
        (attacker.special as f64, defender.special as f64)
    } else {
        (attacker.attack as f64, defender.defense as f64)
    };

    let def_stat = def_stat.max(1.0);

    // STAB
    let stab = if attacker.types.contains(&battle_move.move_type) {
        1.5
    } else {
        1.0
    };

    // Type effectiveness
    let type_mult = combined_effectiveness(&battle_move.move_type, &defender.types);

    // Random factor 0.85-1.0
    let mut rng = rand::rng();
    let random: f64 = rng.random_range(0.85..=1.0);

    let base = ((2.0 * level / 5.0 + 2.0) * power * atk_stat / def_stat) / 50.0 + 2.0;
    let damage = (base * stab * type_mult * random).floor() as u32;

    (damage.max(1), type_mult)
}

/// Simple AI: pick the move that does the most expected damage, with some randomness.
pub fn ai_pick_move(attacker: &BattlePokemon, defender: &BattlePokemon) -> usize {
    if attacker.moves.is_empty() {
        return 0;
    }

    let mut best_idx = 0;
    let mut best_score = 0.0f64;

    for (i, m) in attacker.moves.iter().enumerate() {
        let eff = combined_effectiveness(&m.move_type, &defender.types);
        let stab = if attacker.types.contains(&m.move_type) {
            1.5
        } else {
            1.0
        };
        let score = m.power as f64 * eff * stab;
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    // 30% chance to pick a random move for variety
    let mut rng = rand::rng();
    if rng.random_range(0..10) < 3 {
        rng.random_range(0..attacker.moves.len())
    } else {
        best_idx
    }
}

/// Determine who goes first based on speed (ties: player 1 advantage).
pub fn first_attacker(p1: &BattlePokemon, p2: &BattlePokemon) -> u8 {
    if p1.speed >= p2.speed {
        1
    } else {
        2
    }
}

/// Gen 1: physical vs special is determined by move type
pub fn is_special_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "water" | "fire" | "grass" | "electric" | "ice" | "psychic" | "dragon"
    )
}

/// Build default moves for a Pokémon if it has none assigned (fallback)
pub fn default_moves_for_types(types: &[String]) -> Vec<BattleMove> {
    let mut moves = Vec::new();

    // STAB moves based on type
    for t in types.iter().take(2) {
        let (name, power) = match t.as_str() {
            "normal" => ("Tackle", 40),
            "fire" => ("Flamethrower", 90),
            "water" => ("Surf", 90),
            "electric" => ("Thunderbolt", 90),
            "grass" => ("Razor Leaf", 55),
            "ice" => ("Ice Beam", 90),
            "fighting" => ("Karate Chop", 50),
            "poison" => ("Sludge Bomb", 90),
            "ground" => ("Earthquake", 100),
            "flying" => ("Aerial Ace", 60),
            "psychic" => ("Psychic", 90),
            "bug" => ("Bug Buzz", 90),
            "rock" => ("Rock Slide", 75),
            "ghost" => ("Shadow Ball", 80),
            "dragon" => ("Dragon Pulse", 85),
            "dark" => ("Dark Pulse", 80),
            "steel" => ("Iron Tail", 100),
            "fairy" => ("Moonblast", 95),
            _ => ("Tackle", 40),
        };
        moves.push(BattleMove {
            name: name.to_string(),
            move_type: t.clone(),
            power,
            accuracy: 100,
            is_special: is_special_type(t),
        });
    }

    // Fill remaining slots with Normal move + Struggle
    if moves.len() < 4 {
        moves.push(BattleMove {
            name: "Body Slam".to_string(),
            move_type: "normal".to_string(),
            power: 85,
            accuracy: 100,
            is_special: false,
        });
    }
    if moves.len() < 4 {
        moves.push(BattleMove {
            name: "Hyper Beam".to_string(),
            move_type: "normal".to_string(),
            power: 150,
            accuracy: 90,
            is_special: false,
        });
    }

    moves.truncate(4);
    moves
}
