_default:
	@just --list --unsorted --list-heading '' --list-prefix '—— '

# Run your current project
run:
	RUST_LOG='info,_=warn,rocket=warn,discord_smp_link=debug' cargo run
# Start a developement tunnel to the local server
tunnel NAME:
	cloudflared tunnel run --url localhost:3000 {{NAME}}

# Starts the docker compose file with the provided scope
up SCOPE:
	docker compose --file docker-compose.{{SCOPE}}.yaml up -d
# Stops the docker compose file with the provided scope
down SCOPE:
	docker compose --file docker-compose.{{SCOPE}}.yaml down
# Builds the docker image with the provided tag
build TAG:
	docker build . -t ghcr.io/mrnossiom/discord-smp-link:{{TAG}}

# Retrieves the IP address of the local database
local-db-ip:
	@docker inspect -f {{"'{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}'"}} discord-smp-link-database-1