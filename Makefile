css:
	sass apps/web/scss/app.scss apps/web/assets/public/grade-o-matic.min.css --style=compressed

chef-recipe:
	docker run --rm -v "$$(pwd)":/app -w /app lukemathwalker/cargo-chef:latest-rust-slim-trixie cargo chef prepare --recipe-path recipe.json

seed:
	psql "$${DATABASE_URL:?DATABASE_URL is not set}" -v ON_ERROR_STOP=1 -f db-seed/seed.sql

seed-test:
	psql "$${DATABASE_URL_TEST:?DATABASE_URL_TEST is not set}" -v ON_ERROR_STOP=1 -f db-seed/seed.sql
