# Swarm Test - Tmux Control Rig

A Rust library and CLI tool for programmatically controlling tmux sessions, designed for integration with AI coding agents and OpenCode workflow automation.

## Features

- **Session Management**: Create, list, rename, attach, detach, and destroy tmux sessions
- **Message Passing**: Send prompts to specific panes with metadata tracking
- **Log Reading**: Read, tail, search, and monitor session output in real-time
- **CLI Interface**: Full command-line tool for interactive use
- **Type-Safe API**: Comprehensive Rust library with proper error handling
- **Integration Ready**: Designed for AI agent workflows and automation

## Installation

### Prerequisites

- Rust 2024 edition or later
- tmux installed on your system
- Linux/macOS (file locking features)

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd swarm_test

# Build the project
cargo build --release

# The binary will be available at target/release/swarm_test
```

### Run Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run integration tests specifically
cargo test --test integration_test
```

## CLI Usage

The `swarm_test` CLI provides commands for session management, message passing, and output monitoring.

### Session Management

```bash
# Create a new session
swarm_test session start my_session

# List all sessions
swarm_test session list

# Attach to a session
swarm_test session attach my_session

# Detach from a session
swarm_test session detach my_session

# Stop a session
swarm_test session stop my_session
```

### Message Passing

```bash
# Send a message to a specific pane
swarm_test message send %1 "Hello, OpenCode!"

# Messages are written to .opencode/prompts/<pane_id>.prompt.input
```

### Output Monitoring

```bash
# Read full session output
swarm_test output read $0

# Tail last N lines (default 20)
swarm_test output tail $0 50

# Watch output in real-time (Ctrl+C to stop)
swarm_test output watch $0
```

### System Status

```bash
# Show overall system status
swarm_test status
```

## Library Usage

### Session Management

```rust
use swarm_test::tmux::session;
use swarm_test::types::SessionId;

// Create a new session
let session = session::new_session("my_session")?;
println!("Created session: {}", session.id.0);

// List all sessions
let sessions = session::list_sessions()?;
for s in sessions {
    println!("{}: {} ({} windows)", s.id.0, s.name, s.windows.len());
}

// Get a specific session
let session_id = SessionId("$0".to_string());
let session = session::get_session(&session_id)?;

// Rename a session
session::rename_session(&session_id, "new_name")?;

// Kill a session
session::kill_session(&session_id)?;

// Attach/Detach
session::attach_session(&session_id)?;
session::detach_session(&session_id)?;
```

### Message Sending

```rust
use swarm_test::messaging::send::{MessageSender, PromptMetadata};
use swarm_test::types::{SessionId, PaneId};

// Create sender with base path
let sender = MessageSender::new("/path/to/project");

let pane_id = PaneId("%0".to_string());

// Simple message
sender.send_prompt(&pane_id, "Hello, world!")?;

// Message with metadata
let session_id = SessionId("$0".to_string());
let metadata = PromptMetadata::new(session_id, "AgentName".to_string());
sender.send_prompt_with_metadata(&pane_id, "Task: Fix bug #123", &metadata)?;

// Read prompt back
let content = sender.read_prompt(&pane_id)?;

// Clear prompt
sender.clear_prompt(&pane_id)?;
```

### Log Reading

```rust
use swarm_test::messaging::read::LogReader;
use swarm_test::types::SessionId;

// Create log reader (uses /tmp/tmux_logs by default)
let reader = LogReader::new();

// Or use custom directory
let custom_reader = LogReader::with_dir("/path/to/logs".into());

let session_id = SessionId("$0".to_string());

// Read entire log
let content = reader.read_log(&session_id)?;

// Read as lines
let lines = reader.read_log_lines(&session_id)?;

// Tail last N lines
let recent = reader.tail_log(&session_id, 20)?;

// Search for patterns
let matches = reader.search_log(&session_id, "error")?;

// Get log metadata
let size = reader.get_log_size(&session_id)?;
let timestamp = reader.get_log_timestamp(&session_id)?;

// Clear or delete log
reader.clear_log(&session_id)?;
reader.delete_log(&session_id)?;

// Watch log in real-time (blocks)
reader.watch_log(&session_id, |line| {
    println!("{}", line);
})?;

// List all available session logs
let logged_sessions = reader.list_session_logs()?;
```

### File Locking

```rust
use swarm_test::messaging::send::FileLock;

// Acquire exclusive lock
let _lock = FileLock::acquire("/path/to/lockfile")?;
// Lock released automatically when _lock goes out of scope

// Execute code with lock
FileLock::try_with("/path/to/lockfile", || {
    // Critical section
    do_something();
    Ok(())
})?;
```

## Architecture

### Module Structure

```
src/
├── types.rs          # Core type definitions (Session, Window, Pane, etc.)
├── error.rs          # Error types and Result handling
├── tmux/
│   ├── mod.rs        # Tmux module exports
│   ├── cli.rs        # Tmux command execution
│   ├── session.rs    # Session management functions
│   ├── window.rs     # Window management
│   └── pane.rs       # Pane management
├── messaging/
│   ├── mod.rs        # Messaging module exports
│   ├── send.rs       # Message sending (MessageSender, FileLock)
│   ├── read.rs       # Log reading (LogReader)
│   ├── queue.rs      # Message queue (MessageQueue)
│   ├── parser.rs     # Output parsing
│   └── filter.rs     # Output filtering
├── opencode/
│   ├── mod.rs        # OpenCode protocol integration
│   ├── session.rs    # OpenCode session management
│   └── protocol.rs   # Protocol definitions
└── cli/
    ├── mod.rs        # CLI module
    ├── main.rs       # CLI entry point
    └── commands.rs   # CLI command handlers
```

### Data Flow

```
CLI/Code
    ↓
Session Management (tmux/session.rs)
    ↓
Tmux CLI Wrapper (tmux/cli.rs)
    ↓
Tmux Server
```

Message Flow:
```
Agent → MessageSender → .opencode/prompts/<pane_id>.prompt.input
                                          ↓
                                    Agent reads file
                                          ↓
                                    Executes in tmux pane
                                          ↓
                                    Output captured
                                          ↓
                                    LogReader reads /tmp/tmux_logs/
```

### Key Components

1. **Type System**: Strong typing for SessionId, WindowId, PaneId with serde support
2. **Error Handling**: Comprehensive TmuxError enum with proper error propagation
3. **Command Execution**: Unified command building and execution through tmux CLI
4. **Message Protocol**: File-based message passing with metadata and locking
5. **Log Management**: Flexible log reading with search, tail, and watch capabilities

## Design Decisions

### File-Based Message Passing

Messages are passed through files in `.opencode/prompts/` rather than direct tmux commands because:
- Allows decoupling of message creation and execution
- Enables metadata tracking (session, agent, timestamp)
- Supports offline queueing and processing
- Facilitates testing and debugging

### Separate Log Directory

Logs are stored in `/tmp/tmux_logs/` rather than tmux's default location because:
- Centralized location for all session logs
- Easier to manage permissions
- Supports programmatic access without tmux knowledge
- Simplifies cleanup and rotation

### CLI Wrapper

All tmux operations go through the `tmux` CLI rather than libtmux because:
- Zero external dependencies
- Works with any tmux version
- Simpler debugging (can see commands)
- No unsafe code for library bindings

## Testing

The project includes comprehensive tests:

- **Unit Tests**: Individual module tests in source files
- **Integration Tests**: Full workflow tests in `tests/integration_test.rs`

Run integration tests with:
```bash
cargo test --test integration_test
```

Integration tests cover:
- Session lifecycle (create, rename, kill)
- Message sending and reading
- Log reading operations
- Error handling
- Multiple session management
- Custom log directories

## Development

### Adding New Features

1. Add types to `types.rs` if needed
2. Implement core logic in appropriate module (tmux/, messaging/)
3. Add CLI commands in `cli/commands.rs`
4. Write tests in module files and integration tests
5. Update this README

### Code Style

- Follow Rust naming conventions
- Use `?` operator for error propagation
- Include doc comments for public APIs
- Keep functions focused and small

## License

This project is part of the OpenCode ecosystem.

## Contributing

This project is designed for OpenCode agent workflows. Contributions should maintain compatibility with:
- OpenCode protocol standards
- File-based message passing
- Tmux session management patterns
