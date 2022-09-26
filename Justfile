_default:
	@just --list --unsorted --list-heading '' --list-prefix '—— '

# Builds your current project
build:
	@cargo build
# Builds your current project
run:
	RUST_LOG='info,discord_smp_link=trace' cargo run

# Starts the docker compose file with the provided scope
up SCOPE:
	docker compose --file docker-compose.{{SCOPE}}.yml up -d
# Stops the docker compose file with the provided scope
down SCOPE:
	docker compose --file docker-compose.{{SCOPE}}.yml down
