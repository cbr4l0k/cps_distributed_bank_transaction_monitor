output "droplet_ip" {
  value = digitalocean_droplet.receiver.ipv4_address
}

output "receiver_udp_endpoint" {
  value = "${digitalocean_droplet.receiver.ipv4_address}:${var.receiver_port}"
}

output "dashboard_url" {
  value = "http://${digitalocean_droplet.receiver.ipv4_address}:${var.dashboard_port}"
}

output "ssh_command" {
  value = "ssh root@${digitalocean_droplet.receiver.ipv4_address}"
}
