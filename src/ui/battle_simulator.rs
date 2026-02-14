use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::models::battle::{BattlePhase, BattlePokemon};
use crate::ui::type_color;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let battle = &app.battle;

    match battle.phase {
        BattlePhase::SelectPokemon1 | BattlePhase::SelectPokemon2 => {
            draw_pokemon_picker(f, app, area);
        }
        BattlePhase::SelectMove | BattlePhase::Animating | BattlePhase::Finished => {
            draw_battle_arena(f, app, area);
        }
    }
}

fn draw_pokemon_picker(f: &mut Frame, app: &App, area: Rect) {
    let battle = &app.battle;
    let title = if battle.phase == BattlePhase::SelectPokemon1 {
        " ⚔ Pick Player 1's Pokémon "
    } else {
        " ⚔ Pick Player 2's Pokémon (Opponent) "
    };

    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    // Search bar
    let search_text = if battle.picker_search_mode {
        format!("Search: {}▏", battle.picker_search)
    } else if battle.picker_search.is_empty() {
        "Press / to search | ⭐ = Team members".to_string()
    } else {
        format!("Search: {} (press / to edit)", battle.picker_search)
    };
    let search = Paragraph::new(search_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    );
    f.render_widget(search, chunks[0]);

    // Pokemon list
    let list_data = app.battle_picker_list();
    let items: Vec<ListItem> = list_data
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if i == battle.picker_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if p.name.starts_with('⭐') {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            let type_str = if p.types.is_empty() {
                String::new()
            } else {
                format!(" [{}]", p.types.join("/"))
            };

            ListItem::new(format!(
                " {} #{:04} {}{}",
                if i == battle.picker_selected {
                    "▸"
                } else {
                    " "
                },
                p.id,
                p.name,
                type_str
            ))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Pokémon List ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(list, chunks[1]);
}

fn draw_battle_arena(f: &mut Frame, app: &App, area: Rect) {
    let battle = &app.battle;

    // Layout: [P1 info | P2 info] on top, [moves | battle log] on bottom
    let main_chunks =
        Layout::vertical([Constraint::Length(8), Constraint::Min(0)]).split(area);

    let top_chunks =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunks[0]);

    let bottom_chunks =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(main_chunks[1]);

    // Draw Pokémon panels
    if let Some(ref p1) = battle.pokemon1 {
        draw_pokemon_panel(f, p1, "Player 1", top_chunks[0], Color::Cyan);
    }
    if let Some(ref p2) = battle.pokemon2 {
        draw_pokemon_panel(f, p2, "Opponent", top_chunks[1], Color::Red);
    }

    // Draw move selection
    draw_move_panel(f, app, bottom_chunks[0]);

    // Draw battle log
    draw_battle_log(f, app, bottom_chunks[1]);
}

fn draw_pokemon_panel(f: &mut Frame, pokemon: &BattlePokemon, label: &str, area: Rect, color: Color) {
    let inner = Layout::vertical([
        Constraint::Length(1), // Name + types
        Constraint::Length(2), // HP bar
        Constraint::Length(1), // Stats line 1
        Constraint::Length(1), // Stats line 2
        Constraint::Min(0),
    ])
    .split(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", label))
            .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .inner(area),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", label))
        .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
    f.render_widget(block, area);

    // Name + types
    let type_spans: Vec<Span> = pokemon
        .types
        .iter()
        .map(|t| {
            Span::styled(
                format!(" {} ", t.to_uppercase()),
                Style::default()
                    .fg(Color::White)
                    .bg(type_color(t))
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect();

    let mut name_line = vec![Span::styled(
        format!(" {} ", pokemon.name.to_uppercase()),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )];
    name_line.extend(type_spans);
    f.render_widget(Paragraph::new(Line::from(name_line)), inner[0]);

    // HP bar
    let hp_pct = if pokemon.max_hp > 0 {
        (pokemon.current_hp as f64 / pokemon.max_hp as f64 * 100.0) as u16
    } else {
        0
    };
    let hp_color = match hp_pct {
        0..=25 => Color::Red,
        26..=50 => Color::Yellow,
        _ => Color::Green,
    };
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(hp_color).bg(Color::DarkGray))
        .percent(hp_pct.min(100))
        .label(format!("HP: {}/{}", pokemon.current_hp, pokemon.max_hp));
    f.render_widget(gauge, inner[1]);

    // Stats
    let stats1 = Paragraph::new(Line::from(vec![
        Span::styled(" ATK:", Style::default().fg(Color::Red)),
        Span::raw(format!("{} ", pokemon.attack)),
        Span::styled("DEF:", Style::default().fg(Color::Blue)),
        Span::raw(format!("{} ", pokemon.defense)),
        Span::styled("SPD:", Style::default().fg(Color::Green)),
        Span::raw(format!("{}", pokemon.speed)),
    ]));
    f.render_widget(stats1, inner[2]);

    let stats2 = Paragraph::new(Line::from(vec![
        Span::styled(" SPC:", Style::default().fg(Color::Magenta)),
        Span::raw(format!("{} ", pokemon.special)),
        Span::styled("LVL:", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", pokemon.level)),
    ]));
    f.render_widget(stats2, inner[3]);
}

fn draw_move_panel(f: &mut Frame, app: &App, area: Rect) {
    let battle = &app.battle;
    let is_selecting = battle.phase == BattlePhase::SelectMove;

    let moves = battle
        .pokemon1
        .as_ref()
        .map(|p| &p.moves[..])
        .unwrap_or(&[]);

    let items: Vec<ListItem> = moves
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let selected = is_selecting && i == battle.selected_move;
            let arrow = if selected { "▸" } else { " " };
            let style = if selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let type_style = Style::default()
                .fg(type_color(&m.move_type))
                .add_modifier(Modifier::BOLD);

            ListItem::new(Line::from(vec![
                Span::raw(format!(" {} ", arrow)),
                Span::styled(format!("{:<14}", m.name), style),
                Span::styled(format!(" {:>8} ", m.move_type.to_uppercase()), type_style),
                Span::styled(format!("PWR:{}", m.power), Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let title = match battle.phase {
        BattlePhase::SelectMove => " Moves (↑↓ Enter) ",
        BattlePhase::Animating => " ⚡ Battling... ",
        BattlePhase::Finished => " Press R to rematch ",
        _ => " Moves ",
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(list, area);
}

fn draw_battle_log(f: &mut Frame, app: &App, area: Rect) {
    let battle = &app.battle;

    // Show last N log entries that fit
    let max_lines = area.height.saturating_sub(2) as usize;
    let start = battle.log.len().saturating_sub(max_lines);
    let visible: Vec<Line> = battle.log[start..]
        .iter()
        .map(|entry| {
            let style = if entry.text.contains("super effective") {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if entry.text.contains("not very effective") {
                Style::default().fg(Color::Red)
            } else if entry.text.contains("doesn't affect") {
                Style::default().fg(Color::DarkGray)
            } else if entry.text.contains("fainted") {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if entry.text.contains('🏆') {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if entry.text.starts_with("──") {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            Line::styled(format!(" {}", entry.text), style)
        })
        .collect();

    let log = Paragraph::new(visible)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Battle Log ")
                .title_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(log, area);
}
