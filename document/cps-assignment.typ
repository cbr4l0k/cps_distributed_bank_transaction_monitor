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

The system described in this document simulates a minimal banking
infrastructure using two edge computing nodes — each acting as an independent
transaction source — connected to a centralized analytical server. The
primary objective is to study how cryptographically secured data can be
collected, transmitted, validated, and monitored across a physically
distributed system.

Beyond its role as a course project, the design reflects a realistic pattern
found in financial IoT deployments: constrained edge devices generate
sensitive data, the data must remain confidential in transit, and a
cloud-hosted center aggregates it for real-time anomaly detection. The
system therefore exercises the full CPS stack, from physical hardware up to
an interactive dashboard.

#v(0.4em)
#line(length: 100%, stroke: 0.3pt + rgb("#cccccc"))
#v(0.4em)

== System Architecture Overview

The system is composed of five hierarchical layers following the classical
CPS reference model. Two *Raspberry Pi Zero 2W* nodes act as the physical
edge; a *DigitalOcean* droplet serves as the analytical center. Communication
between nodes and server relies exclusively on *UDP datagrams*, each carrying
a cryptographically protected payload.


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

These nodes represent the constrained-device tier typical of real cyber-
physical deployments: limited CPU, limited RAM, no display, operated
entirely over SSH. Their role is to sense (or, in this simulation, generate)
events and forward them upstream.

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
the server, ensuring a single canonical binary representation across the
entire system.

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

The dashboard provides a live feed of incoming transactions with an
auto-refresh interval of a few seconds, displaying recent records, flagged
anomalies, and per-node statistics. No manual server intervention is
required after initial deployment.

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

The transaction system will be limited to check if the transaction is valid
or not; therefore, balance and availability checks will be neglectedchecks
will be neglected.

= Implementation
