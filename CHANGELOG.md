# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-01-20

### Added

- **Scope formatting**: Configurable `[scope]` section with `min_width`, `alignment`, and `transform` options for consistent message alignment
- **Message formatting**: Configurable `[message]` section with `transform` option (uppercase, lowercase, capitalize)
- **JSON database output**: New `[json]` output with ULID-based entries for structured log storage
- **Auto-highlighting**: Pattern-based highlighting for URLs, paths, numbers, and quoted strings
- **Configurable level labels**: Custom labels per log level via `[tag.labels]`
- **Raw output mode**: Skip formatting for raw message output
- **App name support**: Explicit app name in log commands and records

### Changed

- Internal logger now uses consistent debug logging across all modules
- Highlight config no longer uses hardcoded defaults

### Fixed

- Re-enabled highlighting for internal logger
- Atomic file writes prevent interleaved log lines

## [0.1.0] - 2025-12-01

### Added

- Initial release
- Terminal output with color support
- File output with configurable paths
- Tag formatting with prefix, suffix, transform, min_width, alignment
- Icon sets (NerdFont, ASCII, none)
- Preset/dictionary system for common log patterns
- Hyprland-style config with `source` directive support
- Log level filtering (trace, debug, info, warn, error)

[Unreleased]: https://github.com/ryugen-io/hyprlog/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/ryugen-io/hyprlog/compare/v0.1.0...v0.5.0
[0.1.0]: https://github.com/ryugen-io/hyprlog/releases/tag/v0.1.0
