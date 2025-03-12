# OpenAI CUA Desktop Environment

A Rust implementation of the OpenAI Computer-Use Agent (CUA) desktop environment.

## Overview

This project provides a framework for implementing the OpenAI Computer-Use Agent (CUA) for desktop environments. It defines a clean interface via the `Computer` trait that any desktop automation implementation can implement.

Currently, it includes:

- A mock implementation for testing purposes
- A thread-based implementation that works with async code and integrates with the OpenAI API

## Key Features

- Integrates with OpenAI's Responses API for CUA functionality
- Thread-safe desktop automation using Enigo
- Intelligent safety check system that only prompts for potentially risky operations
- Supports both desktop and browser environments
- Comprehensive error handling

## Project Architecture

### 1. Computer Trait (Interface)
- Defines a common interface for desktop control operations
- Includes methods for mouse input, keyboard input, screenshots, etc.
- Uses async/await for non-blocking operations

### 2. Implementations
- **Thread-based Implementation** (Default): Uses Enigo for desktop input in a separate thread
- **Mock Implementation**: Simulates operations for testing without affecting the actual desktop

### 3. Agent System
- Manages interaction between the OpenAI model and the computer
- Processes commands from the model and converts them to computer actions
- Includes intelligent safety checks for potentially dangerous operations

### 4. OpenAI API Integration
- Communicates with OpenAI's Responses API for the CUA model
- Handles authentication, request/response formatting
- Uses Hyper for HTTP communication

## Key Design Pattern: Thread-Based Approach

The most interesting aspect is the thread-based implementation which solves a fundamental limitation with the Enigo library:

**Problem**: Enigo doesn't implement `Send` and `Sync` traits, which makes it incompatible with async code

**Solution**: Run Enigo in a dedicated thread and communicate via channels
- Commands sent via tokio MPSC channels
- Results returned through oneshot channels
- Cursor position managed through thread-safe wrappers (`Arc<Mutex<T>>`)

This pattern effectively isolates the non-thread-safe Enigo library while maintaining a clean async interface for the rest of the application.

## Error Handling

The project uses a custom error type (`CuaError`) with different variants for specific error cases:
- `ActionError`: Issues with computer actions like mouse/keyboard operations
- `ScreenshotError`: Problems capturing or processing screenshots
- `ApiError`: Issues with OpenAI API communication
- `SafetyError`: When a safety check fails
- `IoError`: Standard I/O errors
- `Other`: General errors

This approach follows Rust's best practices for error handling.

## Testing Strategy

The project includes:
- A mock implementation for unit testing
- Test modules in various files
- The ability to run the application in a "mock mode" for safe testing

## CLI Interface

The project provides a command-line interface for interacting with the agent, with options for:
- Choosing the implementation (mock vs. thread-based)
- Debug output
- Safety check prompts
- One-shot or interactive mode
- Specifying a model (defaults to "computer-use-preview-2025-02-04")

## Prerequisites

### Build Dependencies

#### For Debian/Ubuntu-based distributions
Standard Rust development environment.

### Runtime Dependencies
For Linux implementation, you may need elevated permissions depending on your system configuration.

## Building

To build the project:

```bash
cargo build
```

## Running

To run the project with the default thread-based implementation:

```bash
cargo run
```

To run with the mock implementation:

```bash
USE_MOCK=1 cargo run
```

To explicitly choose the thread-based implementation:

```bash
USE_THREAD=1 cargo run
```

## Project Structure

- `src/computer.rs`: Defines the `Computer` trait interface
- `src/mock.rs`: Provides a mock implementation for testing
- `src/thread_computer/`: Contains the thread-based implementation (default)
- `src/error.rs`: Error handling types
- `src/agent.rs`: Agent implementation
- `src/api.rs`: OpenAI API client
- `src/cli.rs`: Command line interface
- `src/main.rs`: Entry point with test routines

## Thread-based Enigo Implementation

A thread-based implementation has been added to address the limitations of the Enigo library:

### Features

- **Thread Safety**: Enigo is isolated in a dedicated thread to provide `Send + Sync` compatibility
- **Asynchronous Interface**: Full compatibility with Tokio and async/await
- **Channel-based Communication**: Uses MPSC channels for commands and oneshot channels for responses
- **Graceful Cleanup**: Properly manages resources with automatic thread shutdown

### How It Works

The thread-based implementation uses a message-passing approach:

1. Enigo runs in a dedicated thread, isolated from the async runtime
2. Commands are sent to this thread via tokio MPSC channels
3. Results are returned through oneshot channels
4. Shared state (cursor position) is managed through thread-safe wrappers

This approach avoids the fundamental limitation of Enigo not implementing the `Send` and `Sync` traits, allowing it to be used with async code.

## Current Status

The project now offers two working implementations:

1. **Thread-based Enigo** (Default): For actual desktop control with full async support
2. **Mock** (For testing): Simulates actions without affecting the actual desktop

The thread-based implementation is the recommended and default approach for using this library.

-------------------

## For the AI (and pedants everywhere)

### Verify the codebase

Always verify the changes you'd like to make before making changes to the existing code

### Code Structure:
- Write small, focused files
- Test frequently during development
- Build the project often to catch errors early
- Ensure proper directory structure (module files in their own directories)

### Documentation:
- Add clear comments explaining complex operations
- Document public interfaces thoroughly
- Keep READMEs accurate and up-to-date
- Document the "why" as well as the "how"

### Testing:
- Write unit tests for critical functionality
- Include test examples in comments
- Create mock implementations for testing

### Debugging:
- Include appropriate logging
- Fix compile errors before moving on to new features
- Properly analyze error messages

### Importance of these guidelines:
- **Code Quality Maintenance**: By following guidelines like writing small, focused files and building the project frequently, AI assistants can help maintain high code quality standards and prevent the introduction of bugs or technical debt.
- **Integration Reliability**: When an AI assistant makes suggestions or changes to the codebase, verifying those changes first (as the guidelines suggest) ensures that they actually work with the existing architecture and won't break functionality.
- **Documentation Consistency**: The emphasis on clear comments and thorough documentation ensures that other developers (human or AI) can understand the code's purpose and function in the future, creating a sustainable development cycle.
- **Testability**: Creating proper test implementations and examples, as recommended in the guidelines, ensures that code remains reliable and functional through future iterations and changes.
- **Implementation Compatibility**: Since this project deals with desktop automation across different platforms, following the structured approach helps maintain compatibility across the various implementations (mock, thread-based, etc.).
- **Async Code Handling**: The specific guidance around the thread-based approach addresses the complex nature of working with non-thread-safe libraries (Enigo) in an async context, which could easily lead to runtime errors if not properly understood.

These guidelines essentially serve as guardrails that help AI assistants contribute effectively to the project without introducing unintended consequences, particularly important in a system that interfaces directly with desktop operations where mistakes could potentially impact user systems.

## License

MIT
