pub mod pokemon_detail;
pub mod pokemon_list;
pub mod team_builder;
pub mod type_chart;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(f.area());

    draw_tabs(f, app, chunks[0]);

    match app.screen {
        Screen::PokemonList => pokemon_list::draw(f, app, chunks[1]),
        Screen::PokemonDetail => pokemon_detail::draw(f, app, chunks[1]),
        Screen::TypeChart => type_chart::draw(f, app, chunks[1]),
        Screen::TeamBuilder => team_builder::draw(f, app, chunks[1]),
    }

    // Error overlay
    if let Some(ref msg) = app.error_message {
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area);
        let block = Block::default()
            .title(" Error ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let text = Paragraph::new(format!("{}\n\nPress any key to dismiss", msg))
            .block(block)
            .style(Style::default().fg(Color::Red));
        f.render_widget(text, area);
    }
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Span> = Screen::all()
        .iter()
        .map(|s| {
            let style = if *s == app.screen {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Span::styled(format!(" {} ", s.label()), style)
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Pokémon TUI ")
                .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        )
        .select(app.screen.index())
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(tabs, area);
}

pub fn type_color(type_name: &str) -> Color {
    match type_name {
        "normal" => Color::Rgb(168, 168, 120),
        "fire" => Color::Rgb(240, 128, 48),
        "water" => Color::Rgb(104, 144, 240),
        "electric" => Color::Rgb(248, 208, 48),
        "grass" => Color::Rgb(120, 200, 80),
        "ice" => Color::Rgb(152, 216, 216),
        "fighting" => Color::Rgb(192, 48, 40),
        "poison" => Color::Rgb(160, 64, 160),
        "ground" => Color::Rgb(224, 192, 104),
        "flying" => Color::Rgb(168, 144, 240),
        "psychic" => Color::Rgb(248, 88, 136),
        "bug" => Color::Rgb(168, 184, 32),
        "rock" => Color::Rgb(184, 160, 56),
        "ghost" => Color::Rgb(112, 88, 152),
        "dragon" => Color::Rgb(112, 56, 248),
        "dark" => Color::Rgb(112, 88, 72),
        "steel" => Color::Rgb(184, 184, 208),
        "fairy" => Color::Rgb(238, 153, 172),
        _ => Color::White,
    }
}

/// Helper to create a centered rect
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_color_all_types() {
        // Test all 18 Pokemon types
        assert!(matches!(type_color("normal"), Color::Rgb(168, 168, 120)));
        assert!(matches!(type_color("fire"), Color::Rgb(240, 128, 48)));
        assert!(matches!(type_color("water"), Color::Rgb(104, 144, 240)));
        assert!(matches!(type_color("electric"), Color::Rgb(248, 208, 48)));
        assert!(matches!(type_color("grass"), Color::Rgb(120, 200, 80)));
        assert!(matches!(type_color("ice"), Color::Rgb(152, 216, 216)));
        assert!(matches!(type_color("fighting"), Color::Rgb(192, 48, 40)));
        assert!(matches!(type_color("poison"), Color::Rgb(160, 64, 160)));
        assert!(matches!(type_color("ground"), Color::Rgb(224, 192, 104)));
        assert!(matches!(type_color("flying"), Color::Rgb(168, 144, 240)));
        assert!(matches!(type_color("psychic"), Color::Rgb(248, 88, 136)));
        assert!(matches!(type_color("bug"), Color::Rgb(168, 184, 32)));
        assert!(matches!(type_color("rock"), Color::Rgb(184, 160, 56)));
        assert!(matches!(type_color("ghost"), Color::Rgb(112, 88, 152)));
        assert!(matches!(type_color("dragon"), Color::Rgb(112, 56, 248)));
        assert!(matches!(type_color("dark"), Color::Rgb(112, 88, 72)));
        assert!(matches!(type_color("steel"), Color::Rgb(184, 184, 208)));
        assert!(matches!(type_color("fairy"), Color::Rgb(238, 153, 172)));
    }

    #[test]
    fn test_type_color_unknown_type() {
        assert!(matches!(type_color("unknown"), Color::White));
        assert!(matches!(type_color(""), Color::White));
        assert!(matches!(type_color("invalid"), Color::White));
    }

    #[test]
    fn test_centered_rect() {
        let parent = Rect::new(0, 0, 100, 50);

        // Test 50% x 50% rect
        // Note: Layout uses integer division, so values may vary slightly
        let rect = centered_rect(50, 50, parent);
        // Width should be approximately 50 (may be 49-51 due to rounding)
        assert!(rect.width >= 49 && rect.width <= 51);
        // Height should be approximately 25 (may be 24-26 due to rounding)
        assert!(rect.height >= 24 && rect.height <= 26);
        // X should center the width
        assert!(rect.x >= 24 && rect.x <= 26);
        // Y should center the height
        assert!(rect.y >= 11 && rect.y <= 13);
    }

    #[test]
    fn test_centered_rect_small_percentage() {
        let parent = Rect::new(0, 0, 100, 50);

        // Test 20% x 20% rect
        let rect = centered_rect(20, 20, parent);
        assert_eq!(rect.x, 40);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 20);
        assert_eq!(rect.height, 10);
    }

    #[test]
    fn test_centered_rect_large_percentage() {
        let parent = Rect::new(0, 0, 100, 50);

        // Test 90% x 90% rect
        // Note: Layout uses integer division, so values may vary slightly
        let rect = centered_rect(90, 90, parent);
        // Width should be approximately 90 (may be 89-91 due to rounding)
        assert!(rect.width >= 89 && rect.width <= 91);
        // Height should be approximately 45 (may be 44-46 due to rounding)
        assert!(rect.height >= 44 && rect.height <= 46);
        // X should center the width
        assert!(rect.x >= 4 && rect.x <= 6);
        // Y should center the height
        assert!(rect.y >= 1 && rect.y <= 3);
    }

    #[test]
    fn test_centered_rect_full_size() {
        let parent = Rect::new(0, 0, 100, 50);

        // Test 100% x 100% rect (should still be centered, but fills parent)
        let rect = centered_rect(100, 100, parent);
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 50);
    }

    #[test]
    fn test_centered_rect_odd_dimensions() {
        let parent = Rect::new(0, 0, 101, 51);

        // Test with odd parent dimensions
        let rect = centered_rect(50, 50, parent);
        // Should handle odd dimensions gracefully
        assert!(rect.width <= 101);
        assert!(rect.height <= 51);
    }
}
