variable "do_token" {
  description = "DigitalOcean API token."
  type        = string
  sensitive   = true
}

variable "region" {
  description = "DigitalOcean region."
  type        = string
  default     = "nyc1"
}

variable "droplet_size" {
  description = "Low-cost size."
  type        = string
  default     = "s-1vcpu-1gb"
}

variable "ssh_public_key_path" {
  description = "Path to your SSH public key."
  type        = string
  default     = "~/.ssh/id_ed25519.pub"
}

variable "admin_cidrs" {
  description = "CIDR blocks allowed for SSH and dashboard, e.g. [\"203.0.113.10/32\"]."
  type        = list(string)
}

variable "pi_cidrs" {
  description = "CIDR blocks allowed to send UDP datagrams to the receiver."
  type        = list(string)
}

variable "receiver_port" {
  description = "UDP port used by receiver."
  type        = number
  default     = 9876
}

variable "dashboard_port" {
  description = "Tiny read-only dashboard port (Restrict to admin_cidrs)."
  type        = number
  default     = 8080
}
