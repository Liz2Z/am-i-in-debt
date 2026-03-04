# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release
- Support for 智谱 Coding Plan usage monitoring
- Support for Kimi Coding Plan usage monitoring
- macOS status bar application
- Automatic cookie-based login via Chrome DevTools Protocol
- Real-time usage display with progress bars
- Auto-refresh every 30 seconds
- Manual refresh support
- Token exhaustion notification for currently selected provider
  - Sends macOS notification when token is exhausted
  - Only notifies once per exhaustion event
  - Resets notification when token recovers

### Changed
- **Architecture Refactoring**: Migrated to Provider pattern with self-registration
  - Each provider is now a self-contained module with all its logic
  - Uses `inventory` crate for automatic provider registration
  - Adding a new provider only requires creating a single file
  - Eliminated hardcoded `zhipu`/`kimi` references throughout the codebase
- **UsageInfo Refactoring**: Converted from enum to trait
  - `UsageInfo` is now a trait instead of an enum
  - Each provider implements its own `UsageInfo` trait
  - Added `clone_boxed()` method for trait object cloning
  - Removed redundant `Provider.render_menu_items()` method
  - Menu rendering is now fully handled by `UsageInfo.render_menu_items()`

### Technical Details
- Built with Tauri 2.x + Rust
- Provider pattern architecture with `inventory` crate for self-registration
- Unified sidecar binary for cookie retrieval
- Two core traits: `Provider` and `UsageInfo`
