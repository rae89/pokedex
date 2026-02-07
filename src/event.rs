use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::models::pokemon::{MoveDetail, PokemonDetail, PokemonSummary};
use crate::models::type_data::TypeInfo;

/// All events the app can receive
#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    // API responses
    PokemonListLoaded(Vec<PokemonSummary>),
    PokemonTypesUpdated(Vec<(u32, Vec<String>)>), // batch of (id, types)
    PokemonDetailLoaded(Box<PokemonDetail>),
    SpriteLoaded(u32, Vec<u8>), // pokemon_id, png bytes
    TypesLoaded(Vec<TypeInfo>),
    MovesLoaded(Vec<MoveDetail>),
    ApiError(String),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
    tx: mpsc::UnboundedSender<AppEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let poll_tx = tx.clone();
        // Input polling thread
        std::thread::spawn(move || loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(CrosstermEvent::Key(key)) = event::read() {
                    if poll_tx.send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
            }
        });

        Self { rx, tx }
    }

    pub fn tx(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.tx.clone()
    }

    pub async fn next(&mut self) -> Result<AppEvent> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}
