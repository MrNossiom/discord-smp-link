_default:
	@just --list --unsorted --list-heading '' --list-prefix '—— '

# Builds your current project
build:
	@cargo build

# Builds your current project
run:
	@cargo run

# Starts your main docker compose file
up:
	@docker compose up -d

# Stops your main docker compose file
down:
	@docker compose down

# Starts your prodution docker compose file
prod-up:
	@docker compose --file docker-compose.prod.yml up -d

# Stops your prodution docker compose file
prod-down:
	@docker compose --file docker-compose.prod.yml down

purge-logs:
	@rm logs/log.*