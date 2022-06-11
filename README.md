# Discord SMP Link

## Running

Fill the `.env.docker` with the required credentials and settings :

-   `DATABASE_URL`: A MySQL connection URL (`mysql://server:server@127.0.0.1/server`)
-   `DISCORD_TOKEN`: A `Discord` token with the `bot` **AND** `application.commands` scopes
-   `GOOGLE_CLIENT_ID`: A Google OAuth2 web client ID (`xxx.apps.googleusercontent.com`)
-   `GOOGLE_CLIENT_SECRET`: A Google OAuth2 web client secret

-   `LOGS_WEBHOOK`: A `Discord` webhook URL (`https://discord.com/api/webhooks/channel_id/xxx`)
-   `SERVER_URL`: The URL to join our server over the Internet
-   `PORT`: The port on which the server must be bind (`8080`)
-   `PRODUCTION`: `true` or `false`

Build the production docker container :

> `docker compose --file docker-compose.prod.yml up -d`

## Development

You need to install the C library `mysql-client` before compiling the Rust code.

-   **`Linux (Debian/Ubuntu)`**
    Install the MySQL (MariaDB) driver and the associated development packages.

    > `apt install libmysqlclient-dev libmysqlclient21`

-   **`MacOS (w/ Homebrew)`**
    Since the `mysql-client` package is keg-only, you need to force link it with :
    > `brew link mysql-client --force`
