# TodoMVC — Leptos + Axum + SQLite

A full-stack [TodoMVC](https://todomvc.com/) implementation built with Rust.

## Tech Stack

- **Frontend:** [Leptos](https://leptos.dev/) 0.7 (CSR mode) — reactive UI compiled to WebAssembly
- **Backend:** [Axum](https://github.com/tokio-rs/axum) 0.8 — async HTTP server
- **Database:** SQLite via [sqlx](https://github.com/launchbadge/sqlx) with compile-time checked queries
- **Styling:** [Tailwind CSS](https://tailwindcss.com/) v4 + canonical TodoMVC styles
- **Runtime:** [Tokio](https://tokio.rs/)

## Features

- Add, edit, complete, and delete todos
- Double-click to edit inline (Enter saves, Escape cancels, blank deletes)
- Toggle all todos complete/incomplete
- Filter by All / Active / Completed via URL hash routing
- Clear all completed todos
- Persistent SQLite storage
- Responsive TodoMVC-compliant styling

## Prerequisites

- [Rust](https://rustup.rs/) (nightly toolchain — configured via `rust-toolchain.toml`)
- [Node.js](https://nodejs.org/) (for Tailwind CSS)

## Setup

```bash
# Install dependencies
npm install

# Build Tailwind CSS
npm run tailwind:build

# Run database migrations (automatic on server start)
# Default database: sqlite://todos.db (override with DATABASE_URL env var)
```

## Development

```bash
# Build and run the server
cargo run --bin server

# The app will be available at http://localhost:8080

# Watch Tailwind CSS for changes (in a separate terminal)
npm run tailwind:watch
```

## Testing

```bash
# Run all tests (API integration + end-to-end)
cargo test

# Run only API integration tests
cargo test --test api_tests

# Run only end-to-end flow tests
cargo test --test e2e_tests
```

## API Endpoints

| Method   | Path                     | Description              |
|----------|--------------------------|--------------------------|
| `GET`    | `/api/todos`             | List all todos           |
| `POST`   | `/api/todos`             | Create a new todo        |
| `PATCH`  | `/api/todos/:id`         | Update a todo            |
| `DELETE` | `/api/todos/:id`         | Delete a todo            |
| `PATCH`  | `/api/todos/toggle-all`  | Toggle all completed     |
| `DELETE` | `/api/todos/completed`   | Clear completed todos    |

## Project Structure

```
src/
  main.rs              # Axum server entry point
  lib.rs               # Library root
  app.rs               # Leptos root App component
  api.rs               # REST API handlers
  db.rs                # SQLite pool and migrations
  models.rs            # Todo, CreateTodo, UpdateTodo types
  components/
    mod.rs             # Component re-exports
    header.rs          # TodoHeader — title + new-todo input
    todo_item.rs       # TodoItem — toggle, delete, inline edit
    footer.rs          # TodoFooter — count, filters, clear
migrations/
  001_create_todos.sql # Initial schema
tests/
  api_tests.rs         # API integration tests
  e2e_tests.rs         # End-to-end flow tests
style/
  input.css            # Tailwind + TodoMVC styles
```
