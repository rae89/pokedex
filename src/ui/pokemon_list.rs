use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, LoadingState};
use crate::ui::type_color;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).split(area);

    // Search bar with generation filter
    let gen_filter_text = match app.generation_filter {
        Some(gen) => format!("Gen {}", gen),
        None => "All Gens".to_string(),
    };
    
    let search_text = if app.search_mode {
        format!("🔍 Search: {}▌", app.search_query)
    } else if !app.search_query.is_empty() {
        format!("🔍 Filter: {} (press / to edit)", app.search_query)
    } else {
        "Press / to search  |  ↑↓/jk navigate  |  Enter select".to_string()
    };
    
    let filter_line = format!("Generation: {} (G to cycle, 1-9 to select, 0 to clear)", gen_filter_text);
    let full_text = format!("{}\n{}", search_text, filter_line);
    
    let search_block = Block::default().borders(Borders::ALL).title(" Search ");
    let search = Paragraph::new(full_text)
        .block(search_block)
        .style(if app.search_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });
    f.render_widget(search, chunks[0]);

    // Pokemon list
    match app.list_loading {
        LoadingState::Loading | LoadingState::Idle => {
            let loading = Paragraph::new("Loading Pokémon list...")
                .block(Block::default().borders(Borders::ALL).title(" Pokédex "))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(loading, chunks[1]);
        }
        LoadingState::Error => {
            let error = Paragraph::new("Failed to load. Press any key.")
                .block(Block::default().borders(Borders::ALL).title(" Pokédex "))
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, chunks[1]);
        }
        LoadingState::Loaded => {
            let filtered = app.filtered_list();
            let visible_height = chunks[1].height.saturating_sub(2) as usize;

            // Calculate scroll offset to keep selection visible
            let scroll_offset = if app.list_state >= visible_height {
                app.list_state - visible_height + 1
            } else {
                0
            };

            let items: Vec<ListItem> = filtered
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(visible_height)
                .map(|(i, p)| {
                    let selected = i == app.list_state;
                    let mut spans = vec![
                        Span::styled(
                            format!("#{:03} ", p.id),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            format!("{:<12} ", capitalize(&p.name)),
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
                        spans.push(Span::styled(
                            format!(" {} ", t.to_uppercase()),
                            Style::default()
                                .fg(Color::White)
                                .bg(type_color(t))
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

            let title = format!(
                " Pokédex ({}/{}) ",
                filtered.len(),
                app.pokemon_list.len()
            );
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title));
            f.render_widget(list, chunks[1]);
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
