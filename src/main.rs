mod logging;
mod mcp;
mod tui;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use logging::{LogBuffer, LogBufferLayer};
use mcp::McpClient;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tui::{render_ui, App};

#[derive(Parser)]
#[command(name = "mcpeek")]
#[command(about = "MCP Server Inspector - Interactive TUI for Model Context Protocol servers", long_about = None)]
struct Cli {
    #[arg(help = "Command to run the MCP server")]
    command: String,

    #[arg(help = "Arguments to pass to the server command")]
    args: Vec<String>,

    #[arg(short, long, help = "Enable debug logging")]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.debug { Level::DEBUG } else { Level::INFO };

    // Create custom log buffer to capture logs in memory
    let log_buffer = LogBuffer::new();
    let log_buffer_layer = LogBufferLayer::new(log_buffer.clone());

    // Initialize tracing with custom layer instead of stderr
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            log_level,
        ))
        .with(log_buffer_layer)
        .init();

    run_tui(&cli.command, &cli.args, log_buffer, cli.debug).await?;

    Ok(())
}

async fn run_tui(
    command: &str,
    args: &[String],
    log_buffer: LogBuffer,
    debug_mode: bool,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client = McpClient::new(command, args)
        .await
        .context("Failed to create MCP client")?;

    client
        .initialize()
        .await
        .context("Failed to initialize MCP client")?;

    let mut app = App::new(debug_mode);
    let res = run_tui_loop(&mut terminal, &mut app, &client, log_buffer).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    client.shutdown().await?;

    res
}

async fn run_tui_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    client: &McpClient,
    log_buffer: LogBuffer,
) -> Result<()> {
    app.load_data(client).await?;

    loop {
        // Update logs in the background
        app.update_logs(client).await;

        // Update debug logs from buffer
        app.update_debug_logs(log_buffer.get_all());

        terminal.draw(|f| render_ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.tool_call_input_mode {
                        // Handle tool call input mode
                        match key.code {
                            KeyCode::Esc => app.cancel_tool_call(),
                            KeyCode::Enter => {
                                app.execute_tool_call(client).await;
                            }
                            KeyCode::Tab => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    app.previous_input_field();
                                } else {
                                    app.next_input_field();
                                }
                            }
                            KeyCode::BackTab => app.previous_input_field(),
                            KeyCode::Backspace => app.delete_current_input(),
                            KeyCode::Up => app.scroll_tool_input_up(),
                            KeyCode::Down => app.scroll_tool_input_down(),
                            KeyCode::Char(c) => app.update_current_input(c),
                            _ => {}
                        }
                    } else if app.prompt_input_mode {
                        // Handle prompt input mode
                        match key.code {
                            KeyCode::Esc => app.cancel_prompt_input(),
                            KeyCode::Enter => {
                                app.execute_prompt_get(client).await;
                            }
                            KeyCode::Tab => {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    app.previous_input_field();
                                } else {
                                    app.next_input_field();
                                }
                            }
                            KeyCode::BackTab => app.previous_input_field(),
                            KeyCode::Backspace => app.delete_current_input(),
                            KeyCode::Up => app.scroll_tool_input_up(),
                            KeyCode::Down => app.scroll_tool_input_down(),
                            KeyCode::Char(c) => app.update_current_input(c),
                            _ => {}
                        }
                    } else if app.detail_view.is_some() {
                        match key.code {
                            KeyCode::Esc => app.close_detail(),
                            KeyCode::Char('q') | KeyCode::Char('Q') => app.quit(),
                            KeyCode::Char('c') | KeyCode::Char('C') => match app.current_tab {
                                tui::Tab::Tools => app.start_tool_call(),
                                tui::Tab::Prompts => app.start_prompt_get(),
                                tui::Tab::Resources => app.read_resource(client).await,
                                _ => {}
                            },
                            KeyCode::Down => app.next_item(),
                            KeyCode::Up => app.previous_item(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::PageUp => app.page_up(),
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') => app.quit(),
                            KeyCode::Char('c') | KeyCode::Char('C') => match app.current_tab {
                                tui::Tab::Tools => app.start_tool_call(),
                                tui::Tab::Prompts => app.start_prompt_get(),
                                tui::Tab::Resources => app.read_resource(client).await,
                                _ => {}
                            },
                            KeyCode::Tab => {
                                app.current_tab = app.current_tab.next(app.debug_mode);
                                app.load_data(client).await?;
                            }
                            KeyCode::BackTab => {
                                app.current_tab = app.current_tab.previous(app.debug_mode);
                                app.load_data(client).await?;
                            }
                            KeyCode::Left => {
                                app.current_tab = app.current_tab.previous(app.debug_mode);
                                app.load_data(client).await?;
                            }
                            KeyCode::Right => {
                                app.current_tab = app.current_tab.next(app.debug_mode);
                                app.load_data(client).await?;
                            }
                            KeyCode::Down => app.next_item(),
                            KeyCode::Up => app.previous_item(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::Enter => app.show_detail(),
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                app.load_data(client).await?;
                            }
                            KeyCode::Char('e') | KeyCode::Char('E') => {
                                app.scroll_to_bottom();
                            }
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                // Save logs when on ServerLogs or DebugLogs tab
                                if app.current_tab == tui::Tab::ServerLogs
                                    || app.current_tab == tui::Tab::DebugLogs
                                {
                                    match app.export_logs() {
                                        Ok(filename) => {
                                            app.error_message =
                                                Some(format!("✓ Logs saved to: {}", filename));
                                        }
                                        Err(e) => {
                                            app.error_message =
                                                Some(format!("Failed to save logs: {}", e));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
