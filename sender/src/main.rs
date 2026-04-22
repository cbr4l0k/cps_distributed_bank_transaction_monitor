mod config;
mod sequence_counter;

use anyhow::{Error, Result};
use config::Config;
use sequence_counter::SequenceCounter;
use tokio::{
    net::UdpSocket,
    time::{Duration, sleep},
};

use disgrams::{
    crypto::encrypt_packet,
    datagram::Header,
    transaction::{Transaction, TransactionType},
};

const SEQUENCE_COUNTER_PATH: &str = ".sender_sequence_counter";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let conf = Config::build()?;
    let target = conf.get_address()?;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let key = conf.get_cipher_key()?;

    let mut sequence_counter = SequenceCounter::open(SEQUENCE_COUNTER_PATH)?;

    println!("Starting to send transactions to {}", target);
    loop {
        let header = Header::new(369u16, sequence_counter.current());
        let transaction = Transaction::new(1u32, 300f32, TransactionType::Deposit);

        let packet = encrypt_packet(
            &key,
            header,
            Config::get_random_nonce().expect("this cant' fail"),
            transaction,
        )?;

        socket.send_to(packet.as_slice(), target.clone()).await?;
        sequence_counter.increment()?;
        sleep(Duration::from_millis(90)).await;
    }
}
