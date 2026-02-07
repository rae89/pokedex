use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, LoadingState};
use crate::sprite::renderer::SpriteWidget;
use crate::ui::type_color;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    match app.detail_loading {
        LoadingState::Loading | LoadingState::Idle => {
            let loading = Paragraph::new("Loading Pokémon details...")
                .block(Block::default().borders(Borders::ALL).title(" Detail "))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(loading, area);
            return;
        }
        LoadingState::Error => {
            let error = Paragraph::new("Failed to load. Press Esc to go back.")
                .block(Block::default().borders(Borders::ALL).title(" Detail "))
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, area);
            return;
        }
        LoadingState::Loaded => {}
    }

    let detail = match &app.detail {
        Some(d) => d,
        None => return,
    };

    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    // Left: sprite
    let sprite_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" #{:03} ", detail.id));
    let sprite_inner = sprite_block.inner(chunks[0]);
    f.render_widget(sprite_block, chunks[0]);

    if let Some(ref bytes) = app.sprite_bytes {
        if let Some(widget) =
            SpriteWidget::from_png_bytes(bytes, sprite_inner.width, sprite_inner.height)
        {
            f.render_widget(&widget, sprite_inner);
        }
    }

    // Right: info
    let info_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", capitalize(&detail.name)))
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    let info_inner = info_block.inner(chunks[1]);
    f.render_widget(info_block, chunks[1]);

    let mut lines: Vec<Line> = Vec::new();

    // Types
    let mut type_spans = vec![Span::styled(
        "Types: ",
        Style::default().fg(Color::DarkGray),
    )];
    for t in &detail.types {
        type_spans.push(Span::styled(
            format!(" {} ", t.type_info.name.to_uppercase()),
            Style::default()
                .fg(Color::White)
                .bg(type_color(&t.type_info.name))
                .add_modifier(Modifier::BOLD),
        ));
        type_spans.push(Span::raw(" "));
    }
    lines.push(Line::from(type_spans));
    lines.push(Line::from(""));

    // Height/Weight
    lines.push(Line::from(vec![
        Span::styled("Height: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:.1}m", detail.height as f64 / 10.0),
            Style::default().fg(Color::White),
        ),
        Span::raw("    "),
        Span::styled("Weight: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:.1}kg", detail.weight as f64 / 10.0),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));

    // Abilities
    let mut ability_spans = vec![Span::styled(
        "Abilities: ",
        Style::default().fg(Color::DarkGray),
    )];
    for (i, a) in detail.abilities.iter().enumerate() {
        if i > 0 {
            ability_spans.push(Span::raw(", "));
        }
        let name = capitalize(&a.ability.name.replace('-', " "));
        if a.is_hidden {
            ability_spans.push(Span::styled(
                format!("{} (H)", name),
                Style::default().fg(Color::Rgb(180, 180, 180)),
            ));
        } else {
            ability_spans.push(Span::styled(name, Style::default().fg(Color::White)));
        }
    }
    lines.push(Line::from(ability_spans));
    lines.push(Line::from(""));

    // Stats
    lines.push(Line::from(Span::styled(
        "Base Stats",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let stat_names = ["HP", "Atk", "Def", "Sp.Atk", "Sp.Def", "Speed"];
    let stat_colors = [
        Color::Rgb(255, 89, 89),   // HP - red
        Color::Rgb(245, 172, 120), // Atk - orange
        Color::Rgb(250, 224, 120), // Def - yellow
        Color::Rgb(157, 183, 245), // SpAtk - blue
        Color::Rgb(167, 219, 141), // SpDef - green
        Color::Rgb(250, 146, 178), // Speed - pink
    ];

    for (i, stat) in detail.stats.iter().enumerate() {
        let label = if i < stat_names.len() {
            stat_names[i]
        } else {
            &stat.stat.name
        };
        let color = if i < stat_colors.len() {
            stat_colors[i]
        } else {
            Color::White
        };
        let bar_width = (info_inner.width as u32).saturating_sub(16).min(40);
        let filled = ((stat.base_stat as f64 / 255.0) * bar_width as f64) as usize;
        let empty = bar_width as usize - filled;
        let bar_filled = "█".repeat(filled);
        let bar_empty = "░".repeat(empty);

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<7}", label),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{:>3} ", stat.base_stat),
                Style::default().fg(Color::White),
            ),
            Span::styled(bar_filled, Style::default().fg(color)),
            Span::styled(bar_empty, Style::default().fg(Color::Rgb(60, 60, 60))),
        ]));
    }

    lines.push(Line::from(""));
    let total: u32 = detail.stats.iter().map(|s| s.base_stat).sum();
    lines.push(Line::from(vec![
        Span::styled("Total:  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", total),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[a] Add to team  |  [Esc] Back",
        Style::default().fg(Color::DarkGray),
    )));

    let info = Paragraph::new(lines);
    f.render_widget(info, info_inner);
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
