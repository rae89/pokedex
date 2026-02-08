use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, LoadingState};
use crate::ui::type_color;

const TYPE_ORDER: [&str; 18] = [
    "normal", "fire", "water", "electric", "grass", "ice", "fighting", "poison", "ground",
    "flying", "psychic", "bug", "rock", "ghost", "dragon", "dark", "steel", "fairy",
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    match app.type_chart_loading {
        LoadingState::Loading | LoadingState::Idle => {
            let loading = Paragraph::new("Loading type data...")
                .block(Block::default().borders(Borders::ALL).title(" Type Chart "))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(loading, area);
            return;
        }
        LoadingState::Error => {
            let error = Paragraph::new("Failed to load type data.")
                .block(Block::default().borders(Borders::ALL).title(" Type Chart "))
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, area);
            return;
        }
        LoadingState::Loaded => {}
    }

    // Build effectiveness matrix: matrix[attacker][defender] = multiplier
    let mut matrix = [[1.0f32; 18]; 18];
    for info in &app.type_infos {
        let atk_idx = match type_index(&info.name) {
            Some(i) => i,
            None => continue,
        };
        for target in &info.damage_relations.double_damage_to {
            if let Some(def_idx) = type_index(&target.name) {
                matrix[atk_idx][def_idx] = 2.0;
            }
        }
        for target in &info.damage_relations.half_damage_to {
            if let Some(def_idx) = type_index(&target.name) {
                matrix[atk_idx][def_idx] = 0.5;
            }
        }
        for target in &info.damage_relations.no_damage_to {
            if let Some(def_idx) = type_index(&target.name) {
                matrix[atk_idx][def_idx] = 0.0;
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Type Chart (ATK → DEF)  ↑↓←→/hjkl to scroll ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let col_width = 5usize;
    let label_width = 9usize;
    let visible_cols = ((inner.width as usize).saturating_sub(label_width)) / col_width;
    let visible_rows = (inner.height as usize).saturating_sub(2); // header + help line

    let start_col = app
        .type_chart_scroll_x
        .min(18usize.saturating_sub(visible_cols));
    let start_row = app
        .type_chart_scroll_y
        .min(18usize.saturating_sub(visible_rows));

    let mut lines: Vec<Line> = Vec::new();

    // Header row
    let mut header_spans = vec![Span::styled(
        format!("{:>width$}", "DEF→", width = label_width),
        Style::default().fg(Color::DarkGray),
    )];
    for c in start_col..(start_col + visible_cols).min(18) {
        let name = &TYPE_ORDER[c][..TYPE_ORDER[c].len().min(col_width - 1)];
        header_spans.push(Span::styled(
            format!("{:>width$}", name.to_uppercase(), width = col_width),
            Style::default()
                .fg(type_color(TYPE_ORDER[c]))
                .add_modifier(Modifier::BOLD),
        ));
    }
    lines.push(Line::from(header_spans));

    // Data rows
    for r in start_row..(start_row + visible_rows).min(18) {
        let type_name = TYPE_ORDER[r];
        let label = &type_name[..type_name.len().min(label_width - 1)];
        let mut row_spans = vec![Span::styled(
            format!("{:>width$}", label.to_uppercase(), width = label_width),
            Style::default()
                .fg(type_color(type_name))
                .add_modifier(Modifier::BOLD),
        )];
        for val in &matrix[r][start_col..(start_col + visible_cols).min(18)] {
            let (text, color) = match *val {
                v if v >= 2.0 => ("2×", Color::Rgb(80, 220, 80)),
                v if v <= 0.0 => ("0", Color::Rgb(80, 80, 80)),
                v if v < 1.0 => ("½×", Color::Rgb(220, 80, 80)),
                _ => ("·", Color::Rgb(100, 100, 100)),
            };
            row_spans.push(Span::styled(
                format!("{:>width$}", text, width = col_width),
                Style::default().fg(color),
            ));
        }
        lines.push(Line::from(row_spans));
    }

    let chart = Paragraph::new(lines);
    f.render_widget(chart, inner);
}

fn type_index(name: &str) -> Option<usize> {
    TYPE_ORDER.iter().position(|&t| t == name)
}
