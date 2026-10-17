# ADR 0001: Tauri 2 and CodeMirror 6

- Status: accepted
- Date: 2026-10-17

## Context

The MVP needs a credible structured-code editor with viewport rendering, multiple
selections, line numbers, folding, matching, search, commands, incremental syntax
parsing, and extensions. Native Rust GUI toolkits do not currently provide this
complete XML-editor surface as a maintained component.

## Decision

Use Tauri 2 as the desktop shell and CodeMirror 6 with Lezer XML in a small
TypeScript frontend. Keep document ownership, persistence, validation, resource
policy, projects, and standards engines in reusable Rust crates.

## Consequences

The project is Rust-first but not Rust-only. OS webviews can differ, frontend
dependencies need their own supply-chain controls, and UTF-16/UTF-8 offset
conversion is explicit. In return, Phase 1 can focus on XML correctness rather
than building a text editor widget.
