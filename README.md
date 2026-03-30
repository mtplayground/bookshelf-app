# Bookshelf App

A full-stack Rust web application for managing your personal book collection. Browse, search, and organize books and authors with a modern, responsive UI.

## Tech Stack

- **Backend**: [Axum](https://github.com/tokio-rs/axum) 0.8 (Rust async web framework)
- **Frontend**: [Leptos](https://github.com/leptos-rs/leptos) 0.7 (Rust/WASM CSR framework)
- **Database**: SQLite via [sqlx](https://github.com/launchbadge/sqlx) with compile-time checked queries
- **Build**: [Trunk](https://trunkrs.dev/) for WASM frontend, Cargo for backend
- **Shared Types**: Common `types` crate used by both frontend and backend

## Features

- Books & Authors CRUD with form validation
- Search books by title, author, or ISBN
- Filter by genre and sort by title, rating, or year
- Statistics dashboard (totals, ratings, genre breakdown, top books)
- Toast notifications and confirm-before-delete dialogs
- Responsive design with loading spinners and empty states
- Health check endpoint and CORS support

## Prerequisites

- [Rust](https://rustup.rs/) (2024 edition)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)

## Getting Started

### Build and run (production mode)

```bash
# Build the frontend WASM bundle
trunk build --release frontend/index.html

# Build and run the backend (serves API + static frontend)
cargo run --release -p backend
```

The app will be available at `http://localhost:8080`.

### Development mode

Run the backend and frontend dev server separately:

```bash
# Terminal 1: Backend API server
cargo run -p backend

# Terminal 2: Frontend dev server with hot reload (proxies API to backend)
trunk serve frontend/index.html
```

Frontend dev server runs at `http://localhost:8081` and proxies `/api/` requests to the backend on port 8080.

### Run tests

```bash
cargo test -p backend
```

## API Endpoints

### Health

| Method | Path           | Description        |
|--------|----------------|--------------------|
| GET    | `/api/health`  | Health check       |

### Authors

| Method | Path               | Description         |
|--------|--------------------|---------------------|
| GET    | `/api/authors`     | List all authors    |
| POST   | `/api/authors`     | Create an author    |
| GET    | `/api/authors/{id}`| Get author + books  |
| PUT    | `/api/authors/{id}`| Update an author    |
| DELETE | `/api/authors/{id}`| Delete an author    |

### Books

| Method | Path                | Description                          |
|--------|---------------------|--------------------------------------|
| GET    | `/api/books`        | List books (search, genre, sort)     |
| POST   | `/api/books`        | Create a book                        |
| GET    | `/api/books/{id}`   | Get book with author                 |
| PUT    | `/api/books/{id}`   | Update a book                        |
| DELETE | `/api/books/{id}`   | Delete a book                        |
| GET    | `/api/books/stats`  | Dashboard statistics                 |

#### Query Parameters for `GET /api/books`

| Param    | Description                                      |
|----------|--------------------------------------------------|
| `search` | Search by title, author name, or ISBN             |
| `genre`  | Filter by exact genre                             |
| `sort`   | `title`, `title_desc`, `rating`, `year`, `newest` |

## Project Structure

```
bookshelf-app/
  backend/          # Axum API server
    src/
      main.rs       # Entry point
      lib.rs        # App builder (used by tests)
      db.rs         # SQLite pool + migrations
      errors.rs     # Error types
      routes/       # API route handlers
    migrations/     # SQL migrations
    tests/          # Integration tests
  frontend/         # Leptos WASM client
    src/
      main.rs       # Entry point
      app.rs        # Router + layout
      api.rs        # HTTP client functions
      toast.rs      # Toast notification system
      pages/        # Page components
    index.html      # HTML shell
    style.css       # Styles
  types/            # Shared request/response types
```

## License

MIT
