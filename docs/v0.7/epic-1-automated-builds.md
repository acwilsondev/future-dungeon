# Epic 1: Automated Builds & CICD

This epic focuses on implementing a robust Continuous Integration (CI) pipeline to maintain high code quality and ensure the game remains stable as new features are added.

## User Stories

### Continuous Integration

- **As a developer,** I want every push and pull request to be automatically tested so I can catch regressions early.
- **As a developer,** I want the code to be automatically linted and formatted to maintain the project's engineering standards.
- **As a developer,** I want to see code coverage reports automatically so I can identify areas needing more testing.

### Local Automation

- **As a developer,** I want the `Makefile` to include standard development commands (build, run, clean) to streamline my workflow.
- **As a developer,** I want a single command to run all quality checks (linting, testing, complexity analysis) before I commit code.

### Automated Releases

- **As a developer,** I want a GitHub Action to automatically create a release with pre-built binaries when I push a new tag (e.g., `v0.1.0`).
- **As a player,** I want to be able to download the latest version of the game as a standalone executable for my operating system.

## Technical Tasks

- Create a GitHub Actions workflow file (`.github/workflows/ci.yml`).
  - Implement a `build` job that runs `cargo build`.
  - Implement a `test` job that runs `cargo test`.
  - Implement a `lint` job that runs `cargo clippy` and `cargo fmt --check`.
- Create a GitHub Actions workflow file for releases (`.github/workflows/release.yml`).
  - Trigger on push of tags matching `v*`.
  - Matrix build for Linux, macOS, and Windows.
  - Use `softprops/action-gh-release` to create the release and upload the binaries.
- Enhance the `Makefile`:
  - Add `build`, `run`, `test`, `lint`, and `clean` targets.
  - Refactor the `harden` target to use these new atomic targets.
- Investigate and potentially integrate a code coverage tool (like `tarpaulin` or `grcov`) with the CI pipeline.
- Add a status badge to the README (if one exists).
