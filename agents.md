# Agent Guidelines & Workflow

As an auxiliary development agent for this project, you must adhere to the following operational standards to ensure architectural consistency, code quality, and transparent progress tracking.

## 1. Progress & Documentation Tracking

Maintain a rigorous "Source of Truth" by synchronizing the project state across key documentation files:

* **`PRD.md`**: Refer to this for high-level product requirements and functional specifications.
* **`implement.md`**: Update the implementation status of specific features or components immediately after development.
* **`changelog.md`**: Document every significant change, bug fix, or refactor to maintain a clear version history.

## 2. Rust Code Excellence

Adhere to the Rust ecosystem's high standards for safety and style:

* **Validation**: Every code modification must be followed by `cargo check` to catch compilation errors.
* **Formatting**: Run `cargo fmt` to ensure the codebase remains idiomatic and consistent.
* **Safety**: Prioritize safe Rust; any use of `unsafe` blocks must be explicitly justified and documented.

## 3. UI Development (GPUI Framework)

Focus on the specific paradigms of the GPUI framework:

* **Declarative UI**: Utilize GPUI’s declarative patterns to build reactive components.
* **Layout & Styling**: Leverage the built-in Flexbox layout engine and style system to ensure a performant and aesthetically pleasing interface.
* **Event Handling**: Follow GPUI-specific patterns for view management and input handling.

## 4. Knowledge Retrieval & Research

When encountering complex logic or undocumented GPUI behaviors:

* **Contextual Search**: Use `context7` and integrated web search tools to consult the latest GPUI source code, Rust crates.io documentation, or community best practices.
* **Prototyping**: If a pattern is unverified, implement a minimal reproduction before integrating it into the main codebase.

