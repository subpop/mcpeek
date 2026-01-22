mod logging;
mod mcp;
mod protocol;
mod tui;
mod utcp;

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
use protocol::ProtocolClient;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tui::{render_ui, App};
use utcp::UtcpClient;

#[derive(Parser)]
#[command(name = "mcpeek")]
#[command(about = "Protocol Inspector - Interactive TUI for MCP servers and UTCP manuals", long_about = None)]
struct Cli {
    #[arg(
        long,
        help = "Path to UTCP manual JSON file",
        conflicts_with = "command"
    )]
    utcp: Option<String>,

    #[arg(
        help = "Command to run the MCP server",
        required_unless_present = "utcp"
    )]
    command: Option<String>,

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

    run_tui(cli, log_buffer).await?;

    Ok(())
}

async fn run_tui(cli: Cli, log_buffer: LogBuffer) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create the appropriate client based on CLI arguments
    let client: Box<dyn ProtocolClient> = if let Some(utcp_path) = &cli.utcp {
        // UTCP mode
        Box::new(
            UtcpClient::new(utcp_path)
                .await
                .context("Failed to create UTCP client")?,
        )
    } else if let Some(command) = &cli.command {
        // MCP mode
        Box::new(
            McpClient::new(command, &cli.args)
                .await
                .context("Failed to create MCP client")?,
        )
    } else {
        anyhow::bail!("Either --utcp or command must be provided");
    };

    client
        .initialize()
        .await
        .context("Failed to initialize client")?;

    let mut app = App::new(cli.debug);
    let res = run_tui_loop(&mut terminal, &mut app, client.as_ref(), log_buffer).await;

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
    client: &dyn ProtocolClient,
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
