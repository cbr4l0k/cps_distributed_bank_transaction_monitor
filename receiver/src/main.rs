mod config;
mod db;

use anyhow::{Error, Result};
use tokio::net::UdpSocket;

use disgrams::{
    crypto::{PACKET_LEN, decrypt_packet},
    datagram::extract_node_id,
};

use crate::db::get_key_for_node_as_hex;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let conf = config::Config::build().expect("Failed to build configuration");

    let address = conf.get_address()?;
    let socket = UdpSocket::bind(address).await?;
    let mut buf = vec![0u8; PACKET_LEN];

    let pool = db::connect(conf.get_db_url()?).await?;

    for node_id in conf.get_node_ids() {
        db::generate_key_for_node(&pool, *node_id).await?;
        println!(
            "NODE_KEY node_id={} key_hex={}",
            node_id,
            get_key_for_node_as_hex(&pool, *node_id).await?
        );
    }

    loop {
        let (n, addr) = socket.recv_from(&mut buf).await?;
        let packet = &buf[..n];

        let node_id = extract_node_id(packet)?;
        let key = db::get_key_for_node(&pool, node_id).await?;

        match decrypt_packet(&key, packet) {
            Ok((header, transaction)) => {
                println!("from={addr} header={header:?} transaction={transaction:?}");
                db::insert_transaction(&pool, &header, &transaction).await?;
            }
            Err(e) => {
                eprintln!("dropped invalid packet from {addr}: {e}");
            }
        }
    }
}
