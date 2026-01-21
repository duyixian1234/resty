# Changelog

## [Unreleased] - 2026-01-20
### Added
- 初始化 PRD, implement.md, changlog.md。
- 初始化 agents.md 并配置开发规范。
- 配置 Cargo.toml 依赖 (reqwest, serde, tokio)。
- 实现基础 UI 布局，包含侧边栏历史记录和主视图 URL 栏。
- 实现基础点击事件，支持模拟发送请求。
- 修复了 `reqwest` 调用时因缺少 Tokio 运行时导致的崩溃问题（通过集成全局 Tokio Runtime 手柄）。
- 实现完善了请求发送逻辑，现在支持真正的 HTTP GET 请求并展示响应。

### Fixed
- 修复了 URL 输入框无法由键盘编辑的问题。
- 引入了 `TextInput` 组件，采用类似 Zed 项目的 `Focusable` 和 `key_char` 处理逻辑，提高了输入的可扩展性和稳定性。
- 实现了 `TextInput` 的高性能自定义渲染：
    - 支持显示/隐藏光标与选中区域。
    - 完整支持操作系统 IME 输入法集成（通过 `EntityInputHandler`）。
    - 支持基于 Unicode Grapheme 的光标移动和文本编辑。
    - 修复了 `TextRun` 和 `shape_line` 的 API 兼容性问题。
    - 修复了按键输入字符重复（Double Input）的问题：移除 `on_key_down` 中冗余的 `key_char` 处理，统一由 `EntityInputHandler` 管理。
    - 将数据模型和 HTTP 请求逻辑抽离至 `src/app_state.rs`。
    - 将 UI 组件和渲染逻辑移至 `src/workspace.rs`。
    - 引入 `src/theme.rs` 管理颜色主题。
    - 在 `src/main.rs` 中使用 GPUI 的 `Entity` 和 `View` 模式进行重构，遵循 Zed project 架构规范。
