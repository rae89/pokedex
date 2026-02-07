use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::api::client::ApiClient;
use crate::event::AppEvent;
use crate::models::pokemon::{MoveDetail, PokemonDetail, PokemonSummary};
use crate::models::team::{Team, TeamData, TeamMember, TeamMove};
use crate::models::type_data::TypeInfo;

fn extract_id_from_url(url: &str) -> Option<u32> {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()?
        .parse()
        .ok()
}

/// Calculate Pokemon generation from ID based on standard ranges
fn pokemon_generation(id: u32) -> u8 {
    match id {
        1..=151 => 1,
        152..=251 => 2,
        252..=386 => 3,
        387..=493 => 4,
        494..=649 => 5,
        650..=721 => 6,
        722..=809 => 7,
        810..=905 => 8,
        906..=1025 => 9,
        _ => 9, // Default to Gen 9 for any IDs beyond known range
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    PokemonList,
    PokemonDetail,
    TypeChart,
    TeamBuilder,
}

impl Screen {
    pub fn all() -> &'static [Screen] {
        &[
            Screen::PokemonList,
            Screen::PokemonDetail,
            Screen::TypeChart,
            Screen::TeamBuilder,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Screen::PokemonList => "Pokédex",
            Screen::PokemonDetail => "Detail",
            Screen::TypeChart => "Type Chart",
            Screen::TeamBuilder => "Team Builder",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Screen::PokemonList => 0,
            Screen::PokemonDetail => 1,
            Screen::TypeChart => 2,
            Screen::TeamBuilder => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modal {
    PokemonPicker,
    MovePicker,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadingState {
    Idle,
    Loading,
    Loaded,
    Error,
}

pub struct App {
    pub running: bool,
    pub screen: Screen,

    // Pokemon list
    pub pokemon_list: Vec<PokemonSummary>,
    pub list_state: usize, // selected index
    pub list_loading: LoadingState,
    pub search_mode: bool,
    pub search_query: String,
    pub generation_filter: Option<u8>, // None = all generations, Some(1-9) = specific generation

    // Pokemon detail
    pub detail: Option<Box<PokemonDetail>>,
    pub detail_loading: LoadingState,
    pub sprite_bytes: Option<Vec<u8>>,
    pub detail_pokemon_id: Option<u32>,

    // Type chart
    pub type_infos: Vec<TypeInfo>,
    pub type_chart_loading: LoadingState,
    pub type_chart_scroll_x: usize,
    pub type_chart_scroll_y: usize,

    // Team builder
    pub team_data: TeamData,
    pub current_team: usize,
    pub team_slot_selected: usize,
    pub modal: Option<Modal>,
    pub modal_selected: usize,
    pub modal_search: String,

    // Move picker
    pub available_moves: Vec<MoveDetail>,
    pub moves_loading: LoadingState,

    pub error_message: Option<String>,
    tx: mpsc::UnboundedSender<AppEvent>,
}

impl App {
    pub fn new(tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            running: true,
            screen: Screen::PokemonList,
            pokemon_list: Vec::new(),
            list_state: 0,
            list_loading: LoadingState::Idle,
            search_mode: false,
            search_query: String::new(),
            generation_filter: None,
            detail: None,
            detail_loading: LoadingState::Idle,
            sprite_bytes: None,
            detail_pokemon_id: None,
            type_infos: Vec::new(),
            type_chart_loading: LoadingState::Idle,
            type_chart_scroll_x: 0,
            type_chart_scroll_y: 0,
            team_data: TeamData::load(),
            current_team: 0,
            team_slot_selected: 0,
            modal: None,
            modal_selected: 0,
            modal_search: String::new(),
            available_moves: Vec::new(),
            moves_loading: LoadingState::Idle,
            error_message: None,
            tx,
        }
    }

    pub fn filtered_list(&self) -> Vec<&PokemonSummary> {
        let mut filtered: Vec<&PokemonSummary> = self.pokemon_list.iter().collect();

        // Apply generation filter
        if let Some(gen) = self.generation_filter {
            filtered.retain(|p| pokemon_generation(p.id) == gen);
        }

        // Apply search query filter
        if !self.search_query.is_empty() {
            let q = self.search_query.to_lowercase();
            filtered.retain(|p| p.name.contains(&q) || p.id.to_string().contains(&q));
        }

        filtered
    }

    pub fn current_team(&self) -> &Team {
        &self.team_data.teams[self.current_team]
    }

    pub fn current_team_mut(&mut self) -> &mut Team {
        &mut self.team_data.teams[self.current_team]
    }

    pub fn start_loading_list(&mut self) {
        if self.list_loading == LoadingState::Loaded {
            return;
        }
        self.list_loading = LoadingState::Loading;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let client = std::sync::Arc::new(ApiClient::new());
            match client.fetch_pokemon_list().await {
                Ok(list) => {
                    // Extract ID from URL: "https://pokeapi.co/api/v2/pokemon/25/" -> 25
                    let summaries: Vec<PokemonSummary> = list
                        .results
                        .iter()
                        .filter_map(|e| {
                            let id = extract_id_from_url(&e.url)?;
                            Some(PokemonSummary {
                                id,
                                name: e.name.clone(),
                                types: Vec::new(),
                            })
                        })
                        .collect();

                    // Send list immediately (no types yet)
                    let _ = tx.send(AppEvent::PokemonListLoaded(summaries.clone()));

                    // Background-fetch types in batches of 30
                    let entries: Vec<(u32, String)> = summaries
                        .iter()
                        .map(|s| (s.id, s.name.clone()))
                        .collect();

                    for chunk in entries.chunks(30) {
                        let mut handles = Vec::new();
                        for &(id, ref _name) in chunk {
                            let client = client.clone();
                            handles.push(tokio::spawn(async move {
                                match client.fetch_pokemon_detail(&id.to_string()).await {
                                    Ok(detail) => Some((
                                        id,
                                        detail
                                            .types
                                            .iter()
                                            .map(|t| t.type_info.name.clone())
                                            .collect::<Vec<String>>(),
                                    )),
                                    Err(_) => None,
                                }
                            }));
                        }
                        let mut batch = Vec::new();
                        for handle in handles {
                            if let Ok(Some(result)) = handle.await {
                                batch.push(result);
                            }
                        }
                        if !batch.is_empty() {
                            let _ = tx.send(AppEvent::PokemonTypesUpdated(batch));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ApiError(format!("Failed to load Pokémon list: {}", e)));
                }
            }
        });
    }

    pub fn load_detail(&mut self, id: u32) {
        if self.detail_pokemon_id == Some(id) && self.detail.is_some() {
            return;
        }
        self.detail = None;
        self.sprite_bytes = None;
        self.detail_pokemon_id = Some(id);
        self.detail_loading = LoadingState::Loading;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let client = ApiClient::new();
            match client.fetch_pokemon_detail(&id.to_string()).await {
                Ok(detail) => {
                    // Also fetch sprite
                    if let Some(ref url) = detail.sprites.front_default {
                        if let Ok(bytes) = client.fetch_sprite_bytes(url).await {
                            let _ = tx.send(AppEvent::SpriteLoaded(id, bytes));
                        }
                    }
                    let _ = tx.send(AppEvent::PokemonDetailLoaded(Box::new(detail)));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::ApiError(format!("Failed to load detail: {}", e)));
                }
            }
        });
    }

    pub fn load_types(&mut self) {
        if self.type_chart_loading == LoadingState::Loaded {
            return;
        }
        self.type_chart_loading = LoadingState::Loading;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let client = ApiClient::new();
            let type_names = [
                "normal", "fire", "water", "electric", "grass", "ice", "fighting", "poison",
                "ground", "flying", "psychic", "bug", "rock", "ghost", "dragon", "dark",
                "steel", "fairy",
            ];
            let mut infos = Vec::new();
            for name in &type_names {
                match client.fetch_type_info(name).await {
                    Ok(info) => infos.push(info),
                    Err(e) => {
                        let _ = tx.send(AppEvent::ApiError(format!(
                            "Failed to load type {}: {}",
                            name, e
                        )));
                        return;
                    }
                }
            }
            let _ = tx.send(AppEvent::TypesLoaded(infos));
        });
    }

    pub fn load_moves_for_pokemon(&mut self, detail: &PokemonDetail) {
        self.moves_loading = LoadingState::Loading;
        self.available_moves.clear();
        let tx = self.tx.clone();
        // Only load first 50 moves for perf
        let move_names: Vec<String> = detail
            .moves
            .iter()
            .take(50)
            .map(|m| m.move_info.name.clone())
            .collect();
        tokio::spawn(async move {
            let client = ApiClient::new();
            let mut moves = Vec::new();
            for name in &move_names {
                if let Ok(m) = client.fetch_move_detail(name).await {
                    moves.push(m);
                }
            }
            // Sort by power descending, then name
            moves.sort_by(|a, b| {
                b.power
                    .unwrap_or(0)
                    .cmp(&a.power.unwrap_or(0))
                    .then(a.name.cmp(&b.name))
            });
            let _ = tx.send(AppEvent::MovesLoaded(moves));
        });
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => self.handle_key(key),
            AppEvent::Tick => {}
            AppEvent::PokemonListLoaded(list) => {
                self.pokemon_list = list;
                self.list_loading = LoadingState::Loaded;
            }
            AppEvent::PokemonTypesUpdated(batch) => {
                for (id, types) in batch {
                    if let Some(p) = self.pokemon_list.iter_mut().find(|p| p.id == id) {
                        p.types = types;
                    }
                }
            }
            AppEvent::PokemonDetailLoaded(detail) => {
                self.detail = Some(detail);
                self.detail_loading = LoadingState::Loaded;
            }
            AppEvent::SpriteLoaded(id, bytes) => {
                if self.detail_pokemon_id == Some(id) {
                    self.sprite_bytes = Some(bytes);
                }
            }
            AppEvent::TypesLoaded(infos) => {
                self.type_infos = infos;
                self.type_chart_loading = LoadingState::Loaded;
            }
            AppEvent::MovesLoaded(moves) => {
                self.available_moves = moves;
                self.moves_loading = LoadingState::Loaded;
            }
            AppEvent::ApiError(msg) => {
                self.error_message = Some(msg);
                self.list_loading = LoadingState::Error;
                self.detail_loading = LoadingState::Error;
                self.type_chart_loading = LoadingState::Error;
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Dismiss error on any key
        if self.error_message.is_some() {
            self.error_message = None;
            return;
        }

        // Modal handling
        if let Some(modal) = self.modal {
            self.handle_modal_key(key, modal);
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') if !self.search_mode => {
                self.running = false;
                return;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
                return;
            }
            KeyCode::Tab if !self.search_mode => {
                let screens = Screen::all();
                let idx = self.screen.index();
                self.screen = screens[(idx + 1) % screens.len()];
                self.on_screen_enter();
                return;
            }
            KeyCode::BackTab if !self.search_mode => {
                let screens = Screen::all();
                let idx = self.screen.index();
                self.screen = screens[(idx + screens.len() - 1) % screens.len()];
                self.on_screen_enter();
                return;
            }
            KeyCode::Char(c @ '1'..='4') if !self.search_mode && self.screen != Screen::PokemonList => {
                let idx = (c as usize) - ('1' as usize);
                self.screen = Screen::all()[idx];
                self.on_screen_enter();
                return;
            }
            _ => {}
        }

        // Per-screen keys
        match self.screen {
            Screen::PokemonList => self.handle_list_key(key),
            Screen::PokemonDetail => self.handle_detail_key(key),
            Screen::TypeChart => self.handle_type_chart_key(key),
            Screen::TeamBuilder => self.handle_team_key(key),
        }
    }

    fn on_screen_enter(&mut self) {
        match self.screen {
            Screen::PokemonList => self.start_loading_list(),
            Screen::PokemonDetail => {
                // If no detail loaded, pick from list
                if self.detail.is_none() && !self.pokemon_list.is_empty() {
                    let filtered = self.filtered_list();
                    if let Some(p) = filtered.first() {
                        self.load_detail(p.id);
                    }
                }
            }
            Screen::TypeChart => self.load_types(),
            Screen::TeamBuilder => {}
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) {
        if self.search_mode {
            match key.code {
                KeyCode::Esc => {
                    self.search_mode = false;
                }
                KeyCode::Enter => {
                    self.search_mode = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.list_state = 0;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.list_state = 0;
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
            }
            KeyCode::Char('G') => {
                // Cycle through generations: None -> Gen 1 -> ... -> Gen 9 -> None
                self.generation_filter = match self.generation_filter {
                    None => Some(1),
                    Some(9) => None,
                    Some(n) => Some(n + 1),
                };
                self.list_state = 0;
            }
            KeyCode::Char('g') => {
                // Also allow lowercase 'g' to cycle
                self.generation_filter = match self.generation_filter {
                    None => Some(1),
                    Some(9) => None,
                    Some(n) => Some(n + 1),
                };
                self.list_state = 0;
            }
            KeyCode::Char('0') => {
                // Clear generation filter
                self.generation_filter = None;
                self.list_state = 0;
            }
            KeyCode::Char(c @ '1'..='9') => {
                // Direct generation selection (1-9)
                let gen = (c as u8) - b'0';
                self.generation_filter = Some(gen);
                self.list_state = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.list_state > 0 {
                    self.list_state -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.filtered_list().len().saturating_sub(1);
                if self.list_state < max {
                    self.list_state += 1;
                }
            }
            KeyCode::Enter => {
                let filtered = self.filtered_list();
                if let Some(p) = filtered.get(self.list_state) {
                    let id = p.id;
                    self.load_detail(id);
                    self.screen = Screen::PokemonDetail;
                }
            }
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::PokemonList;
            }
            KeyCode::Char('a') => {
                if let Some(ref detail) = self.detail {
                    if self.current_team().members.len() < 6 {
                        let member = TeamMember {
                            pokemon_id: detail.id,
                            pokemon_name: detail.name.clone(),
                            types: detail
                                .types
                                .iter()
                                .map(|t| t.type_info.name.clone())
                                .collect(),
                            moves: Vec::new(),
                        };
                        self.current_team_mut().members.push(member);
                        self.team_data.save();
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_type_chart_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.type_chart_scroll_y = self.type_chart_scroll_y.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.type_chart_scroll_y < 17 {
                    self.type_chart_scroll_y += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.type_chart_scroll_x = self.type_chart_scroll_x.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.type_chart_scroll_x < 17 {
                    self.type_chart_scroll_x += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_team_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.team_slot_selected > 0 {
                    self.team_slot_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.team_slot_selected < 5 {
                    self.team_slot_selected += 1;
                }
            }
            KeyCode::Enter => {
                let slot = self.team_slot_selected;
                let member_count = self.team_data.teams[self.current_team].members.len();
                if slot < member_count {
                    let member_id = self.team_data.teams[self.current_team].members[slot].pokemon_id;
                    // Open move picker for existing member
                    let detail_matches = self.detail.as_ref().map_or(false, |d| d.id == member_id);
                    if detail_matches {
                        let detail = self.detail.clone().unwrap();
                        self.modal = Some(Modal::MovePicker);
                        self.modal_selected = 0;
                        self.load_moves_for_pokemon(&detail);
                        return;
                    }
                    // Load detail then open move picker
                    self.load_detail(member_id);
                    self.modal = Some(Modal::MovePicker);
                    self.modal_selected = 0;
                } else {
                    // Open pokemon picker for empty slot
                    self.modal = Some(Modal::PokemonPicker);
                    self.modal_selected = 0;
                    self.modal_search.clear();
                    self.start_loading_list();
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                let slot = self.team_slot_selected;
                let team = &mut self.team_data.teams[self.current_team];
                if slot < team.members.len() {
                    team.members.remove(slot);
                    self.team_data.save();
                }
            }
            KeyCode::Char('n') => {
                // New team
                self.team_data.teams.push(Team {
                    name: format!("Team {}", self.team_data.teams.len() + 1),
                    members: Vec::new(),
                });
                self.current_team = self.team_data.teams.len() - 1;
                self.team_slot_selected = 0;
                self.team_data.save();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.current_team > 0 {
                    self.current_team -= 1;
                    self.team_slot_selected = 0;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.current_team < self.team_data.teams.len() - 1 {
                    self.current_team += 1;
                    self.team_slot_selected = 0;
                }
            }
            _ => {}
        }
    }

    fn handle_modal_key(&mut self, key: KeyEvent, modal: Modal) {
        match key.code {
            KeyCode::Esc => {
                self.modal = None;
                return;
            }
            _ => {}
        }

        match modal {
            Modal::PokemonPicker => self.handle_pokemon_picker_key(key),
            Modal::MovePicker => self.handle_move_picker_key(key),
        }
    }

    fn handle_pokemon_picker_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !self.search_mode => {
                if self.modal_selected > 0 {
                    self.modal_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if !self.search_mode => {
                let max = self.modal_filtered_list().len().saturating_sub(1);
                if self.modal_selected < max {
                    self.modal_selected += 1;
                }
            }
            KeyCode::Char('/') if !self.search_mode => {
                self.search_mode = true;
                self.modal_search.clear();
            }
            KeyCode::Enter => {
                if self.search_mode {
                    self.search_mode = false;
                    return;
                }
                let filtered = self.modal_filtered_list();
                if let Some(p) = filtered.get(self.modal_selected) {
                    if self.current_team().members.len() < 6 {
                        let member = TeamMember {
                            pokemon_id: p.id,
                            pokemon_name: p.name.clone(),
                            types: p.types.clone(),
                            moves: Vec::new(),
                        };
                        self.current_team_mut().members.push(member);
                        self.team_data.save();
                        self.modal = None;
                    }
                }
            }
            KeyCode::Backspace if self.search_mode => {
                self.modal_search.pop();
                self.modal_selected = 0;
            }
            KeyCode::Char(c) if self.search_mode => {
                self.modal_search.push(c);
                self.modal_selected = 0;
            }
            _ => {}
        }
    }

    pub fn modal_filtered_list(&self) -> Vec<&PokemonSummary> {
        let mut filtered: Vec<&PokemonSummary> = self.pokemon_list.iter().collect();

        // Apply generation filter
        if let Some(gen) = self.generation_filter {
            filtered.retain(|p| pokemon_generation(p.id) == gen);
        }

        // Apply search query filter
        if !self.modal_search.is_empty() {
            let q = self.modal_search.to_lowercase();
            filtered.retain(|p| p.name.contains(&q) || p.id.to_string().contains(&q));
        }

        filtered
    }

    fn handle_move_picker_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.modal_selected > 0 {
                    self.modal_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.available_moves.len().saturating_sub(1);
                if self.modal_selected < max {
                    self.modal_selected += 1;
                }
            }
            KeyCode::Enter => {
                let slot = self.team_slot_selected;
                let selected = self.modal_selected;
                if let Some(mv) = self.available_moves.get(selected) {
                    let new_move = TeamMove {
                        name: mv.name.clone(),
                        move_type: mv.move_type.name.clone(),
                        power: mv.power,
                    };
                    let team = &mut self.team_data.teams[self.current_team];
                    if slot < team.members.len() {
                        let member = &mut team.members[slot];
                        if member.moves.len() < 4 {
                            member.moves.push(new_move);
                            let full = member.moves.len() >= 4;
                            self.team_data.save();
                            if full {
                                self.modal = None;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
