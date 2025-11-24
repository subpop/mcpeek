use super::app::{App, Tab};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn render_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_tabs(f, app, chunks[0]);

    if let Some(detail) = &app.detail_view {
        render_detail(f, app, detail, chunks[1]);
    } else {
        render_content(f, app, chunks[1]);
    }

    render_help(f, app, chunks[2]);

    // Render tool call input form as overlay
    if app.tool_call_input_mode {
        render_tool_input_form(f, app);
    }

    // Render prompt input form as overlay
    if app.prompt_input_mode {
        render_prompt_input_form(f, app);
    }
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let mut tab_titles = vec![
        Tab::Tools.as_str(),
        Tab::Prompts.as_str(),
        Tab::Resources.as_str(),
        Tab::ServerInfo.as_str(),
        Tab::ServerLogs.as_str(),
    ];

    if app.debug_mode {
        tab_titles.push(Tab::DebugLogs.as_str());
    }

    let selected_index = match app.current_tab {
        Tab::Tools => 0,
        Tab::Prompts => 1,
        Tab::Resources => 2,
        Tab::ServerInfo => 3,
        Tab::ServerLogs => 4,
        Tab::DebugLogs => 5,
    };

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("mcpeek - An MCP Inspector"),
        )
        .select(selected_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn render_content(f: &mut Frame, app: &App, area: Rect) {
    if app.loading {
        let loading = Paragraph::new("Loading...")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(loading, area);
        return;
    }

    if let Some(error) = &app.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .block(Block::default().borders(Borders::ALL).title("Error"))
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });
        f.render_widget(error_widget, area);
        return;
    }

    match app.current_tab {
        Tab::Tools => render_tools(f, app, area),
        Tab::Prompts => render_prompts(f, app, area),
        Tab::Resources => render_resources(f, app, area),
        Tab::ServerInfo => render_server_info(f, app, area),
        Tab::ServerLogs => render_logs(f, app, area),
        Tab::DebugLogs => render_debug_logs(f, app, area),
    }
}

fn render_tools(f: &mut Frame, app: &App, area: Rect) {
    if app.tools.is_empty() {
        let empty = Paragraph::new("No tools available")
            .block(Block::default().borders(Borders::ALL).title("Tools"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .tools
        .iter()
        .map(|tool| {
            let content = vec![Line::from(vec![
                Span::styled(
                    &tool.name,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::raw(tool.description.as_deref().unwrap_or("No description")),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Tools ({})", app.tools.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(app.selected_tool));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_prompts(f: &mut Frame, app: &App, area: Rect) {
    if app.prompts.is_empty() {
        let empty = Paragraph::new("No prompts available")
            .block(Block::default().borders(Borders::ALL).title("Prompts"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .prompts
        .iter()
        .map(|prompt| {
            let args_count = prompt.arguments.as_ref().map(|a| a.len()).unwrap_or(0);
            let content = vec![Line::from(vec![
                Span::styled(
                    &prompt.name,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" ({} args) - ", args_count)),
                Span::raw(prompt.description.as_deref().unwrap_or("No description")),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Prompts ({})", app.prompts.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(app.selected_prompt));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_resources(f: &mut Frame, app: &App, area: Rect) {
    if app.resources.is_empty() {
        let empty = Paragraph::new("No resources available")
            .block(Block::default().borders(Borders::ALL).title("Resources"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .resources
        .iter()
        .map(|resource| {
            let content = vec![Line::from(vec![
                Span::styled(
                    &resource.name,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(&resource.uri, Style::default().fg(Color::Blue)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Resources ({})", app.resources.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(app.selected_resource));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_server_info(f: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(info) = &app.server_info {
        let caps = &info.capabilities;

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Server: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "{} v{}",
                    info.server_info.name, info.server_info.version
                )),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Protocol Version: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&info.protocol_version),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Capabilities:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ];

        if let Some(tools_cap) = &caps.tools {
            lines.push(Line::from(format!(
                "  Tools: Yes{}",
                if tools_cap.list_changed.unwrap_or(false) {
                    " (supports list changes)"
                } else {
                    ""
                }
            )));
        } else {
            lines.push(Line::from("  Tools: No"));
        }

        if let Some(prompts_cap) = &caps.prompts {
            lines.push(Line::from(format!(
                "  Prompts: Yes{}",
                if prompts_cap.list_changed.unwrap_or(false) {
                    " (supports list changes)"
                } else {
                    ""
                }
            )));
        } else {
            lines.push(Line::from("  Prompts: No"));
        }

        if let Some(resources_cap) = &caps.resources {
            let mut features = vec![];
            if resources_cap.subscribe.unwrap_or(false) {
                features.push("subscribe");
            }
            if resources_cap.list_changed.unwrap_or(false) {
                features.push("list changes");
            }
            let feature_str = if features.is_empty() {
                String::new()
            } else {
                format!(" (supports {})", features.join(", "))
            };
            lines.push(Line::from(format!("  Resources: Yes{}", feature_str)));
        } else {
            lines.push(Line::from("  Resources: No"));
        }

        if caps.logging.is_some() {
            lines.push(Line::from("  Logging: Yes"));
        } else {
            lines.push(Line::from("  Logging: No"));
        }

        Text::from(lines)
    } else {
        Text::from("No server information available")
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Server Information (↑/↓: Scroll)"),
        )
        .wrap(Wrap { trim: true })
        .scroll((app.server_info_scroll as u16, 0));

    f.render_widget(paragraph, area);
}

fn render_detail(f: &mut Frame, app: &App, detail: &str, area: Rect) {
    let paragraph = Paragraph::new(detail)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Detail View (↑/↓: Scroll | Esc: Close)"),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll as u16, 0));

    f.render_widget(paragraph, area);
}

fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    if app.logs.is_empty() {
        let empty = Paragraph::new("No logs yet. Server stderr output will appear here.")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Server Logs (stderr)"),
            )
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let log_text = app.logs.join("");

    let paragraph = Paragraph::new(log_text)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Server Logs ({} lines) - ↑/↓: Scroll | E: Jump to End | S: Save",
            app.logs.len()
        )))
        .wrap(Wrap { trim: false })
        .scroll((app.log_scroll as u16, 0));

    f.render_widget(paragraph, area);
}

fn render_debug_logs(f: &mut Frame, app: &App, area: Rect) {
    if app.debug_logs.is_empty() {
        let empty = Paragraph::new("No debug logs yet. Application debug output will appear here.")
            .block(Block::default().borders(Borders::ALL).title("Debug Logs"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    // Format debug logs with color-coding by level
    let mut lines = Vec::new();
    for entry in &app.debug_logs {
        let level_style = match entry.level.as_str() {
            "ERROR" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            "WARN" => Style::default().fg(Color::Yellow),
            "INFO" => Style::default().fg(Color::Green),
            "DEBUG" => Style::default().fg(Color::Cyan),
            "TRACE" => Style::default().fg(Color::Gray),
            _ => Style::default().fg(Color::White),
        };

        let line = Line::from(vec![
            Span::styled(
                format!("[{}] ", entry.timestamp),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(format!("{:5} ", entry.level), level_style),
            Span::styled(
                format!("{}: ", entry.target),
                Style::default().fg(Color::Blue),
            ),
            Span::raw(&entry.message),
        ]);
        lines.push(line);
    }

    let text = Text::from(lines);

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Debug Logs ({} entries) - ↑/↓: Scroll | E: Jump to End | S: Save",
            app.debug_logs.len()
        )))
        .wrap(Wrap { trim: false })
        .scroll((app.debug_log_scroll as u16, 0));

    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match (app.tool_call_input_mode, app.prompt_input_mode, &app.detail_view, app.current_tab) {
        (true, _, _, _) =>
            "TAB/Shift+TAB: Navigate Fields | ↑/↓: Scroll | Type: Enter Value | ENTER: Execute | ESC: Cancel",
        (_, true, _, _) =>
            "TAB/Shift+TAB: Navigate Fields | ↑/↓: Scroll | Type: Enter Value | ENTER: Get Prompt | ESC: Cancel",
        (_, _, Some(_), Tab::Tools) =>
            "↑/↓: Scroll | C: Call Tool | ESC: Close | Q: Quit",
        (_, _, Some(_), Tab::Prompts) =>
            "↑/↓: Scroll | C: Get Prompt | ESC: Close | Q: Quit",
        (_, _, Some(_), Tab::Resources) =>
            "↑/↓: Scroll | C: Read Resource | ESC: Close | Q: Quit",
        (_, _, Some(_), _) =>
            "↑/↓: Scroll | ESC: Close | Q: Quit",
        (_, _, None, Tab::ServerLogs) =>
            "TAB: Next Tab | ←/→: Switch Tabs | ↑/↓: Scroll | E: Jump to End | S: Save Logs | R: Refresh | Q: Quit",
        (_, _, None, Tab::DebugLogs) =>
            "TAB: Next Tab | ←/→: Switch Tabs | ↑/↓: Scroll | E: Jump to End | S: Save Logs | R: Refresh | Q: Quit",
        (_, _, None, Tab::ServerInfo) =>
            "TAB: Next Tab | ←/→: Switch Tabs | ↑/↓: Scroll | ENTER: Details | R: Refresh | Q: Quit",
        (_, _, None, Tab::Tools) =>
            "TAB: Next Tab | ←/→: Switch Tabs | ↑/↓: Navigate | ENTER: Details | C: Call Tool | R: Refresh | Q: Quit",
        (_, _, None, Tab::Prompts) =>
            "TAB: Next Tab | ←/→: Switch Tabs | ↑/↓: Navigate | ENTER: Details | C: Get Prompt | R: Refresh | Q: Quit",
        (_, _, None, Tab::Resources) =>
            "TAB: Next Tab | ←/→: Switch Tabs | ↑/↓: Navigate | ENTER: Details | C: Read Resource | R: Refresh | Q: Quit",
    };

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

fn render_tool_input_form(f: &mut Frame, app: &App) {
    // Calculate centered popup area
    let area = f.area();
    let popup_width = area.width.saturating_sub(10).min(80);
    let popup_height = (app.input_fields.len() as u16 * 3 + 8).min(area.height.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Clear the background to create a solid opaque popup
    f.render_widget(Clear, popup_area);

    // Render the block with border and background
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(format!(
            "Call Tool: {}",
            app.tools
                .get(app.selected_tool)
                .map(|t| t.name.as_str())
                .unwrap_or("")
        ))
        .style(Style::default().bg(Color::Black));
    f.render_widget(block, popup_area);

    // Inner area for content
    let inner = Rect {
        x: popup_area.x + 2,
        y: popup_area.y + 2,
        width: popup_area.width.saturating_sub(4),
        height: popup_area.height.saturating_sub(4),
    };

    if app.input_fields.is_empty() {
        // No parameters needed
        let text = vec![
            Line::from("This tool has no parameters."),
            Line::from(""),
            Line::from("Press ENTER to execute or ESC to cancel."),
        ];
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, inner);
    } else {
        // Render input fields
        let mut lines = Vec::new();

        for (i, field) in app.input_fields.iter().enumerate() {
            let is_current = i == app.input_field_index;
            let value = app
                .tool_call_inputs
                .get(&field.name)
                .map(|s| s.as_str())
                .unwrap_or("");

            let field_label = format!(
                "{} ({}{})",
                field.name,
                field.field_type,
                if field.required { ", required" } else { "" }
            );

            let label_style = if is_current {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(Span::styled(field_label, label_style)));

            if let Some(desc) = &field.description {
                lines.push(Line::from(Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(Color::Gray),
                )));
            }

            let value_style = if is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let display_value = if is_current && value.is_empty() {
                "_"
            } else if value.is_empty() {
                "(empty)"
            } else {
                value
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(display_value, value_style),
                if is_current {
                    Span::styled("█", Style::default().fg(Color::Green))
                } else {
                    Span::raw("")
                },
            ]));

            if i < app.input_fields.len() - 1 {
                lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.tool_input_scroll as u16, 0));

        f.render_widget(paragraph, inner);
    }
}

fn render_prompt_input_form(f: &mut Frame, app: &App) {
    // Calculate centered popup area
    let area = f.area();
    let popup_width = area.width.saturating_sub(10).min(80);
    let popup_height = (app.input_fields.len() as u16 * 3 + 8).min(area.height.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // Clear the background to create a solid opaque popup
    f.render_widget(Clear, popup_area);

    // Render the block with border and background
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(format!(
            "Get Prompt: {}",
            app.prompts
                .get(app.selected_prompt)
                .map(|p| p.name.as_str())
                .unwrap_or("")
        ))
        .style(Style::default().bg(Color::Black));
    f.render_widget(block, popup_area);

    // Inner area for content
    let inner = Rect {
        x: popup_area.x + 2,
        y: popup_area.y + 2,
        width: popup_area.width.saturating_sub(4),
        height: popup_area.height.saturating_sub(4),
    };

    if app.input_fields.is_empty() {
        // No parameters needed
        let text = vec![
            Line::from("This prompt has no arguments."),
            Line::from(""),
            Line::from("Press ENTER to get prompt or ESC to cancel."),
        ];
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, inner);
    } else {
        // Render input fields
        let mut lines = Vec::new();

        for (i, field) in app.input_fields.iter().enumerate() {
            let is_current = i == app.input_field_index;
            let value = app
                .prompt_inputs
                .get(&field.name)
                .map(|s| s.as_str())
                .unwrap_or("");

            let field_label = format!(
                "{} ({}{})",
                field.name,
                field.field_type,
                if field.required { ", required" } else { "" }
            );

            let label_style = if is_current {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(Span::styled(field_label, label_style)));

            if let Some(desc) = &field.description {
                lines.push(Line::from(Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(Color::Gray),
                )));
            }

            let value_style = if is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let display_value = if is_current && value.is_empty() {
                "_"
            } else if value.is_empty() {
                "(empty)"
            } else {
                value
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(display_value, value_style),
                if is_current {
                    Span::styled("█", Style::default().fg(Color::Green))
                } else {
                    Span::raw("")
                },
            ]));

            if i < app.input_fields.len() - 1 {
                lines.push(Line::from(""));
            }
        }

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.tool_input_scroll as u16, 0));

        f.render_widget(paragraph, inner);
    }
}
