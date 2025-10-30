# Project structure

```
.
├── Cargo.toml            # workspace
├── compose.yml           # docker compose for db, backend, frontend
├── backend/
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── migrations/
│   │   └── 2025-10-30-000001_create_posts_table.sql
│   └── src/
│       └── main.rs
└── frontend/
    ├── Cargo.toml
    ├── Dockerfile
    └── src/
        └── main.rs
```

# Sviluppo locale

- Avvia tutto:

```
docker compose up --build
```

- Backend API:
  - GET/POST /api/posts
  - GET/PUT /api/posts/:id
  - GET /             -> JSON con ultimi post (per semplicità)

- Frontend (Leptos):
  - http://localhost:3001

# Note
- Variabili d'ambiente importanti:
  - DATABASE_URL=postgres://app:app@db:5432/appdb
  - HOST, PORT per il backend

- Migrazioni SQLx: usiamo file .sql in backend/migrations e macro `sqlx::migrate!()`

# Build produzione

- Backend: multi-stage con distroless
- Frontend: build statico servito da Nginx
