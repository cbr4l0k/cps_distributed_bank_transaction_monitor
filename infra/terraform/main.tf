provider "digitalocean" {
  token = var.do_token
}

resource "digitalocean_ssh_key" "cps" {
  name       = "cps-bank-lab-key"
  public_key = file(pathexpand(var.ssh_public_key_path))
}

resource "digitalocean_droplet" "receiver" {
  image    = "ubuntu-24-04-x64"
  name     = "cps-bank-receiver"
  region   = var.region
  size     = var.droplet_size
  ssh_keys = [digitalocean_ssh_key.cps.fingerprint]

  tags = ["cps-bank-lab", "terraform"]

  user_data = templatefile("${path.module}/user-data.yaml.tftpl", {
    receiver_port  = var.receiver_port
    dashboard_port = var.dashboard_port
  })
}

resource "digitalocean_firewall" "receiver" {
  name        = "cps-bank-receiver-fw"
  droplet_ids = [digitalocean_droplet.receiver.id]

  inbound_rule {
    protocol         = "tcp"
    port_range       = "22"
    source_addresses = var.admin_cidrs
  }

  inbound_rule {
    protocol         = "udp"
    port_range       = tostring(var.receiver_port)
    source_addresses = var.pi_cidrs
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = tostring(var.dashboard_port)
    source_addresses = var.admin_cidrs
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "icmp"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }
}
