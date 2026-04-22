mod config;
use anyhow::{Error, Result};
use config::Config;
use tokio::{
    net::UdpSocket,
    time::{Duration, sleep},
};

use disgrams::{
    crypto::encrypt_packet,
    datagram::Header,
    transaction::{Transaction, TransactionType},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let conf = Config::build()?;
    let target = conf.get_address()?;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let key = conf.get_cipher_key()?;

    let mut i = 0u32;

    println!("Starting to send transactions to {}", target);
    loop {
        let header = Header::new(369u16, i);
        let transaction = Transaction::new(1u32, 300f32, TransactionType::Deposit);

        let packet = encrypt_packet(
            &key,
            header,
            Config::get_random_nonce().expect("this cant' fail"),
            transaction,
        )?;

        socket.send_to(packet.as_slice(), target.clone()).await?;
        // socket.send_to(b"\n", &target).await?;
        i += 1;
        sleep(Duration::from_millis(90)).await;
    }
}
