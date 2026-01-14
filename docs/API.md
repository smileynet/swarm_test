# API Reference

Complete API documentation for the swarm_test library.

## Table of Contents

- [Types](#types)
- [Session Management](#session-management)
- [Message Sending](#message-sending)
- [Log Reading](#log-reading)
- [Message Queue](#message-queue)
- [Error Handling](#error-handling)

---

## Types

### SessionId

Wrapper type for tmux session identifiers.

```rust
pub struct SessionId(pub String);
```

**Implements**: Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash

**Example**:
```rust
let id = SessionId("$0".to_string());
let id2 = SessionId::from("$1");
```

---

### WindowId

Wrapper type for tmux window identifiers.

```rust
pub struct WindowId(pub String);
```

**Implements**: Debug, Clone, Serialize, Deserialize, PartialEq, Eq

**Example**:
```rust
let id = WindowId("@0".to_string());
```

---

### PaneId

Wrapper type for tmux pane identifiers.

```rust
pub struct PaneId(pub String);
```

**Implements**: Debug, Clone, Serialize, Deserialize, PartialEq, Eq

**Example**:
```rust
let id = PaneId("%0".to_string());
```

---

### Session

Represents a tmux session with all its metadata.

```rust
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub windows: Vec<Window>,
    pub attached: bool,
}
```

**Fields**:
- `id: SessionId` - Unique session identifier
- `name: String` - Human-readable session name
- `windows: Vec<Window>` - List of windows in this session
- `attached: bool` - Whether a client is attached

---

### Window

Represents a tmux window.

```rust
pub struct Window {
    pub id: WindowId,
    pub session_id: SessionId,
    pub name: String,
    pub panes: Vec<Pane>,
    pub active: bool,
}
```

**Fields**:
- `id: WindowId` - Unique window identifier
- `session_id: SessionId` - Parent session
- `name: String` - Window name
- `panes: Vec<Pane>` - List of panes in this window
- `active: bool` - Whether this is the active window

---

### Pane

Represents a tmux pane.

```rust
pub struct Pane {
    pub id: PaneId,
    pub window_id: WindowId,
    pub session_id: SessionId,
    pub current_path: Option<String>,
    pub pid: Option<u32>,
    pub active: bool,
}
```

**Fields**:
- `id: PaneId` - Unique pane identifier
- `window_id: WindowId` - Parent window
- `session_id: SessionId` - Parent session
- `current_path: Option<String>` - Current working directory
- `pid: Option<u32>` - Process ID of pane command
- `active: bool` - Whether this is the active pane

---

### Message

Represents a message sent to a pane.

```rust
pub struct Message {
    pub id: String,
    pub pane_id: PaneId,
    pub content: String,
    pub timestamp: u64,
}
```

**Fields**:
- `id: String` - Unique message identifier (UUID)
- `pane_id: PaneId` - Target pane
- `content: String` - Message content
- `timestamp: u64` - Unix timestamp

---

### Command

Represents a tmux command to execute.

```rust
pub struct Command {
    pub command: String,
    pub target: CommandTarget,
    pub args: Vec<String>,
}
```

**Fields**:
- `command: String` - Tmux command (e.g., "new-session")
- `target: CommandTarget` - Command target scope
- `args: Vec<String>` - Command arguments

---

### CommandTarget

Specifies the scope of a tmux command.

```rust
pub enum CommandTarget {
    Server,
    Session(SessionId),
    Window(WindowId),
    Pane(PaneId),
}
```

**Variants**:
- `Server` - Server-wide command
- `Session(SessionId)` - Session-specific command
- `Window(WindowId)` - Window-specific command
- `Pane(PaneId)` - Pane-specific command

---

### Response

Represents the result of a tmux command.

```rust
pub struct Response {
    pub success: bool,
    pub data: ResponseData,
    pub error: Option<String>,
}
```

**Fields**:
- `success: bool` - Whether command succeeded
- `data: ResponseData` - Response payload
- `error: Option<String>` - Error message if failed

---

### ResponseData

Enum containing possible response payloads.

```rust
pub enum ResponseData {
    Empty,
    Session(Session),
    Window(Window),
    Pane(Pane),
    Sessions(Vec<Session>),
    Windows(Vec<Window>),
    Panes(Vec<Pane>),
    Output(String),
}
```

**Variants**:
- `Empty` - No data
- `Session(Session)` - Single session
- `Window(Window)` - Single window
- `Pane(Pane)` - Single pane
- `Sessions(Vec<Session>)` - List of sessions
- `Windows(Vec<Window>)` - List of windows
- `Panes(Vec<Pane>)` - List of panes
- `Output(String)` - Raw string output

---

### Result

Type alias for result type used throughout library.

```rust
pub type Result<T> = std::result::Result<T, error::TmuxError>;
```

---

## Session Management

### `new_session`

Creates a new tmux session with the given name.

```rust
pub fn new_session(name: impl Into<String>) -> Result<Session>
```

**Parameters**:
- `name: impl Into<String>` - Session name

**Returns**: `Result<Session>`

**Errors**:
- `TmuxError::Command` - Tmux command failed
- `TmuxError::NotFound` - Session not found after creation

**Example**:
```rust
let session = tmux::session::new_session("my_session")?;
```

---

### `kill_session`

Terminates a tmux session.

```rust
pub fn kill_session(id: &SessionId) -> Result<()>
```

**Parameters**:
- `id: &SessionId` - Session to terminate

**Returns**: `Result<()>`

**Errors**:
- `TmuxError::Command` - Tmux command failed

**Example**:
```rust
tmux::session::kill_session(&session.id)?;
```

---

### `list_sessions`

Lists all active tmux sessions.

```rust
pub fn list_sessions() -> Result<Vec<Session>>
```

**Returns**: `Result<Vec<Session>>`

**Errors**:
- `TmuxError::Command` - Tmux command failed

**Example**:
```rust
let sessions = tmux::session::list_sessions()?;
for s in sessions {
    println!("{}: {}", s.id.0, s.name);
}
```

---

### `get_session`

Retrieves a specific session by ID.

```rust
pub fn get_session(id: &SessionId) -> Result<Session>
```

**Parameters**:
- `id: &SessionId` - Session ID to retrieve

**Returns**: `Result<Session>`

**Errors**:
- `TmuxError::NotFound` - Session not found

**Example**:
```rust
let session = tmux::session::get_session(&SessionId("$0".into()))?;
```

---

### `attach_session`

Attaches the current client to a session.

```rust
pub fn attach_session(id: &SessionId) -> Result<()>
```

**Parameters**:
- `id: &SessionId` - Session to attach

**Returns**: `Result<()>`

**Errors**:
- `TmuxError::Command` - Tmux command failed

**Example**:
```rust
tmux::session::attach_session(&session.id)?;
```

---

### `detach_session`

Detaches all clients from a session.

```rust
pub fn detach_session(id: &SessionId) -> Result<()>
```

**Parameters**:
- `id: &SessionId` - Session to detach

**Returns**: `Result<()>`

**Errors**:
- `TmuxError::Command` - Tmux command failed

**Example**:
```rust
tmux::session::detach_session(&session.id)?;
```

---

### `rename_session`

Renames a session.

```rust
pub fn rename_session(id: &SessionId, new_name: impl Into<String>) -> Result<()>
```

**Parameters**:
- `id: &SessionId` - Session to rename
- `new_name: impl Into<String>` - New session name

**Returns**: `Result<()>`

**Errors**:
- `TmuxError::Command` - Tmux command failed

**Example**:
```rust
tmux::session::rename_session(&session.id, "new_name")?;
```

---

## Message Sending

### MessageSender

Handles sending messages to tmux panes.

```rust
pub struct MessageSender {
    // fields are private
}
```

#### `new`

Creates a new MessageSender with the given base path.

```rust
pub fn new<P: AsRef<Path>>(base_path: P) -> Self
```

**Parameters**:
- `base_path: P` - Base path for prompt directory

**Returns**: `MessageSender`

**Example**:
```rust
let sender = MessageSender::new("/path/to/project");
```

---

#### `base_path`

Returns the base path for this sender.

```rust
pub fn base_path(&self) -> &Path
```

**Returns**: `&Path`

---

#### `send_prompt`

Sends a simple prompt to a pane.

```rust
pub fn send_prompt(&self, pane_id: &PaneId, prompt: &str) -> Result<()>
```

**Parameters**:
- `pane_id: &PaneId` - Target pane
- `prompt: &str` - Prompt content

**Returns**: `Result<()>`

**Errors**:
- `TmuxError::Io` - File I/O error

**Example**:
```rust
sender.send_prompt(&pane_id, "Hello, world!")?;
```

---

#### `send_prompt_with_metadata`

Sends a prompt with metadata tracking.

```rust
pub fn send_prompt_with_metadata(
    &self,
    pane_id: &PaneId,
    prompt: &str,
    metadata: &PromptMetadata,
) -> Result<()>
```

**Parameters**:
- `pane_id: &PaneId` - Target pane
- `prompt: &str` - Prompt content
- `metadata: &PromptMetadata` - Metadata to include

**Returns**: `Result<()>`

**Errors**:
- `TmuxError::Io` - File I/O error

**Example**:
```rust
let metadata = PromptMetadata::new(session_id, "AgentName".to_string());
sender.send_prompt_with_metadata(&pane_id, "Task: Fix bug", &metadata)?;
```

---

#### `read_prompt`

Reads the current prompt for a pane.

```rust
pub fn read_prompt(&self, pane_id: &PaneId) -> Result<String>
```

**Parameters**:
- `pane_id: &PaneId` - Target pane

**Returns**: `Result<String>`

**Errors**:
- `TmuxError::NotFound` - Prompt file not found
- `TmuxError::Io` - File I/O error

**Example**:
```rust
let prompt = sender.read_prompt(&pane_id)?;
```

---

#### `clear_prompt`

Removes the prompt file for a pane.

```rust
pub fn clear_prompt(&self, pane_id: &PaneId) -> Result<()>
```

**Parameters**:
- `pane_id: &PaneId` - Target pane

**Returns**: `Result<()>`

**Errors**:
- `TmuxError::Io` - File I/O error

**Example**:
```rust
sender.clear_prompt(&pane_id)?;
```

---

### PromptMetadata

Metadata associated with a prompt.

```rust
pub struct PromptMetadata {
    pub session_id: SessionId,
    pub timestamp: u64,
    pub agent: String,
}
```

#### `new`

Creates new PromptMetadata with current timestamp.

```rust
pub fn new(session_id: SessionId, agent: String) -> Self
```

**Parameters**:
- `session_id: SessionId` - Session ID
- `agent: String` - Agent name

**Returns**: `PromptMetadata`

**Example**:
```rust
let metadata = PromptMetadata::new(session_id, "Worker1".to_string());
```

---

### FileLock

Provides file-based locking for concurrent operations.

```rust
pub struct FileLock {
    // fields are private
}
```

#### `acquire`

Acquires an exclusive lock on a file.

```rust
pub fn acquire<P: AsRef<Path>>(path: P) -> Result<Self>
```

**Parameters**:
- `path: P` - File path to lock

**Returns**: `Result<FileLock>`

**Errors**:
- `TmuxError::Process` - Lock already held
- `TmuxError::Io` - File I/O error

**Example**:
```rust
let _lock = FileLock::acquire("/path/to/lockfile")?;
// Lock released automatically when _lock goes out of scope
```

---

#### `try_with`

Executes a closure while holding a lock.

```rust
pub fn try_with<F, R, P: AsRef<Path>>(path: P, f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
```

**Parameters**:
- `path: P` - File path to lock
- `f: F` - Closure to execute

**Returns**: `Result<R>`

**Example**:
```rust
FileLock::try_with("/path/to/lockfile", || {
    do_critical_work();
    Ok(())
})?;
```

---

## Log Reading

### LogReader

Handles reading and monitoring of tmux logs.

```rust
pub struct LogReader {
    // fields are private
}
```

#### `new`

Creates a new LogReader using default log directory (`/tmp/tmux_logs`).

```rust
pub fn new() -> Self
```

**Returns**: `LogReader`

**Example**:
```rust
let reader = LogReader::new();
```

---

#### `with_dir`

Creates a new LogReader with custom log directory.

```rust
pub fn with_dir(log_dir: PathBuf) -> Self
```

**Parameters**:
- `log_dir: PathBuf` - Custom log directory path

**Returns**: `LogReader`

**Example**:
```rust
let reader = LogReader::with_dir("/custom/logs".into());
```

---

#### `read_log`

Reads entire log content for a session.

```rust
pub fn read_log(&self, session_id: &SessionId) -> Result<String>
```

**Parameters**:
- `session_id: &SessionId` - Session ID

**Returns**: `Result<String>`

**Errors**:
- `TmuxError::Command` - Failed to read log

**Example**:
```rust
let content = reader.read_log(&session_id)?;
```

---

#### `read_log_lines`

Reads log content as a vector of lines.

```rust
pub fn read_log_lines(&self, session_id: &SessionId) -> Result<Vec<String>>
```

**Parameters**:
- `session_id: &SessionId` - Session ID

**Returns**: `Result<Vec<String>>`

**Errors**:
- `TmuxError::Command` - Failed to read log

**Example**:
```rust
let lines = reader.read_log_lines(&session_id)?;
```

---

#### `read_log_from`

Reads log lines starting from a given offset.

```rust
pub fn read_log_from(&self, session_id: &SessionId, offset: usize) -> Result<Vec<String>>
```

**Parameters**:
- `session_id: &SessionId` - Session ID
- `offset: usize` - Line number to start from

**Returns**: `Result<Vec<String>>`

**Errors**:
- `TmuxError::Command` - Failed to read log

**Example**:
```rust
let lines = reader.read_log_from(&session_id, 100)?;
```

---

#### `read_pane_output`

Reads output for a specific pane.

```rust
pub fn read_pane_output(&self, pane_id: &PaneId) -> Result<String>
```

**Parameters**:
- `pane_id: &PaneId` - Pane ID

**Returns**: `Result<String>`

**Errors**:
- `TmuxError::Command` - Failed to read pane log

**Example**:
```rust
let output = reader.read_pane_output(&pane_id)?;
```

---

#### `watch_log`

Monitors log in real-time (blocking operation).

```rust
pub fn watch_log<F>(&self, session_id: &SessionId, callback: F) -> Result<(), TmuxError>
where
    F: Fn(&str) + Send + Sync,
```

**Parameters**:
- `session_id: &SessionId` - Session ID
- `callback: F` - Function called for each new line

**Returns**: `Result<(), TmuxError>`

**Errors**:
- `TmuxError::Command` - Failed to read log

**Example**:
```rust
reader.watch_log(&session_id, |line| {
    println!("{}", line);
})?;
```

---

#### `get_log_size`

Gets the size of a log file in bytes.

```rust
pub fn get_log_size(&self, session_id: &SessionId) -> Result<u64>
```

**Parameters**:
- `session_id: &SessionId` - Session ID

**Returns**: `Result<u64>`

**Errors**:
- `TmuxError::Command` - Failed to get metadata

**Example**:
```rust
let size = reader.get_log_size(&session_id)?;
```

---

#### `get_log_timestamp`

Gets the modification timestamp of a log file.

```rust
pub fn get_log_timestamp(&self, session_id: &SessionId) -> Result<u64>
```

**Parameters**:
- `session_id: &SessionId` - Session ID

**Returns**: `Result<u64>` - Unix timestamp

**Errors**:
- `TmuxError::Command` - Failed to get metadata

**Example**:
```rust
let timestamp = reader.get_log_timestamp(&session_id)?;
```

---

#### `tail_log`

Returns the last N lines of a log.

```rust
pub fn tail_log(&self, session_id: &SessionId, n: usize) -> Result<Vec<String>>
```

**Parameters**:
- `session_id: &SessionId` - Session ID
- `n: usize` - Number of lines to return

**Returns**: `Result<Vec<String>>`

**Errors**:
- `TmuxError::Command` - Failed to read log

**Example**:
```rust
let last_lines = reader.tail_log(&session_id, 20)?;
```

---

#### `search_log`

Searches log for lines containing a pattern.

```rust
pub fn search_log(&self, session_id: &SessionId, pattern: &str) -> Result<Vec<String>>
```

**Parameters**:
- `session_id: &SessionId` - Session ID
- `pattern: &str` - Search pattern

**Returns**: `Result<Vec<String>>`

**Errors**:
- `TmuxError::Command` - Failed to read log

**Example**:
```rust
let errors = reader.search_log(&session_id, "error")?;
```

---

#### `clear_log`

Clears the content of a log file.

```rust
pub fn clear_log(&self, session_id: &SessionId) -> Result<(), TmuxError>
```

**Parameters**:
- `session_id: &SessionId` - Session ID

**Returns**: `Result<(), TmuxError>`

**Errors**:
- `TmuxError::Command` - Failed to clear log

**Example**:
```rust
reader.clear_log(&session_id)?;
```

---

#### `delete_log`

Deletes a log file.

```rust
pub fn delete_log(&self, session_id: &SessionId) -> Result<(), TmuxError>
```

**Parameters**:
- `session_id: &SessionId` - Session ID

**Returns**: `Result<(), TmuxError>`

**Errors**:
- `TmuxError::Command` - Failed to delete log

**Example**:
```rust
reader.delete_log(&session_id)?;
```

---

#### `list_session_logs`

Lists all sessions that have log files.

```rust
pub fn list_session_logs(&self) -> Result<Vec<SessionId>>
```

**Returns**: `Result<Vec<SessionId>>`

**Errors**:
- `TmuxError::Command` - Failed to read directory

**Example**:
```rust
let logged_sessions = reader.list_session_logs()?;
```

---

### Default Implementation

LogReader implements Default trait.

```rust
impl Default for LogReader {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Message Queue

### MessageQueue

Manages a queue of messages for delivery.

```rust
pub struct MessageQueue {
    // fields are private
}
```

#### `new`

Creates a new empty MessageQueue.

```rust
pub fn new() -> Self
```

**Returns**: `MessageQueue`

---

#### `push`

Adds a message to the queue.

```rust
pub fn push(&mut self, message: QueuedMessage)
```

**Parameters**:
- `message: QueuedMessage` - Message to queue

---

#### `pop`

Removes and returns the oldest message.

```rust
pub fn pop(&mut self) -> Option<QueuedMessage>
```

**Returns**: `Option<QueuedMessage>`

---

#### `peek`

Returns the oldest message without removing it.

```rust
pub fn peek(&self) -> Option<&QueuedMessage>
```

**Returns**: `Option<&QueuedMessage>`

---

#### `stats`

Returns queue statistics.

```rust
pub fn stats(&self) -> QueueStats
```

**Returns**: `QueueStats`

---

### QueuedMessage

Represents a message in the queue.

```rust
pub struct QueuedMessage {
    pub id: String,
    pub pane_id: PaneId,
    pub content: String,
    pub queued_at: u64,
}
```

**Fields**:
- `id: String` - Unique message identifier
- `pane_id: PaneId` - Target pane
- `content: String` - Message content
- `queued_at: u64` - Unix timestamp when queued

---

### QueueStats

Statistics about the message queue.

```rust
pub struct QueueStats {
    pub total_messages: usize,
    pub oldest_message_age: Option<u64>,
}
```

**Fields**:
- `total_messages: usize` - Number of messages in queue
- `oldest_message_age: Option<u64>` - Age of oldest message in seconds

---

## Error Handling

### TmuxError

Comprehensive error type for all library operations.

```rust
pub enum TmuxError {
    Io(String),
    Command(String),
    Parse(String),
    NotFound(String),
    Process(std::io::ErrorKind, String),
}
```

**Variants**:

#### `Io(String)`

I/O operation failed.

**Parameters**:
- `String` - Error description

---

#### `Command(String)`

Tmux command execution failed.

**Parameters**:
- `String` - Error description

---

#### `Parse(String)`

Failed to parse response.

**Parameters**:
- `String` - Error description

---

#### `NotFound(String)`

Resource not found.

**Parameters**:
- `String` - Description of what was not found

---

#### `Process(std::io::ErrorKind, String)`

Process-related error.

**Parameters**:
- `std::io::ErrorKind` - Type of error
- `String` - Error description

---

### Error Display

All TmuxError variants implement `Display` trait for user-friendly error messages.

### Error Source

TmuxError implements `std::error::Error` for proper error handling chains.

---

## Result Type

Throughout the library, the `Result` type is used:

```rust
pub type Result<T> = std::result::Result<T, TmuxError>;
```

This ensures consistent error handling across all operations.

---

## CLI Commands

For CLI usage, see the main [README.md](../README.md).
