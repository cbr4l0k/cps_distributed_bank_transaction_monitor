mod config;
use anyhow::{Error, Result};
use tokio::net::UdpSocket;

use disgrams::crypto::{PACKET_LEN, decrypt_packet, extract_node_id};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let key = [7u8; 32];
    let conf = config::Config::build()?;
    let address = conf.get_address()?;

    let socket = UdpSocket::bind(address).await?;
    let mut buf = vec![0u8; PACKET_LEN];

    loop {
        let (n, addr) = socket.recv_from(&mut buf).await?;

        let id = extract_node_id(&buf)?;
        println!("this is the ID {} of the machine.", id);
        match decrypt_packet(&key, &buf[..n]) {
            Ok((header, transaction)) => {
                println!("from={addr} header={header:?} transaction={transaction:?}");
            }
            Err(e) => {}
        }
    }
}
