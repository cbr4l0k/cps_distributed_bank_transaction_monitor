mod config;
mod sequence_counter;
mod transaction_generator;

use anyhow::{Error, Result};
use config::Config;
use sequence_counter::SequenceCounter;
use tokio::{net::UdpSocket, time::sleep};
use transaction_generator::TransactionGenerator;

use disgrams::{
    crypto::encrypt_packet,
    datagram::Header,
    transaction::{self, Transaction, TransactionType},
};

const SEQUENCE_COUNTER_PATH: &str = ".sender_sequence_counter";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let conf = Config::build()?;
    let node_id = conf.get_node_id();
    let target = conf.get_address()?;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let key = conf.get_cipher_key()?;

    let mut sequence_counter =
        SequenceCounter::open(format!("{}_{}", SEQUENCE_COUNTER_PATH, node_id))?;
    let mut transaction_generator = TransactionGenerator::new()?;

    println!(
        "Starting node_id={} to send transactions to {}",
        node_id, target
    );
    loop {
        let header = Header::new(node_id, sequence_counter.current());
        let transaction = if conf.tamper_active() {
            let tx = Transaction::new(999_999, 25_000.00, TransactionType::Transfer);
            println!("sending PHYSICAL_TAMPER transaction={:?}", tx);
            tx
        } else {
            let generated = transaction_generator.next();
            if generated.is_suspicious {
                println!(
                    "sending suspicious transaction reason={:?} transaction={:?}",
                    generated.reason, generated.transaction
                );
            }
            generated.transaction
        };

        let packet = encrypt_packet(
            key,
            header,
            Config::get_random_nonce().expect("this cant' fail"),
            transaction,
        )?;

        socket.send_to(packet.as_slice(), target.clone()).await?;
        sequence_counter.increment()?;
        sleep(transaction_generator.next_delay()).await;
    }
}
