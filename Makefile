css:
	sass scss/app.scss assets/public/grade-o-matic.min.css --style=compressed

seed:
	psql "$${DATABASE_URL:?DATABASE_URL is not set}" -v ON_ERROR_STOP=1 -f db-seed/seed.sql

seed-test:
	psql "$${DATABASE_URL_TEST:?DATABASE_URL_TEST is not set}" -v ON_ERROR_STOP=1 -f db-seed/seed.sql
