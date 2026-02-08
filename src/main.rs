#![allow(dead_code)]

mod api;
mod app;
mod event;
mod models;
mod sprite;
mod tui;
mod ui;

use anyhow::Result;
use app::App;
use event::EventHandler;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = tui::init()?;

    let events = EventHandler::new();
    let mut app = App::new(events.tx());

    // Kick off initial data load
    app.start_loading_list();

    let result = run(&mut terminal, &mut app, events).await;

    tui::restore()?;
    result
}

async fn run(terminal: &mut tui::Tui, app: &mut App, mut events: EventHandler) -> Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        let event = events.next().await?;
        app.handle_event(event);
    }
    Ok(())
}
