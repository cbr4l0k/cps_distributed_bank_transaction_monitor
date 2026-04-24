Context:
- cyber-physical systems
- project

For this assignment, the idea is to implement a cyber-physical system
composed by two `Raspberry Pi Zero 2W` nodes, simulating bank behavior;
transmiting cryptographically secured transaction datagrams over `UDP` to a
centralized analysis server hosted in Digital Ocean.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Level 1 — Physical layer

As mentioned before, for this layer, there will be two `Raspberry Pi Zero
2W`, with headless alpine linux as OS. And connected bia either WiFi or my
mobile hotspot; depending on how well the configuration works in ITMO's
network.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Level 2 — Data Acquisition/Generation

Each node will be running a rust binary, that will generate synthetic bank
transactions. This transactions will be composed by the fields: `node_id`,
`account_id`, `amount`, `tx_type`, and `timestamp`. And finally serialized
by a serialization/desialization crate that will be shared between both
between the nodes and the server for standard procesing.

---

Context:
- cyber-physical systems
- project
- CPS Levels
- cryptography
- AEAD GCM

# Level 3 — Network/Datagram layer

By using UDP sockets, the raspberries will be sending datagrams to the
server using the following structure (previously serialized):

```
[node_id:2B][seq:4B][timestaump:8B][nonce:12B][ciphertext:12B][hmac_tag:32B]
```

> cipher: because transactions are confidential and require proof of non
> tempering, I'll be using the AEAD GCM schema. Ciphering the `Transaction`
> using AES-256-GCM and then signing the whole packet (minus the tag xd)
> using HMAC-SHA256. This way both the ciphered protection is not spied,
> and any change on even plain information sent, marks as not valid.

> datagram logic:
> - node_id: I'm having 2 rasberry pi... and this bytes give me more than
>   enough room for identifying each of them, as separate devices.
> - seq: for each node, the number of the transaction. With 4B, there's
>   $2^{8*4}$ possible number; which, is more than enough for sake of this
>   demo.
> - timestamp: this are formated as `u64`, therefore 8B are requiered to
>   store this info.
> - nonce: This is used to generate the `GHASH`, and is required to either
>   have 12B; which, then is followed by 4B counter when actually using it
>   to generate the keystream.
> - ciphertext:Rust say it takes 12B to store the information in this
>   specific data structure... then, after ciphered, the size should stay
>   the same. 
> - hmac_tag: this is used 32B by default, and this is the piece of the
>   package that will tell if the data was tempered or not.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Level 4 — Processing layer

This layer will start by checking the tag, confirming that the information
is not corrupted; and, descarting it if it is. Following by deserializing
the data, and storing the transactions. 

Also, just to add some kind of anomaly detection, there will be a window
covering the last 50 transactions, and it will flag a transaction on
storage if there's something weird, like if there's been way too many
transactions in a certain period of time, or if the amount of the
transaction is way higher (2 or 3 standar dev.) higher than usual.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Level 5 — Analytical center

DigitalOcean droplet, that stores the valid transactions and also hosting a
light dashboard that shows live feed of the transaction; with an
auto-refresh of every couple of seconds. 

This part will be deployed using terraform, to keep consistency, with the
firewall for port oppening. And for the SSH key injection so that the
processing layer is able to check and decipher the incoming packages.


---

Context:
- cyber-physical systems
- project
- terraform

# Terraform idea

Terraform is being used to create the cloud part of the project.

The goal is to avoid creating the DigitalOcean server manually, because then the
setup would be harder to repeat or explain.

---

Context:
- cyber-physical systems
- project
- digital ocean

# DigitalOcean droplet

The droplet is the central server of the system.

It will receive the datagrams sent by the Raspberry Pi nodes and run the
`receiver` binary.

---

Context:
- cyber-physical systems
- project
- cost

# Cheap infrastructure

The infrastructure is intentionally small.

Only one droplet is used, because this project only needs to receive data from
two Raspberry Pi nodes.

---

Context:
- cyber-physical systems
- project
- terraform

# Why not more cloud services

There is no Kubernetes, no managed database, and no load balancer.

For this demo, that would be too much and not really needed.

---

Context:
- cyber-physical systems
- project
- raspberry pi

# Raspberry Pi nodes

The two Raspberry Pi Zero 2W boards are the physical nodes.

Each one runs the `sender` binary and generates synthetic bank transactions.

---

Context:
- cyber-physical systems
- project
- nodes

# Node identity

Each Raspberry Pi has a different `node_id`.

This is needed so the server can know which physical node sent each transaction.

---
Context:

- cyber-physical systems
- project
- network

# Node to server relationship

The relationship is simple:

```text
Raspberry Pi 1 ---> DigitalOcean receiver
Raspberry Pi 2 ---> DigitalOcean receiver
```

Both nodes send data to the same server.

---

Context:
- cyber-physical systems
- project
- UDP

# UDP port

The receiver listens on a UDP port.

Terraform opens this port in the firewall, so the Raspberry Pis can send the
datagrams to the droplet.

---

Context:
- cyber-physical systems
- project
- firewall

# Firewall

The firewall is created with terraform too.

Only the ports needed for the project should be opened.

---

Context:
- cyber-physical systems
- project
- SSH

# SSH access

Terraform adds my SSH key to the droplet.

This lets me connect to the server without setting a password manually.

---

Context:
- cyber-physical systems
- project
- deployment

# Terraform output

After creating the droplet, terraform gives back the public IP.

That IP is then used in the Raspberry Pi sender configuration.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Physical layer match

The physical layer is represented by the two Raspberry Pis.

Terraform does not create this part, because these are real devices.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Data generation match

Each Raspberry Pi generates transactions locally.

The data is not created in the cloud, it starts from the physical nodes.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Network layer match

The network layer is the UDP communication between the Raspberry Pis and the
droplet.

The datagrams travel through the network to the receiver.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Processing layer match

The receiver running in the droplet checks and processes the datagrams.

This is where the transaction becomes useful data for the system.

---

Context:
- cyber-physical systems
- project
- CPS Levels

# Analytical center match

The droplet is the analytical center.

It stores the valid transactions and can show what is happening in the system.

---

Context:
- cyber-physical systems
- project
- current work

# Current infrastructure work

The current work is to connect the physical Raspberry Pi nodes with the cloud
receiver.

Terraform creates the cloud side, and the Raspberry Pis connect to it after
that.

---

Context:
- cyber-physical systems
- project
- final idea

# Final idea

The project is not only a local simulation.

It uses real Raspberry Pi devices, a real network, and a real server deployed in
DigitalOcean.

---

Context:
- cyber-physical systems
- project
- missing

# Stuff I'm missing :D

## The hardware stuff, I should buy:

- 2 push buttons or magnetic reed switches
- 4 LEDs
- 4 resistors, 220Ω to 330Ω
- jumper wires and breadboard

## Networking issues

Arranging the interconnection between the Raspberries and the DO server has
been more challenging than expected... because of the network restrictions,
getting certificates, even updating the internal clock of the raspberries
has been challenging; therefore, will be part of the second assingment.
