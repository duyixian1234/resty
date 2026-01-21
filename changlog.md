# Changelog

## [Unreleased] - 2026-01-21
### Added
- 初始化 PRD, implement.md, changlog.md。
- 初始化 agents.md 并配置开发规范。
- 配置 Cargo.toml 依赖 (reqwest, serde, tokio)。
- 实现基础 UI 布局，包含侧边栏历史记录和主视图 URL 栏。
- 实现基础点击事件，支持模拟发送请求。
- 修复了 `reqwest` 调用时因缺少 Tokio 运行时导致的崩溃问题（通过集成全局 Tokio Runtime 手柄）。
- 实现完善了请求发送逻辑，现在支持真正的 HTTP GET 请求并展示响应。
- **多种响应类型渲染支持**：
    - 新增 `response.rs` 模块，实现了 `Response` 和 `ResponseContent` 类型系统。
    - 支持自动检测和渲染 JSON、Text、Image、Binary 等多种响应类型。
    - 基于 HTTP `Content-Type` 头自动识别响应格式。
    - JSON 响应自动格式化美化显示。
    - 图片响应显示：支持渲染 PNG、JPEG、WebP、GIF 等主流格式，并支持自适应缩放显示。
    - 二进制响应显示大小信息和提示。
    - 响应头部状态栏显示：状态码、耗时、数据大小、内容类型。
    - 响应状态码使用不同颜色标识 (2xx 绿色、4xx/5xx 红色、其他黄色)。
- **响应渲染深度改进**：
    - 引入响应选项卡系统 (Body/Headers)，支持在响应体和 HTTP 头之间无缝切换。
    - 实现结构化的 Headers 视图，以键值对形式清晰展示响应头。
    - 为 JSON、Text 和错误信息显示添加了垂直滚动支持 (`overflow_y_scroll`)，妥善处理长内容展示。
    - 增强了组件的类型安全性，优化了 UI 渲染性能。
    - 优化了错误信息的视觉呈现，使用醒目的背景色和边框。

### Fixed
- 修复了 Windows Release 版本启动时会弹出终端窗口的问题（通过添加 `#![windows_subsystem = "windows"]`）。

### Refactored
- **架构升级与性能优化**：
    - 引入 `SharedString` 替换 `String` 作为 UI 层的数据传输格式，大幅减少渲染过程中的内存分配。
    - 持久化 `reqwest::Client` 实例至 `AppState`，启用连接池复用，极大提升了网络请求性能并降低资源消耗。
    - 优化异步运行时集成：将 `main` 函数重构为返回 `anyhow::Result` 模式，移除危险的 `.unwrap()`。
    - 优化 `AppState::send_request` 逻辑，移除冗余的闭包克隆，并采用更高效的异步任务派发方式。
    - 简化 `TextInput` 接口：使其 `text()` 方法返回 `SharedString`，并优化其内部 `paint` 渲染逻辑。
    - 修复了 UI 循环中 10 余处不必要的 `.clone()` 调用，通过引用和共享字符串优化了 CPU 使用率。

### Fixed
- 修复了 `cx.spawn` 闭包中的生命周期问题和类型不匹配问题。
- 修复了 `TextInput` 渲染时 `placeholder` 与 `content` 的类型不一致问题。
- 修正了 `main.rs` 中 `Application::run` 返回值的错误处理逻辑。
- 修复了 URL 输入框无法由键盘编辑的问题。
- 引入了 `TextInput` 组件，采用类似 Zed 项目的 `Focusable` 和 `key_char` 处理逻辑，提高了输入的可扩展性和稳定性。

