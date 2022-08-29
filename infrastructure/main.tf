terraform {
  required_version = "> 1.0.0"

  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

provider "digitalocean" {}

resource "digitalocean_droplet" "client" {
  image     = "ubuntu-22-04-x64"
  name      = "discord-smp-link"
  region    = "fra1"
  size      = "s-1vcpu-512mb-10gb"
  user_data = file("client_setup.yml")
}
