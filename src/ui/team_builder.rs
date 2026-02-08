use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, LoadingState, Modal};
use crate::models::pokemon::PokemonSummary;
use crate::ui::{centered_rect, type_color};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(area);

    draw_team_slots(f, app, chunks[0]);
    draw_coverage(f, app, chunks[1]);

    // Modals
    if let Some(modal) = app.modal {
        match modal {
            Modal::PokemonPicker => draw_pokemon_picker(f, app),
            Modal::MovePicker => draw_move_picker(f, app),
        }
    }
}

fn draw_team_slots(f: &mut Frame, app: &App, area: Rect) {
    let team = app.current_team();
    let title = format!(
        " {} ({}/{})  ←→ switch  n=new  d=delete ",
        team.name,
        team.members.len(),
        6
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut items: Vec<ListItem> = Vec::new();
    for i in 0..6 {
        let selected = i == app.team_slot_selected;
        let line = if i < team.members.len() {
            let member = &team.members[i];
            let mut spans = vec![
                Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:<12}", capitalize(&member.pokemon_name)),
                    Style::default().fg(Color::White).add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
            ];
            for t in &member.types {
                spans.push(Span::styled(
                    format!(" {} ", t.to_uppercase()),
                    Style::default()
                        .fg(Color::White)
                        .bg(type_color(t))
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(" "));
            }
            if !member.moves.is_empty() {
                spans.push(Span::raw("  "));
                for (j, mv) in member.moves.iter().enumerate() {
                    if j > 0 {
                        spans.push(Span::styled(", ", Style::default().fg(Color::DarkGray)));
                    }
                    spans.push(Span::styled(
                        capitalize(&mv.name.replace('-', " ")),
                        Style::default().fg(type_color(&mv.move_type)),
                    ));
                }
            }
            Line::from(spans)
        } else {
            Line::from(vec![
                Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::DarkGray)),
                Span::styled("(empty)", Style::default().fg(Color::Rgb(80, 80, 80))),
            ])
        };

        let item = if selected {
            ListItem::new(line).style(Style::default().bg(Color::Rgb(40, 40, 60)))
        } else {
            ListItem::new(line)
        };
        items.push(item);
    }

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_coverage(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Type Coverage ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let team = app.current_team();
    if team.members.is_empty() {
        let text = Paragraph::new("Add Pokémon to see type coverage")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(text, inner);
        return;
    }

    // Collect all types on the team
    let team_types: Vec<&str> = team
        .members
        .iter()
        .flat_map(|m| m.types.iter().map(|s| s.as_str()))
        .collect();

    // Collect all move types
    let move_types: Vec<&str> = team
        .members
        .iter()
        .flat_map(|m| m.moves.iter().map(|mv| mv.move_type.as_str()))
        .collect();

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Team Types",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));

    // Unique types on team
    let mut unique_types: Vec<&str> = team_types.clone();
    unique_types.sort();
    unique_types.dedup();
    let type_spans: Vec<Span> = unique_types
        .iter()
        .flat_map(|t| {
            vec![
                Span::styled(
                    format!(" {} ", t.to_uppercase()),
                    Style::default()
                        .fg(Color::White)
                        .bg(type_color(t))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ]
        })
        .collect();
    lines.push(Line::from(type_spans));
    lines.push(Line::from(""));

    // Move coverage
    if !move_types.is_empty() {
        lines.push(Line::from(Span::styled(
            "Move Coverage",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        let mut unique_move_types: Vec<&str> = move_types;
        unique_move_types.sort();
        unique_move_types.dedup();
        let move_spans: Vec<Span> = unique_move_types
            .iter()
            .flat_map(|t| {
                vec![
                    Span::styled(
                        format!(" {} ", t.to_uppercase()),
                        Style::default()
                            .fg(Color::White)
                            .bg(type_color(t))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                ]
            })
            .collect();
        lines.push(Line::from(move_spans));
        lines.push(Line::from(""));

        // Uncovered types (no super-effective move against)
        let all_types = [
            "normal", "fire", "water", "electric", "grass", "ice", "fighting", "poison", "ground",
            "flying", "psychic", "bug", "rock", "ghost", "dragon", "dark", "steel", "fairy",
        ];

        // Simple effectiveness check based on move types
        let covered: Vec<&str> = all_types
            .iter()
            .filter(|def| {
                unique_move_types
                    .iter()
                    .any(|atk| is_super_effective(atk, def))
            })
            .copied()
            .collect();

        let uncovered: Vec<&str> = all_types
            .iter()
            .filter(|t| !covered.contains(t))
            .copied()
            .collect();

        if !uncovered.is_empty() {
            lines.push(Line::from(Span::styled(
                "Not Super Effective Against",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            let uncov_spans: Vec<Span> = uncovered
                .iter()
                .flat_map(|t| {
                    vec![
                        Span::styled(
                            format!(" {} ", t.to_uppercase()),
                            Style::default().fg(Color::White).bg(type_color(t)),
                        ),
                        Span::raw(" "),
                    ]
                })
                .collect();
            lines.push(Line::from(uncov_spans));
        }
    }

    let text = Paragraph::new(lines);
    f.render_widget(text, inner);
}

fn draw_pokemon_picker(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let title = if app.search_mode {
        format!(" Pick Pokémon (🔍 {}▌) ", app.modal_search)
    } else {
        " Pick Pokémon (/ to search, Enter to select) ".to_string()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let filtered = app.modal_filtered_list();
    let visible_height = inner.height as usize;

    let scroll_offset = if app.modal_selected >= visible_height {
        app.modal_selected - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, p)| {
            let p: &PokemonSummary = p;
            let selected = i == app.modal_selected;
            let mut spans = vec![
                Span::styled(
                    format!("#{:03} ", p.id),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<12}", capitalize(&p.name)),
                    if selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ];
            for t in &p.types {
                let type_name: &str = t.as_str();
                spans.push(Span::styled(
                    format!(" {} ", type_name.to_uppercase()),
                    Style::default()
                        .fg(Color::White)
                        .bg(type_color(type_name))
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(" "));
            }
            let line = Line::from(spans);
            if selected {
                ListItem::new(line).style(Style::default().bg(Color::Rgb(40, 40, 60)))
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_move_picker(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Pick Moves (Enter to add, Esc to close) ")
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.moves_loading != LoadingState::Loaded {
        let loading = Paragraph::new("Loading moves...").style(Style::default().fg(Color::Yellow));
        f.render_widget(loading, inner);
        return;
    }

    let visible_height = inner.height as usize;
    let scroll_offset = if app.modal_selected >= visible_height {
        app.modal_selected - visible_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .available_moves
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(i, mv)| {
            let selected = i == app.modal_selected;
            let power_str = mv
                .power
                .map(|p| format!("{:>3}", p))
                .unwrap_or_else(|| " --".to_string());
            let class = mv
                .damage_class
                .as_ref()
                .map(|c| c.name.clone())
                .unwrap_or_default();
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<20}", capitalize(&mv.name.replace('-', " "))),
                    if selected {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
                Span::styled(
                    format!(" {} ", mv.move_type.name.to_uppercase()),
                    Style::default()
                        .fg(Color::White)
                        .bg(type_color(&mv.move_type.name))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  Pow:{}", power_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("  {:<8}", class),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            if selected {
                ListItem::new(line).style(Style::default().bg(Color::Rgb(40, 40, 60)))
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

/// Simplified super-effectiveness lookup for coverage display
fn is_super_effective(atk: &str, def: &str) -> bool {
    matches!(
        (atk, def),
        ("fire", "grass")
            | ("fire", "ice")
            | ("fire", "bug")
            | ("fire", "steel")
            | ("water", "fire")
            | ("water", "ground")
            | ("water", "rock")
            | ("electric", "water")
            | ("electric", "flying")
            | ("grass", "water")
            | ("grass", "ground")
            | ("grass", "rock")
            | ("ice", "grass")
            | ("ice", "ground")
            | ("ice", "flying")
            | ("ice", "dragon")
            | ("fighting", "normal")
            | ("fighting", "ice")
            | ("fighting", "rock")
            | ("fighting", "dark")
            | ("fighting", "steel")
            | ("poison", "grass")
            | ("poison", "fairy")
            | ("ground", "fire")
            | ("ground", "electric")
            | ("ground", "poison")
            | ("ground", "rock")
            | ("ground", "steel")
            | ("flying", "grass")
            | ("flying", "fighting")
            | ("flying", "bug")
            | ("psychic", "fighting")
            | ("psychic", "poison")
            | ("bug", "grass")
            | ("bug", "psychic")
            | ("bug", "dark")
            | ("rock", "fire")
            | ("rock", "ice")
            | ("rock", "flying")
            | ("rock", "bug")
            | ("ghost", "psychic")
            | ("ghost", "ghost")
            | ("dragon", "dragon")
            | ("dark", "psychic")
            | ("dark", "ghost")
            | ("steel", "ice")
            | ("steel", "rock")
            | ("steel", "fairy")
            | ("fairy", "fighting")
            | ("fairy", "dragon")
            | ("fairy", "dark")
    )
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
