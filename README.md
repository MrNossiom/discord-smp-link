# Discord SMP Link

## Running

Fill the `.env.docker` with the required credentials and settings :

-   `DATABASE_URL`: A MySQL connection URL (`mysql://server:server@127.0.0.1/server`)
-   `DISCORD_TOKEN`: A `Discord` token with the `bot` **AND** `application.commands` scopes
-   `GOOGLE_CLIENT_ID`: A Google OAuth2 web client ID (`xxx.apps.googleusercontent.com`)
-   `GOOGLE_CLIENT_SECRET`: A Google OAuth2 web client secret

-   `LOGS_WEBHOOK`: A `Discord` webhook URL (`https://discord.com/api/webhooks/channel_id/xxx`)
-   `SERVER_URL`: The URL to join our server over the Internet
-   `PORT`: The port on which the HTTP server must be bind (defaults to `80`)
-   `PORT_HTTPS`: The port on which the HTTPS server must be bind (`443`)
-   `PRODUCTION`: `true` or `false`

Build the production docker container:

> `docker build . -t ghcr.io/mrnossiom/discord-smp-link:latest`
> and lunch it with
> `docker compose -f docker-compose.production.yml up -d` or `just up production`

## Development

To run the local MySQL database, you can use the `docker-compose.local.yml` file:

> `docker compose -f docker-compose.local.yml up -d`
> or
> `just up local`

You can use [`mkcert`](https://github.com/FiloSottile/mkcert) to generate a local certificate for the server in development:

> `mkcert -install` and then `mkcert localhost`

## Compiling

You need to install the C library `mysql-client` before compiling the Rust code.

-   **`Linux (Debian/Ubuntu)`**
    Install the MySQL (MariaDB) driver and the associated development packages.

    > `apt install libmysqlclient-dev libmysqlclient21`

-   **`MacOS (w/ Homebrew)`**
    Since the `mysql-client` package is keg-only, you need to force link it with :
    > `brew link mysql-client --force`

## Run

### Vector

Get the config file for `Vector` with the LogTail settings:

> `wget -O ->> vector.toml https://logtail.com/vector-toml/docker/<id>`
