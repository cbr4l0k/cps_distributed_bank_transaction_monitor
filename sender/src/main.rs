mod config;
mod sequence_counter;
mod transaction_generator;

use anyhow::{Error, Result};
use config::Config;
use sequence_counter::SequenceCounter;
use tokio::{
    net::UdpSocket,
    time::{Duration, sleep},
};
use transaction_generator::TransactionGenerator;

use disgrams::{crypto::encrypt_packet, datagram::Header};

const SEQUENCE_COUNTER_PATH: &str = ".sender_sequence_counter";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let conf = Config::build()?;
    let target = conf.get_address()?;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let key = conf.get_cipher_key()?;

    let mut sequence_counter = SequenceCounter::open(SEQUENCE_COUNTER_PATH)?;
    let mut transaction_generator = TransactionGenerator::new()?;

    println!("Starting to send transactions to {}", target);
    loop {
        let header = Header::new(369u16, sequence_counter.current());
        let generated = transaction_generator.next();

        if generated.is_suspicious {
            println!(
                "sending suspicious transaction reason={:?} transaction={:?}",
                generated.reason, generated.transaction
            );
        }

        let packet = encrypt_packet(
            &key,
            header,
            Config::get_random_nonce().expect("this cant' fail"),
            generated.transaction,
        )?;

        socket.send_to(packet.as_slice(), target.clone()).await?;
        sequence_counter.increment()?;
        sleep(Duration::from_millis(90)).await;
    }
}
