use ratatui::{
    Frame,
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::app::{FocusArea, ToolsApp};

pub fn render(frame: &mut Frame, app: &mut ToolsApp) {
    let size = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(7)])
        .split(size);

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ])
        .split(vertical[0]);

    let wallets_area = header_chunks[0];
    app.layout.update_wallet_area(wallets_area);
    draw_wallets(frame, wallets_area, app);

    let actions_area = header_chunks[1];
    app.layout.update_action_area(actions_area);
    draw_actions(frame, actions_area, app);

    draw_detail(frame, header_chunks[2], app);
    draw_footer(frame, vertical[1], app);
}

fn draw_wallets(frame: &mut Frame, area: Rect, app: &ToolsApp) {
    let focus = matches!(app.focus, FocusArea::Wallets);
    let border_style = if focus {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let block = Block::default()
        .title("Wallets")
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = app
        .wallets
        .iter()
        .map(|wallet| {
            let mut spans = vec![Span::styled(
                wallet.title.clone(),
                Style::default().fg(if wallet.is_primary {
                    Color::Yellow
                } else {
                    Color::White
                }),
            )];
            if let Some(subtitle) = &wallet.subtitle {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    subtitle.clone(),
                    Style::default().fg(Color::Gray),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    if !app.wallets.is_empty() {
        state.select(Some(app.wallet_index));
    }

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list.block(block), area, &mut state);
}

fn draw_actions(frame: &mut Frame, area: Rect, app: &ToolsApp) {
    let focus = matches!(app.focus, FocusArea::Actions);
    let border_style = if focus {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default()
    };
    let block = Block::default()
        .title("Actions")
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = app
        .actions
        .iter()
        .map(|action| {
            let title = action.title();
            let desc = action.description();
            ListItem::new(vec![
                Line::styled(title, Style::default().fg(Color::LightGreen)),
                Line::styled(desc, Style::default().fg(Color::Gray)),
                Line::default(),
            ])
        })
        .collect();

    let mut state = ListState::default();
    if !app.actions.is_empty() {
        state.select(Some(app.action_index));
    }

    let list = List::new(items).highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_stateful_widget(list.block(block), area, &mut state);
}

fn draw_detail(frame: &mut Frame, area: Rect, app: &ToolsApp) {
    let detail = app.action_details();
    let block = Block::default()
        .title("Details")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightBlue));
    let paragraph = Paragraph::new(detail)
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &ToolsApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Min(10)])
        .split(area);

    let status = format!(
        "RPC: {} | Mode: {} | Focus: {}",
        app.context.rpc_endpoint,
        if app.context.dry_run {
            "dry-run"
        } else {
            "mainnet"
        },
        app.focus_label()
    );
    let status_block = Block::default()
        .title("Status")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    frame.render_widget(
        Paragraph::new(status)
            .block(status_block)
            .wrap(Wrap { trim: true }),
        chunks[0],
    );

    let logs_block = Block::default()
        .title("Activity")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let available_height = chunks[1].height.saturating_sub(2).max(1) as usize;
    let log_lines: Vec<Line> = app
        .visible_logs(available_height)
        .into_iter()
        .map(|line| Line::from(Span::raw(line)))
        .collect();

    let logs_widget = Paragraph::new(log_lines)
        .block(logs_block)
        .wrap(Wrap { trim: false });
    frame.render_widget(logs_widget, chunks[1]);
}
