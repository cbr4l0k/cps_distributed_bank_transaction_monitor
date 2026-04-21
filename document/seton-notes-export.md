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
