use ratatui::{
    Frame,
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::app::{FocusArea, ToolsApp};

pub fn render(frame: &mut Frame, app: &mut ToolsApp) {
    let size = frame.area();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    draw_header(frame, root[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(root[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(body[0]);

    app.layout.update_wallet_area(left[0]);
    draw_wallets(frame, left[0], app);
    app.layout.update_action_area(left[1]);
    draw_actions(frame, left[1], app);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(body[1]);

    draw_detail(frame, right[0], app);
    draw_activity(frame, right[1], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &ToolsApp) {
    let mut spans = Vec::new();
    spans.push(Span::styled(
        " Galileo Tools ",
        Style::default()
            .fg(Color::Black)
            .bg(Color::LightBlue)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        app.focus_label(),
        Style::default().fg(Color::White),
    ));
    spans.push(Span::raw("  ·  "));
    spans.push(Span::styled(
        format!(
            "Mode: {}",
            if app.context.dry_run {
                "dry-run"
            } else {
                "mainnet"
            }
        ),
        if app.context.dry_run {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        },
    ));
    spans.push(Span::raw("  ·  "));
    spans.push(Span::styled(
        format!("RPC: {}", app.context.rpc_endpoint),
        Style::default().fg(Color::Gray),
    ));

    let hints = Line::from("Enter 执行 · Tab 切换 · Esc 返回 · Ctrl+C 退出");
    let text = vec![Line::from(spans), hints];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_wallets(frame: &mut Frame, area: Rect, app: &ToolsApp) {
    let focus = matches!(app.focus, FocusArea::Wallets);
    let border_style = if focus {
        Style::default().fg(Color::LightCyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = Line::from(vec![
        Span::styled("Wallets", Style::default().fg(Color::White)),
        Span::raw("  ·  "),
        Span::styled(
            format!("{} accounts", app.wallets.len()),
            Style::default().fg(Color::Gray),
        ),
    ]);
    let block = Block::default()
        .title(title)
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
            ListItem::new(Line::from(vec![
                Span::styled(title, Style::default().fg(Color::LightGreen)),
                Span::raw("  "),
                Span::styled(desc, Style::default().fg(Color::Gray)),
            ]))
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

fn draw_activity(frame: &mut Frame, area: Rect, app: &ToolsApp) {
    let block = Block::default()
        .title("Activity")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let available_height = area.height.saturating_sub(2).max(1) as usize;
    let log_lines: Vec<Line> = app
        .visible_logs(available_height)
        .into_iter()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Gray))))
        .collect();

    let logs_widget = Paragraph::new(log_lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(logs_widget, area);
}
