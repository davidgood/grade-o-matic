# grade-o-matic

Grade-O-Matic is a classroom management and grading platform built with Rust and Axum.
It includes JWT-based auth, role-aware UI pages, and server-rendered templates with Minijinja.

Inspired by CMU's [autolab project](https://github.com/autolab/Autolab) (which seems abandoned) and GitHub classrooms.

This was born with some custom Python scripts I wrote to fetch student code from GitHub classrooms,
then compile (for compiled code) and run unit and integration tests. They've been working well for me,
but I wanted to try something new.

I wanted to evolve the "project" and improve my Rust skills while also adding features that I use in my own classroom.

I bootstrapped this project with [clean-axum-demo](https://github.com/sukjaelee/clean_axum_demo) as it seemed
to be a good starting point with an architecture very close to how I would design a layered web app, thus I can
understand it better. My background is primarily C, C++, C#, and Python.

## Project status

This project is currently a **work in progress**. Features, routes, and data models are still evolving.

## Tech stack

- Rust + Axum
- SQLx (Postgres + Migrations + offline validation)
- Minijinja (because I know Django and it works.)
- Tailwind for styling
- HTMX and Alpine.js
- Docker / Docker Compose
- PostgreSQL

## How it works

- API routes are mounted in `src/app.rs` (domain routers under `/auth`, `/classes`, `/assignments`, etc).
- Web UI routes are mounted from `src/web/routes.rs` (`/ui/...` pages).
- UI templates are Minijinja HTML files in `templates/` and are registered in `src/web/mod.rs`.
- Static files are served from `assets/public` at `/assets/public/...`.

## Run locally

1. Configure environment variables (copy `.env.example` to `.env` and adjust values).
2. Build frontend CSS:

```bash
make css
```

3. Seed database:

```bash
make seed
```

4. Start server:

```bash
cargo run
```

## Use Docker

A Dockerfile is provided that uses `cargo-chef` to build a production-ready image.

There are several docker-compose files to choose from. I already run a postgres server locally in
a separate container, so the root docker-compose file is only the grade-o-matic app.

The `docker-compose.yml` expects to find your environment variables in a `.env` file.

If you already have a postgres container running, and want to use it:

```bash
  docker compose up --build
```

If you want to run the app and postgres together:

```bash
  docker compose -f docker-compose.yml up --build
```

Add `-d` to run in detached mode as a background process.

5. Open:

- App UI: `http://localhost:3000/`
- Login page: `http://localhost:3000/ui/login`
- API docs: `http://localhost:3000/docs`
- Health: `http://localhost:3000/health`

## Seeded login accounts

From `db-seed/seed.sql`:

- `admin01` / `test_password`
- `instructor01` / `test_password`
- `apitest01` / `test_password`

## UI auth and role behavior

- Browser UI uses JWT via `auth_token` cookie.
- Unauthenticated `/ui/*` requests redirect to `/ui/login`.
- Admin pages (for example `/ui/admin/users`) require admin role.
- Instructor pages:
    - `/ui/instructors`: classes owned by the logged-in instructor
    - `/ui/instructors/classes/{id}`: class detail + assignments for that class

## Frontend styles (SCSS)

SCSS sources live in `scss/` and compile to `assets/public/grade-o-matic.min.css`, served at:

- `/assets/public/grade-o-matic.min.css`

Build styles:

```bash
make css
```

## Database seeding

Seed using `DATABASE_URL`:

```bash
make seed
```

Seed test DB using `DATABASE_URL_TEST`:

```bash
make seed-test
```

## CI environment contract

CI is configured to run without relying on a local `.env` file. The workflow sets:

- `SQLX_OFFLINE=true`
- `JWT_SECRET_KEY=ci-test-jwt-secret`
- `OIDC_ENABLED=false`

Notes:

- `SQLX_OFFLINE=true` prevents SQLx compile-time DB lookups during CI builds.
- `JWT_SECRET_KEY` ensures JWT-dependent tests and middleware initialize.
- `OIDC_ENABLED=false` avoids OIDC config failures when vars are not provided.
