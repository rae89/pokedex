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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_event_handler_new() {
        let handler = EventHandler::new();
        // Should create channel successfully
        assert!(handler.tx.send(AppEvent::Tick).is_ok());
    }

    #[test]
    fn test_event_handler_tx() {
        let handler = EventHandler::new();
        let tx = handler.tx();

        // Should be able to send events through cloned sender
        assert!(tx.send(AppEvent::Tick).is_ok());
        assert!(tx
            .send(AppEvent::Key(KeyEvent::new(
                KeyCode::Char('q'),
                KeyModifiers::empty()
            )))
            .is_ok());
    }

    #[tokio::test]
    async fn test_event_handler_next() {
        let mut handler = EventHandler::new();
        let tx = handler.tx();

        // Send an event
        tx.send(AppEvent::Tick).unwrap();

        // Should receive it
        let event = handler.next().await.unwrap();
        match event {
            AppEvent::Tick => {}
            _ => panic!("Expected Tick event"),
        }
    }

    #[tokio::test]
    async fn test_event_handler_multiple_events() {
        let mut handler = EventHandler::new();
        let tx = handler.tx();

        tx.send(AppEvent::Tick).unwrap();
        tx.send(AppEvent::Tick).unwrap();

        let event1 = handler.next().await.unwrap();
        let event2 = handler.next().await.unwrap();

        match (event1, event2) {
            (AppEvent::Tick, AppEvent::Tick) => {}
            _ => panic!("Expected two Tick events"),
        }
    }

    #[tokio::test]
    async fn test_event_handler_key_event() {
        let mut handler = EventHandler::new();
        let tx = handler.tx();

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        tx.send(AppEvent::Key(key)).unwrap();

        let event = handler.next().await.unwrap();
        match event {
            AppEvent::Key(k) => match k.code {
                KeyCode::Char('q') => {}
                _ => panic!("Expected 'q' key"),
            },
            _ => panic!("Expected Key event"),
        }
    }

    #[tokio::test]
    async fn test_event_handler_api_events() {
        let mut handler = EventHandler::new();
        let tx = handler.tx();

        // Test PokemonListLoaded
        let summaries = vec![crate::models::pokemon::PokemonSummary {
            id: 1,
            name: "bulbasaur".to_string(),
            types: vec![],
        }];
        tx.send(AppEvent::PokemonListLoaded(summaries.clone()))
            .unwrap();

        let event = handler.next().await.unwrap();
        match event {
            AppEvent::PokemonListLoaded(list) => {
                assert_eq!(list.len(), 1);
                assert_eq!(list[0].id, 1);
            }
            _ => panic!("Expected PokemonListLoaded event"),
        }
    }
}
