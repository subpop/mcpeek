# Usage Examples

## Getting Started

First, build the project:

```bash
cargo build --release
```

## Example 1: Exploring a Weather MCP Server

Assuming you have a weather MCP server at `~/mcp-servers/weather.js`:

```bash
./target/release/mcpeek node ~/mcp-servers/weather.js
```

Once in the TUI:
- Press `Tab` to switch between Tools, Prompts, Resources, Server Info, and Logs tabs
- Use `↑`/`↓` to navigate through items (or scroll logs in the Logs tab)
- Press `Enter` to view details about a selected tool/prompt/resource
- Press `C` to call/execute a selected tool (in the Tools tab)
- Press `E` to jump to the end of logs (useful when in the Logs tab)
- Press `R` to refresh the current view
- Press `Q` to quit

**Note**: The Logs tab automatically captures all stderr output from the MCP server in real-time, so you can monitor server activity and debug issues without the output interfering with the TUI.

## Example 2: Python MCP Server

For a Python-based MCP server:

```bash
./target/release/mcpeek python -m my_mcp_server --config config.json
```

## Example 3: Using uvx with MCP Servers

Many MCP servers are distributed via PyPI and can be run with `uvx`:

### Git MCP Server
```bash
./target/release/mcpeek uvx mcp-server-git --repository /path/to/repo
```

### Filesystem MCP Server
```bash
./target/release/mcpeek uvx mcp-server-filesystem /allowed/path
```

## Example 4: Working with Prompts

To work with prompts, launch the TUI and navigate to the Prompts tab:

```bash
./target/release/mcpeek node server.js
```

Then:
1. Press `Tab` until you reach the "Prompts" tab
2. Use `↑`/`↓` to select a prompt
3. Press `Enter` to view details about the prompt, including its arguments

## Example 5: Resource Management

To explore resources, launch the TUI and navigate to the Resources tab:

```bash
./target/release/mcpeek node server.js
```

Then:
1. Press `Tab` until you reach the "Resources" tab
2. Use `↑`/`↓` to browse available resources
3. Press `Enter` to view details about a specific resource

## Example 6: Debug Mode

Enable debug logging to troubleshoot issues:

```bash
# TUI with debug output
./target/release/mcpeek --debug node server.js 2> debug.log
```

## Example 7: Interactive Tool Calling

Call tools interactively with the form-based parameter input:

```bash
./target/release/mcpeek node server.js
```

Then:
1. Navigate to the "Tools" tab (default tab)
2. Use `↑`/`↓` to select a tool
3. Press `C` to start a tool call
4. An input form appears showing all tool parameters
5. Use `Tab` / `Shift+Tab` to navigate between parameter fields
6. Type values directly into each field
7. Press `Enter` to execute the tool call
8. View the results in the detail view

This makes it easy to call tools with complex nested arguments without manually crafting JSON.

## Example 8: Monitoring Server Logs

The TUI includes a dedicated Logs tab that captures all server stderr output:

```bash
./target/release/mcpeek node server.js
```

Then:
1. Press `Tab` repeatedly until you reach the "Logs" tab
2. Use `↑`/`↓` to scroll through the logs
3. Press `E` to jump to the most recent logs
4. Press `R` to fetch any new logs

This is particularly useful for:
- Debugging server issues
- Monitoring server activity
- Understanding what the server is doing behind the scenes
- Troubleshooting connection or protocol errors

## Tips

1. **Interactive Tool Calling**: Use the `C` key in the Tools tab to call tools with a convenient form-based interface. This is much easier than manually crafting JSON parameters.

2. **Quick Navigation**: Use `Tab` and `Shift+Tab` (or arrow keys `←`/`→`) to quickly switch between different MCP capabilities.

3. **Error Handling**: Enable the `--debug` flag to see detailed error messages in stderr:
   ```bash
   ./target/release/mcpeek --debug node server.js 2> debug.log
   ```

4. **Server Arguments**: Pass arguments directly to the MCP server command:
   ```bash
   ./target/release/mcpeek python server.py --port 8080 --verbose
   ```

5. **Server Logs**: All server stderr output is automatically captured in the Logs tab, preventing it from interfering with the interface while still being accessible for debugging.

6. **Refresh Data**: Press `R` at any time to refresh the current tab's data from the server.
