# Discord SMP Link [![WakaTime](https://wakatime.com/badge/github/mrnossiom/discord-smp-link.svg)](https://wakatime.com/badge/github/mrnossiom/discord-smp-link)

## Running

Fill the `.env.docker` with the required credentials and settings, see the `Config` struct in `src/config.rs` for more information on these.

Build the production docker container:

> `docker build . -t ghcr.io/mrnossiom/discord-smp-link:latest`
> and lunch it with
> `docker compose -f docker-compose.production.yaml up -d` or `just up production`

## Development

To run the local MySQL database, you can use the `docker-compose.local.yaml` file:

> `docker compose -f docker-compose.local.yaml up -d`
> or
> `just up local`

To test authentification across the internet, I recommend using [`cloudflared`](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation) to tunnel the authentification endpoint and server to the sub-domain of your choice, like `dev-smp-link.wiro.codes`.

> `Cloudflare` handles HTTPS for you, giving you direct HTTP requests.

```sh
cloudflared tunnel login
cloudflared tunnel create smp-link-dev
cloudflared tunnel route dns smp-link-dev smp-link-dev.wiro.codes
cloudflared tunnel run --url localhost:3000 smp-link-dev

# You can delete your tunnel with
cloudflared tunnel delete smp-link-dev
```

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

> `vector top --url http://<docker bridge ip>:8686/graphql`
