# PRD: GPUI REST Client

## 项目目标
实现一个基于 GPUI 框架的高性能、现代化的跨平台 REST 客户端，类似于 Postman。

## 核心功能
### V1.0 基础功能
- **请求配置**:
  - 支持 HTTP 动词 (GET, POST, PUT, DELETE)。
  - URL 输入框。
  - 请求头 (Headers) 编辑。
  - 请求体 (Body) 编辑 (支持 JSON)。
- **响应展示**:
  - 响应状态码、时间、大小。
  - 格式化的 JSON 视图。
  - 原始数据视图。
- **历史记录**:
  - 左侧侧边栏展示最近的请求记录。

### V2.0 增强功能
- 集合 (Collections) 管理。
- 环境变量支持。
- 脚本支持 (Pre-request/Tests)。
- 更多 Body 类型 (Multipart, GraphQL)。

## 技术栈
- **语言**: Rust
- **UI 框架**: GPUI (0.2.x)
- **HTTP 客户端**: Reqwest
- **异步运行时**: Tokio (配合 GPUI Task Context)
- **序列化**: Serde
