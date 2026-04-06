# Engineering Standards for RustLike

This document outlines the best practices, testing strategies, and documentation standards for the development of "RustLike".

## 1. Rust Best Practices

### Code Style & Quality
- **Formatting:** Use `cargo fmt` for consistent code formatting.
- **Linting:** Regularly run `cargo clippy` and address all warnings. Use `#[deny(clippy::all)]` in the main entry point to enforce quality.
- **Safety:** Avoid `unsafe` code unless absolutely necessary for performance or FFI.
- **Explicit Types:** While Rust has excellent type inference, use explicit type annotations for complex return types and function signatures to improve readability.
- **Error Handling:**
  - Use `thiserror` for library-level errors (where callers need to match on variants).
  - Use `anyhow` for application-level errors (where you just need to propagate the error).
  - **NEVER** use `unwrap()` or `expect()` in production code. Always handle errors gracefully with `match` or `if let`.

### Architecture (ECS)
- **Separation of Concerns:** Keep game logic (Systems) separate from data (Components) and rendering.
- **Serialization:** Ensure all game state components derive `Serialize` and `Deserialize` (using `serde`) for persistence.

## 2. Testing Strategy

### Unit Testing
- Small, focused tests should be placed in the same file as the code they test, inside a `#[cfg(test)] mod tests` block.
- Aim for high coverage of game logic (combat calculations, movement validation).
- We should target 80% code coverage across the repo.

### Integration Testing
- More complex scenarios (dungeon generation, multi-turn interactions) should be placed in the `tests/` directory.

### Validation
- All pull requests and changes must pass `cargo test`, `cargo clippy`, and `cargo fmt --check`.

## 3. Documentation Standards

### Code Documentation
- Use `///` for documentation comments on functions, structs, and enums.
- Use `//!` for module-level documentation.
- **Examples in Docs:** Whenever possible, include doc tests (examples that `cargo test` runs) to demonstrate how to use a function.

### Project Documentation
- All architectural decisions and major features must be documented in the `docs/` folder.
- Maintain a clear changelog and update epics as progress is made.

## 4. Implementation Guidelines (Epic 1)

### Rendering
- Use `ratatui` for the terminal UI framework.
- Use `crossterm` for cross-platform terminal backend and event handling.
- **State-Based Rendering:** The renderer should take the current game state (or a view of it) and draw it without modifying the state.

### Input
- Handle input asynchronously or via a non-blocking loop to ensure the game remains responsive.
- Map keys to high-level game commands (e.g., `Direction::Up`) rather than handling raw key codes in the game loop.
