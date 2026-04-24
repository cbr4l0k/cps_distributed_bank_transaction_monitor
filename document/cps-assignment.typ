#set document(
  title: "Cyber-Physical System: Distributed Bank Transaction Monitor",
  author: "Man o nena tan lind@  el que lea esto <3",
)

#set page(
  paper: "a4",
  margin: (top: 2.5cm, bottom: 2.5cm, left: 2.8cm, right: 2.8cm),
  numbering: "1",
)

#set text(
  font: "New Computer Modern",
  size: 11pt,
  lang: "en",
  hyphenate: false,
)

#set par(
  justify: true,
  leading: 0.75em,
  spacing: 1.2em,
)

#set heading(numbering: "1.")

#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(1.2em)
  text(13pt, weight: "bold", it)
  v(0.4em)
}

#show heading.where(level: 2): it => {
  v(0.8em)
  text(11pt, weight: "bold",it)
  v(0.3em)
}

#v(1fr)

#align(center)[
  #text(16pt, weight: "bold")[
    Cyber-Physical System: \
    Distributed Bank Transaction Monitor
  ]
  #v(0.5em)
  #text(11pt, fill: rgb("#555555"))[
    Juan Pablo Sierra Useche
    (473836\/4150C) \
    Assignment — Cyber-Physical Systems \
    ITMO University · #datetime.today().display("[month repr:long] [year]")
  ]
  #v(1.5cm)
  #block(
    fill: rgb("#f7f9fb"),
    stroke: 0.5pt + rgb("#b0bec5"),
    radius: 4pt,
    inset: 10pt,
  )[
    #text(font: "Courier New", size: 9.5pt)[
      *Disclaimer:* The following document was developed as a series of
      notes describing the design and methodologies behind this project.
      The text was then structured and translated using an LLM, and
      finally added manually to a `typst` template.
    ]
  ]
]

#v(1fr)

#pagebreak()

#outline(
  title: "Contents",
  indent: auto,
)

= Plan

== Purpose of Use

The system simulates a minimal banking infrastructure: two edge computing
nodes send synthetic transactions over an encrypted channel to a central
server that validates, stores, and flags anomalies.

The design follows a pattern common in financial IoT deployments: constrained
edge devices generate sensitive data, the data must stay confidential in
transit, and a cloud-hosted center catches anomalies. The implementation
covers all five CPS layers.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== System Architecture Overview

The system has five layers following the CPS reference model. Two
*Raspberry Pi Zero 2W* nodes are the physical edge; a *DigitalOcean* droplet
is the analytical center. All communication between nodes and server uses
*UDP datagrams*, each carrying a cryptographically protected payload.


#figure(
image("./assets/cps_system_levels.svg", width: 89%),
  caption: [CPS layer summary],
)

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== Description of System Levels

=== Level 1 — Physical Layer

The physical layer consists of two *Raspberry Pi Zero 2W* single-board
computers, each running a headless *Alpine Linux* installation to minimize
resource overhead. Network connectivity is established via Wi-Fi, either
through the campus network or a mobile hotspot as fallback.

These are the constrained end of the stack: limited CPU, limited RAM, no
display, managed entirely over SSH. In this simulation they generate events
rather than sense real ones, then forward them upstream.

=== Level 2 — Data Acquisition and Generation

Each node runs a compiled *Rust binary* responsible for producing synthetic
bank transactions. A transaction is a structured record with the following
fields:

#table(
  columns: (auto, 1fr),
  inset: 7pt,
  stroke: 0.4pt + rgb("#bbbbbb"),
  fill: (x, y) => if y == 0 { rgb("#f0f4f8") } else { white },
  [*Field*], [*Description*],
  [`node_id`],    [Identifies the originating Raspberry Pi],
  [`account_id`], [Synthetic bank account identifier],
  [`amount`],     [Transaction value in arbitrary units],
  [`tx_type`],    [Category: deposit, withdrawal, transfer (for now)],
  [`timestamp`],  [Unix epoch in microseconds (`u64`)],
)

Records are serialized using a shared Rust crate common to both nodes and
the server, so there is one canonical binary format used throughout.

=== Level 3 — Network / Datagram Layer

Transactions are transmitted over *UDP sockets* as fixed-layout datagrams.
Each datagram follows the structure below:

#align(center)[
  #block(
    fill: rgb("#f7f9fb"),
    stroke: 0.5pt + rgb("#b0bec5"),
    radius: 4pt,
    inset: 10pt,
  )[
    #text(font: "Courier New", size: 9.5pt)[
      `[node_id:2B][seq:4B][timestamp:8B][nonce:12B][ciphertext:12B][hmac_tag:32B]`
    ]
  ]
]

#v(0.4em)

*Field rationale:*

- *`node_id (2B)`* — Two bytes are sufficient to address up to `65 536`
  distinct nodes; ample for the two nodes in this demo.

- *`seq (4B)`* — A per-node monotonically increasing counter. $2^32$
  possible values cover any realistic demo lifetime and allow replay-attack
  detection on the server side.

- *`timestamp (8B)`* — Stored as a `u64` Unix timestamp, requiring
  exactly 8 bytes. Enables latency measurement and ordering of out-of-
  sequence datagrams.

- *`nonce (12B)`* — The GCM standard nonce size. AES-256-GCM derives its
  keystream from a 96-bit nonce concatenated with a 32-bit counter block;
  the 12-byte field maps directly to this requirement.

- *`ciphertext (12B)`* — The encrypted `Transaction` struct. AES-GCM is a
  stream cipher mode, so ciphertext length equals plaintext length. The
  following struct will be the one encoded, and it sums `9B` plus `3B` of
  alignment; therefore, `12B` in total.

  ```rust
use std::mem::size_of;

#[repr(C)]
struct AccountId(u32);
#[repr(C)]
struct Amount(f32);
#[repr(u8)]
enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer,
}
#[repr(C)]
struct Transaction {
    account_id: AccountId,
    amount: Amount,
    tx_type: TransactionType,
}
fn main() {
    println!("AccountId: {}", size_of::<AccountId>());              // 4B
    println!("Amount: {}", size_of::<Amount>());                    // 4B
    println!("TransactionType: {}", size_of::<TransactionType>());  // 1B
    println!("Transaction: {}", size_of::<Transaction>());          // 12B
}
  ```

- *`hmac_tag (32B)`* — An HMAC-SHA256 digest computed over all preceding
  datagram fields (excluding the tag itself). This provides integrity
  verification of the unencrypted header fields that AEAD alone would not
  cover.

*Cipher design:* the `Transaction` payload is encrypted with
*AES-256-GCM*, an AEAD scheme that simultaneously provides
confidentiality and ciphertext authenticity. The outer *HMAC-SHA256* then
covers the entire datagram envelope — including cleartext metadata like
`node_id`, `seq`, and `timestamp` — ensuring that any tampering with the
unencrypted header is also detectable at the server.

=== Level 4 — Processing Layer

The server-side processing pipeline operates in the following sequence:

+ *Integrity check:* verify the HMAC-SHA256 tag; discard any datagram that
  fails validation before further processing.
+ *Decryption:* use AES-256-GCM to decrypt the ciphertext and verify the
  GCM authentication tag.
+ *Deserialization:* reconstruct the `Transaction` struct using the shared
  serde crate.
+ *Storage:* persist the validated transaction to the database.
+ *Anomaly detection:* evaluate the transaction against a rolling window of
  the last *50 records* per node. Two heuristics are applied:

  - *Velocity check:* flag if the number of transactions in a short time
    window exceeds a defined threshold (burst detection).
  - *Amount check:* flag if the transaction amount deviates more than
    $±2σ$ from the window mean (outlier detection).

Flagged transactions are stored with an anomaly marker rather than
discarded, preserving a complete audit trail.

=== Level 5 — Analytical Center

The analytical center is a *DigitalOcean* droplet that hosts both the
database and a lightweight web dashboard. Infrastructure is defined
entirely in *Terraform*, covering:

- Firewall rules for inbound UDP (datagram receiver) and HTTPS (dashboard);
- SSH key injection so the processing layer can establish secure management
  sessions;
- Droplet provisioning with a reproducible configuration.

The dashboard shows a live feed of incoming transactions: recent records,
flagged anomalies, and per-node statistics, refreshed every few seconds.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== Calculation Algorithms

=== Anomaly Scoring (Sliding Window)

Let $W = {a_1, a_2, dots, a_N}$ be the amounts of the last $N = 50$
transactions for a given node. Define:

$
  mu_W = frac(1, N) sum_{i=1}^{N} a_i, quad (#footnote[#link("https://en.wikipedia.org/wiki/Arithmetic_mean#Definition")[Formula source]]) \
  sigma_W = sqrt(frac(1, N - 1) sum_{i=1}^{N} (a_i - mu_W)^2)
  (#footnote[#link("https://en.wikipedia.org/wiki/Arithmetic_mean#Definition")[Formula
  source]]) (#footnote[Will be using the corrected standard deviation;
  because, it adjust the bias when estimating from a sample. Prividing a
  more accurate estimation for the small sample.])\

$

An incoming transaction with amount $a$ is flagged if:

$
  |a - mu_W| > 3 dot sigma_W
$

The threshold $k$ is configurable. Velocity anomalies are detected by
counting transactions whose timestamps fall within a sliding 60-second
window and comparing against a configurable burst limit.

=== Datagram Integrity Pipeline

#align(center)[
  #block(
    fill: rgb("#f7f9fb"),
    stroke: 0.5pt + rgb("#b0bec5"),
    radius: 4pt,
    inset: 10pt,
  )[
    #text(font: "Courier New", size: 9.5pt)[
      Transaction → [AES-256-GCM encrypt] → ciphertext → [HMAC-SHA256 sign] → datagram
    ]
  ]
]

On the receiver, the pipeline is strictly reversed: HMAC verification first,
then GCM decryption. A failure at any step causes the datagram to be
silently dropped and logged.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== Monitoring

The dashboard exposes the following live metrics:

#table(
  columns: (auto, 1fr),
  inset: 7pt,
  stroke: 0.4pt + rgb("#bbbbbb"),
  fill: (x, y) => if y == 0 { rgb("#f0f4f8") } else { white },
  [*Metric*], [*Description*],
  [Transaction feed],    [Chronological list of last $N$ validated transactions],
  [Anomaly feed],        [Transactions flagged by either heuristic, with reason],
  [Node heartbeat],      [Last-seen timestamp per node; stale if $> 30$ s],
  [Throughput],          [Transactions per second, computed over a 10 s window],
  [Drop rate],           [Ratio of discarded (invalid HMAC/GCM) datagrams],
)

All metrics are updated on each dashboard refresh cycle (every ~3 seconds).
Server-side logs retain the full event history for post-hoc analysis.

== Scope

The transaction system only checks whether a transaction is cryptographically
valid. Balance and availability checks are out of scope.

= Implementation

This section describes the system as actually built, including the decisions
that diverged from the plan.

== Project Structure

The Rust workspace is split into three binaries and one shared library:

#table(
  columns: (auto, 1fr),
  inset: 7pt,
  stroke: 0.4pt + rgb("#bbbbbb"),
  fill: (x, y) => if y == 0 { rgb("#f0f4f8") } else { white },
  [*Crate / file*], [*Role*],
  [`disgrams`],               [Shared library: packet layout, crypto, transaction types],
  [`sender`],                 [Runs on each Raspberry Pi; generates and transmits datagrams],
  [`receiver`],               [Runs on the DigitalOcean droplet; validates, decrypts, stores],
  [`cps-bank-dashboard.py`],  [Python HTTP server; reads the SQLite database and serves a live view],
)

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== Datagram Format Revision

The planned datagram used an outer HMAC-SHA256 to protect the cleartext header
fields. During implementation this was replaced with a simpler approach: the
header bytes are passed as *Additional Authenticated Data (AAD)* directly to
the AES-256-GCM cipher. The GCM authentication tag covers both the ciphertext
and the AAD, so any modification to the cleartext header causes decryption to
fail without needing a separate HMAC pass.

The revised structure is:

#align(center)[
  #block(
    fill: rgb("#f7f9fb"),
    stroke: 0.5pt + rgb("#b0bec5"),
    radius: 4pt,
    inset: 10pt,
  )[
    #text(font: "Courier New", size: 9.5pt)[
      `[node_id:2B][seq:4B][timestamp:8B][nonce:12B][ciphertext:9B][gcm_tag:16B]`
    ]
  ]
]

#v(0.3em)

Total packet size is 51 bytes. The 32-byte outer HMAC from the plan is gone;
the 16-byte GCM tag takes over both roles. The `Transaction` struct is
serialized manually (big-endian fields, no C alignment padding), so it fits in
9 bytes rather than the 12 assumed in the planning phase.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== Sender

The sender binary reads its configuration from environment variables:
`CIPHER_KEY` (hex-encoded 32-byte key), `NODE_ID`, `TARGET_HOST`, and
`TARGET_PORT`. A missing `CIPHER_KEY` is a fatal error at startup; everything
else falls back to a safe default.

The sequence counter is stored as a plain text file
(`.sender_sequence_counter_{node_id}`) with owner-only permissions (`0600`).
It is loaded at startup and written to disk after every successful send, so
the counter survives restarts without replay-attack risk.

=== Transaction Generator

Transaction amounts and timing are produced by a seeded
linear-congruential generator (LCG). A custom LCG instead of a dependency
keeps the binary cross-compilable for `armv6` (the Raspberry Pi Zero 2W
target) without pulling in crates that may not support the target.

Normal transactions are generated every 3.5–7 seconds with amounts between
10 and 500 arbitrary units. Every eighth transaction is a suspicious outlier
with an amount between 5 001 and 25 000 units, enough to trip the
amount-based alert on the dashboard.

Velocity anomalies are injected using a burst pattern: after every 42 normal
transactions the sender fires 10 packets at 20–80 ms intervals, then pauses
for 65 seconds before resuming normal cadence. This creates a repeatable
velocity spike that the monitoring layer can observe.

=== Physical Tamper Mode

The sender checks for the existence of a file whose path is given by the
`TAMPER_FILE` environment variable. When that file is present, every
outgoing packet is replaced with a fixed suspicious transaction
(`account_id=999 999`, `amount=25 000`, `tx_type=Transfer`). The file can
later be wired to a GPIO pin connected to a push button or reed switch,
simulating a tamper event at the physical layer. For now the triggering is
done in software; the actual GPIO wiring is part of the second assignment.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== Receiver

The receiver is an async Tokio binary. On startup it connects to a SQLite
database (via `sqlx`) and generates a random 32-byte AES key for each
configured node ID, storing it with `INSERT OR IGNORE` so keys persist across
restarts. The key for each node is printed to stdout, allowing the operator to
copy it into the matching sender's environment.

For each incoming UDP datagram the receiver:

+ Reads the first two bytes to identify the originating node.
+ Fetches that node's key from the database.
+ Calls `decrypt_packet`, which verifies the GCM tag (covering both header
  and ciphertext) and decrypts the payload.
+ On success, inserts the transaction into `received_transactions`.
  On failure, logs the error and discards the packet.

The `received_transactions` table has a `UNIQUE (node_id, seq)` constraint,
so a replayed datagram is silently rejected at the insert level.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== Dashboard

The dashboard is a single Python script serving a minimal HTML page over HTTP.
It reads directly from the SQLite database on each request and refreshes every
5 seconds. Two alert conditions are evaluated on each render:

- *Huge amount:* any transaction in the last 25 records with `amount ≥ 5 000`.
- *Stale node:* any node whose most recent transaction is older than 60 seconds.

The sliding-window $sigma$-based anomaly scoring described in the plan is not
yet wired up to the dashboard. The `get_last_fifty_trans` function exists in
the receiver codebase and the math is correct, but the result is currently not
passed to the dashboard or stored as an anomaly marker.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== Deployment

Three systemd services manage the running processes: `cps-bank-receiver`,
`cps-bank-dashboard`, and `cps-bank-sender`. Each runs under a dedicated
`cpsbank` user with a restricted privilege set (`NoNewPrivileges`,
`ProtectSystem`, `ProtectHome`, `PrivateTmp`). The receiver and dashboard
share the same database file at `/var/lib/cps-bank/receiver.db`.

The cloud infrastructure is defined in Terraform under `infra/terraform/`.
A single `terraform apply` provisions the full stack:

+ *Droplet:* an Ubuntu 24.04 `s-1vcpu-1gb` instance in the `nyc1` region,
  bootstrapped via a `cloud-init` template that installs the binaries and
  registers the systemd services.
+ *SSH key:* the operator's public key is uploaded to DigitalOcean and
  injected into the droplet at creation time, so no password is ever set.
+ *Firewall:*  three inbound rules are created:
  - SSH (TCP 22) allowed only from the `admin_cidrs` variable.
  - Receiver (UDP 9876) allowed only from `pi_cidrs` — the IP ranges of the
    Raspberry Pi nodes. Any datagram arriving from an address outside that
    list is dropped by the firewall before it even reaches the process.
  - Dashboard (TCP 8080) allowed only from `admin_cidrs`.
  All outbound traffic is unrestricted.
+ *Output:* after `apply`, Terraform prints the droplet's public IP, which
  is then copied into the sender's `TARGET_HOST` environment variable on
  each Raspberry Pi.

The IP allowlist exists because the Raspberry Pi addresses are fixed and
known ahead of time. Blocking unknown sources at the firewall before packets
reach the process is free hardening with no application changes needed.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== What Is Still Missing

The GPIO wiring is not done yet. The physical buttons and LEDs described in
the plan — 2 push buttons, 4 LEDs, resistors, jumper wires — are still
unconnected. The `TAMPER_FILE` mechanism is a software stand-in for now;
the actual wiring is deferred to the second assignment.

Network provisioning on ITMO's campus network has been more challenging than
expected. Certificate issues and NTP synchronization on the Raspberry Pi Zero 2W
nodes required manual intervention, so the end-to-end test between the physical
nodes and the DigitalOcean droplet is still pending.
