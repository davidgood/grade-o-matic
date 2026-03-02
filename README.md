# grade-o-matic

## CI Environment Contract

CI is configured to run builds and tests without relying on a local `.env` file.

The GitHub Actions workflow sets:

- `SQLX_OFFLINE=true`
- `JWT_SECRET_KEY=ci-test-jwt-secret`
- `OIDC_ENABLED=false`

Testing context:

- `SQLX_OFFLINE=true` prevents SQLx compile-time DB lookups during CI builds.
- `JWT_SECRET_KEY` ensures JWT-dependent tests and middleware can initialize.
- `OIDC_ENABLED=false` avoids failing config validation when OIDC vars are not provided in CI.

Integration tests also set safe defaults in test helpers when these vars are missing, so local test runs do not require
`.env`.

## Frontend styles (SCSS)

SCSS sources live in `scss/` and compile to `assets/public/grade-o-matic.min.css`, which is served at
`/assets/public/grade-o-matic.min.css`.

Build styles:

```bash
make css
```

## Database seeding

Seed using `DATABASE_URL`:

```bash
make seed
```

Seed using `DATABASE_URL_TEST`:

```bash
make seed-test
```
