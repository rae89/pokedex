use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, LoadingState};
use crate::models::pokemon::EvolutionChainLink;
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

    // Evolution chain
    lines.push(Line::from(""));
    if let Some(ref chain) = app.evolution_chain {
        lines.push(Line::from(Span::styled(
            "Evolution",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        let paths = flatten_evolution_paths(&chain.chain);
        let current_name = &detail.name;

        if paths.len() == 1 && paths[0].len() == 1 {
            // Single-stage Pokemon, no evolutions
            lines.push(Line::from(Span::styled(
                "Does not evolve",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for path in &paths {
                let mut spans: Vec<Span> = Vec::new();
                for (i, (name, method)) in path.iter().enumerate() {
                    if i > 0 {
                        let method_str = method.as_deref().unwrap_or("???");
                        spans.push(Span::styled(
                            format!(" → ({}) ", method_str),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    let display_name = capitalize(name);
                    if name == current_name {
                        spans.push(Span::styled(
                            display_name,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ));
                    } else {
                        spans.push(Span::styled(
                            display_name,
                            Style::default().fg(Color::White),
                        ));
                    }
                }
                lines.push(Line::from(spans));
            }
        }
    } else if app.evolution_chain_loading == LoadingState::Loading {
        lines.push(Line::from(Span::styled(
            "Loading evolution chain...",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[a] Add to team  |  [\u{2190}\u{2192}] Nav  |  [e/E] Evo  |  [Esc] Back",
        Style::default().fg(Color::DarkGray),
    )));

    let info = Paragraph::new(lines);
    f.render_widget(info, info_inner);
}

/// Flatten the recursive evolution chain into linear paths.
/// Each path is a Vec of (species_name, evolution_method_description).
/// The first entry in each path has method=None (base form).
fn flatten_evolution_paths(link: &EvolutionChainLink) -> Vec<Vec<(String, Option<String>)>> {
    let mut results = Vec::new();
    collect_paths(link, &mut Vec::new(), &mut results);
    results
}

fn collect_paths(
    link: &EvolutionChainLink,
    current: &mut Vec<(String, Option<String>)>,
    results: &mut Vec<Vec<(String, Option<String>)>>,
) {
    let method = if current.is_empty() {
        None
    } else {
        Some(format_evolution_method(link))
    };
    current.push((link.species.name.clone(), method));

    if link.evolves_to.is_empty() {
        results.push(current.clone());
    } else {
        for next in &link.evolves_to {
            collect_paths(next, current, results);
        }
    }

    current.pop();
}

fn format_evolution_method(link: &EvolutionChainLink) -> String {
    let detail = match link.evolution_details.first() {
        Some(d) => d,
        None => return "???".to_string(),
    };

    let trigger = &detail.trigger.name;
    match trigger.as_str() {
        "level-up" => {
            if let Some(level) = detail.min_level {
                return format!("Lv. {}", level);
            }
            if let Some(happiness) = detail.min_happiness {
                return format!("Happiness {}", happiness);
            }
            if let Some(ref mv) = detail.known_move {
                return format!("Know {}", capitalize(&mv.name));
            }
            if let Some(ref loc) = detail.location {
                return format!("At {}", capitalize(&loc.name));
            }
            if let Some(ref tod) = detail.time_of_day {
                if !tod.is_empty() {
                    return format!("Level up ({})", tod);
                }
            }
            "Level up".to_string()
        }
        "use-item" => {
            if let Some(ref item) = detail.item {
                capitalize(&item.name.replace('-', " "))
            } else {
                "Use item".to_string()
            }
        }
        "trade" => {
            if let Some(ref species) = detail.trade_species {
                format!("Trade for {}", capitalize(&species.name))
            } else if let Some(ref item) = detail.held_item {
                format!("Trade w/ {}", capitalize(&item.name.replace('-', " ")))
            } else {
                "Trade".to_string()
            }
        }
        other => capitalize(&other.replace('-', " ")),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pokemon::{EvolutionDetail, NamedResource};

    fn named(name: &str) -> NamedResource {
        NamedResource {
            name: name.to_string(),
            url: String::new(),
        }
    }

    fn level_up_detail(level: u32) -> EvolutionDetail {
        EvolutionDetail {
            trigger: named("level-up"),
            min_level: Some(level),
            item: None,
            held_item: None,
            min_happiness: None,
            known_move: None,
            location: None,
            time_of_day: None,
            trade_species: None,
        }
    }

    fn empty_detail_with_trigger(trigger: &str) -> EvolutionDetail {
        EvolutionDetail {
            trigger: named(trigger),
            min_level: None,
            item: None,
            held_item: None,
            min_happiness: None,
            known_move: None,
            location: None,
            time_of_day: None,
            trade_species: None,
        }
    }

    // -- flatten_evolution_paths tests --

    #[test]
    fn test_flatten_linear_three_stage_chain() {
        // Bulbasaur → Ivysaur (Lv. 16) → Venusaur (Lv. 32)
        let chain = EvolutionChainLink {
            species: named("bulbasaur"),
            evolution_details: vec![],
            evolves_to: vec![EvolutionChainLink {
                species: named("ivysaur"),
                evolution_details: vec![level_up_detail(16)],
                evolves_to: vec![EvolutionChainLink {
                    species: named("venusaur"),
                    evolution_details: vec![level_up_detail(32)],
                    evolves_to: vec![],
                }],
            }],
        };

        let paths = flatten_evolution_paths(&chain);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 3);
        assert_eq!(paths[0][0], ("bulbasaur".to_string(), None));
        assert_eq!(
            paths[0][1],
            ("ivysaur".to_string(), Some("Lv. 16".to_string()))
        );
        assert_eq!(
            paths[0][2],
            ("venusaur".to_string(), Some("Lv. 32".to_string()))
        );
    }

    #[test]
    fn test_flatten_branching_evolution() {
        // Eevee → Vaporeon (Water Stone) / Jolteon (Thunder Stone)
        let chain = EvolutionChainLink {
            species: named("eevee"),
            evolution_details: vec![],
            evolves_to: vec![
                EvolutionChainLink {
                    species: named("vaporeon"),
                    evolution_details: vec![EvolutionDetail {
                        item: Some(named("water-stone")),
                        ..empty_detail_with_trigger("use-item")
                    }],
                    evolves_to: vec![],
                },
                EvolutionChainLink {
                    species: named("jolteon"),
                    evolution_details: vec![EvolutionDetail {
                        item: Some(named("thunder-stone")),
                        ..empty_detail_with_trigger("use-item")
                    }],
                    evolves_to: vec![],
                },
            ],
        };

        let paths = flatten_evolution_paths(&chain);
        assert_eq!(paths.len(), 2);
        // Each path starts with Eevee
        assert_eq!(paths[0][0].0, "eevee");
        assert_eq!(paths[1][0].0, "eevee");
        // First branch: Vaporeon
        assert_eq!(paths[0][1].0, "vaporeon");
        assert_eq!(paths[0][1].1, Some("Water stone".to_string()));
        // Second branch: Jolteon
        assert_eq!(paths[1][1].0, "jolteon");
        assert_eq!(paths[1][1].1, Some("Thunder stone".to_string()));
    }

    #[test]
    fn test_flatten_single_stage_pokemon() {
        // Tauros — does not evolve
        let chain = EvolutionChainLink {
            species: named("tauros"),
            evolution_details: vec![],
            evolves_to: vec![],
        };

        let paths = flatten_evolution_paths(&chain);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 1);
        assert_eq!(paths[0][0], ("tauros".to_string(), None));
    }

    #[test]
    fn test_flatten_two_stage_chain() {
        // Pikachu → Raichu (Thunder Stone)
        let chain = EvolutionChainLink {
            species: named("pichu"),
            evolution_details: vec![],
            evolves_to: vec![EvolutionChainLink {
                species: named("pikachu"),
                evolution_details: vec![EvolutionDetail {
                    min_happiness: Some(220),
                    ..empty_detail_with_trigger("level-up")
                }],
                evolves_to: vec![EvolutionChainLink {
                    species: named("raichu"),
                    evolution_details: vec![EvolutionDetail {
                        item: Some(named("thunder-stone")),
                        ..empty_detail_with_trigger("use-item")
                    }],
                    evolves_to: vec![],
                }],
            }],
        };

        let paths = flatten_evolution_paths(&chain);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 3);
        assert_eq!(paths[0][0].0, "pichu");
        assert_eq!(paths[0][1].0, "pikachu");
        assert_eq!(paths[0][1].1, Some("Happiness 220".to_string()));
        assert_eq!(paths[0][2].0, "raichu");
        assert_eq!(paths[0][2].1, Some("Thunder stone".to_string()));
    }

    // -- format_evolution_method tests --

    #[test]
    fn test_format_level_up_with_min_level() {
        let link = EvolutionChainLink {
            species: named("ivysaur"),
            evolution_details: vec![level_up_detail(16)],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Lv. 16");
    }

    #[test]
    fn test_format_level_up_with_happiness() {
        let link = EvolutionChainLink {
            species: named("pikachu"),
            evolution_details: vec![EvolutionDetail {
                min_happiness: Some(220),
                ..empty_detail_with_trigger("level-up")
            }],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Happiness 220");
    }

    #[test]
    fn test_format_level_up_with_known_move() {
        let link = EvolutionChainLink {
            species: named("sylveon"),
            evolution_details: vec![EvolutionDetail {
                known_move: Some(named("baby-doll-eyes")),
                ..empty_detail_with_trigger("level-up")
            }],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Know Baby-doll-eyes");
    }

    #[test]
    fn test_format_level_up_with_location() {
        let link = EvolutionChainLink {
            species: named("leafeon"),
            evolution_details: vec![EvolutionDetail {
                location: Some(named("eterna-forest")),
                ..empty_detail_with_trigger("level-up")
            }],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "At Eterna-forest");
    }

    #[test]
    fn test_format_level_up_with_time_of_day() {
        let link = EvolutionChainLink {
            species: named("espeon"),
            evolution_details: vec![EvolutionDetail {
                time_of_day: Some("day".to_string()),
                ..empty_detail_with_trigger("level-up")
            }],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Level up (day)");
    }

    #[test]
    fn test_format_level_up_plain() {
        let link = EvolutionChainLink {
            species: named("something"),
            evolution_details: vec![empty_detail_with_trigger("level-up")],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Level up");
    }

    #[test]
    fn test_format_use_item() {
        let link = EvolutionChainLink {
            species: named("vaporeon"),
            evolution_details: vec![EvolutionDetail {
                item: Some(named("water-stone")),
                ..empty_detail_with_trigger("use-item")
            }],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Water stone");
    }

    #[test]
    fn test_format_use_item_no_item() {
        let link = EvolutionChainLink {
            species: named("something"),
            evolution_details: vec![empty_detail_with_trigger("use-item")],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Use item");
    }

    #[test]
    fn test_format_trade() {
        let link = EvolutionChainLink {
            species: named("machamp"),
            evolution_details: vec![empty_detail_with_trigger("trade")],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Trade");
    }

    #[test]
    fn test_format_trade_with_species() {
        let link = EvolutionChainLink {
            species: named("escavalier"),
            evolution_details: vec![EvolutionDetail {
                trade_species: Some(named("shelmet")),
                ..empty_detail_with_trigger("trade")
            }],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Trade for Shelmet");
    }

    #[test]
    fn test_format_trade_with_held_item() {
        let link = EvolutionChainLink {
            species: named("steelix"),
            evolution_details: vec![EvolutionDetail {
                held_item: Some(named("metal-coat")),
                ..empty_detail_with_trigger("trade")
            }],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Trade w/ Metal coat");
    }

    #[test]
    fn test_format_no_evolution_details() {
        let link = EvolutionChainLink {
            species: named("something"),
            evolution_details: vec![],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "???");
    }

    #[test]
    fn test_format_other_trigger() {
        let link = EvolutionChainLink {
            species: named("something"),
            evolution_details: vec![empty_detail_with_trigger("shed")],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Shed");
    }

    #[test]
    fn test_format_hyphenated_trigger() {
        let link = EvolutionChainLink {
            species: named("something"),
            evolution_details: vec![empty_detail_with_trigger("spin-type")],
            evolves_to: vec![],
        };
        assert_eq!(format_evolution_method(&link), "Spin type");
    }
}
