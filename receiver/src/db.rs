use anyhow::{Error, Result, anyhow};
use disgrams::{datagram::Header, transaction::Transaction};
use getrandom::fill;
use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

pub async fn connect(database_url: &str) -> Result<SqlitePool, Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;
    migrate(&pool).await?;
    Ok(pool)
}

async fn migrate(db: &SqlitePool) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS nodes (
            node_id INTEGER PRIMARY KEY,
            decrypt_key BLOB NOT NULL
        )",
    )
    .execute(db)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS received_transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            node_id INTEGER NOT NULL,
            seq INTEGER NOT NULL,
            sent_timestamp_us INTEGER NOT NULL,
            account_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            tx_type INTEGER NOT NULL,
            received_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

            UNIQUE (node_id, seq),
            FOREIGN KEY (node_id) REFERENCES nodes(node_id)
        )",
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_key_for_node(db: &SqlitePool, node_id: u16) -> Result<[u8; 32], Error> {
    let row = sqlx::query("SELECT decrypt_key FROM nodes WHERE node_id = ?")
        .bind(node_id as i64)
        .fetch_one(db)
        .await?;
    let key_bytes: Vec<u8> = row.try_get("decrypt_key")?;

    key_bytes
        .try_into()
        .map_err(|_| anyhow!("Key for node {node_id} is not 32 bytes :("))
}

pub async fn generate_key_for_node(db: &SqlitePool, node_id: u16) -> Result<[u8; 32], Error> {
    let mut key = [0u8; 32];
    fill(&mut key)?;

    sqlx::query("INSERT OR IGNORE INTO nodes (node_id, decrypt_key) VALUES (?, ?)")
        .bind(node_id as i64)
        .bind(key.to_vec())
        .execute(db)
        .await?;
    Ok(key)
}

pub async fn insert_transaction(
    db: &SqlitePool,
    header: &Header,
    transaction: &Transaction,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO received_transactions (
            node_id,
            seq,
            sent_timestamp_us,
            account_id,
            amount,
            tx_type
         )
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(header.node_id as i64)
    .bind(header.seq as i64)
    .bind(header.timestamp.expect("no timestamp can't happen") as i64)
    .bind(transaction.account_id as i64)
    .bind(transaction.amount)
    .bind(transaction.get_transaction_type_as_number())
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_key_for_node_as_hex(db: &SqlitePool, node_id: u16) -> Result<String, Error> {
    let key = get_key_for_node(db, node_id).await?;
    Ok(hex::encode(key))
}

pub async fn get_last_fifty_trans(db: &SqlitePool) -> Result<Vec<Transaction>, Error> {
    let rows = sqlx::query(
        "SELECT account_id, amount, tx_type FROM received_transactions
         ORDER BY sent_timestamp_us DESC
         LIMIT 50",
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| row_to_transaction(&row))
        .collect()
}

fn row_to_transaction(row: &sqlx::sqlite::SqliteRow) -> Result<Transaction, Error> {
    let account_id = row.try_get::<i64, _>("account_id")? as u32;
    let amount = row.try_get::<f32, _>("amount")?;
    let tx_type_num = row.try_get::<i64, _>("tx_type")? as u8;
    let tx_type = match tx_type_num {
        0 => disgrams::transaction::TransactionType::Deposit,
        1 => disgrams::transaction::TransactionType::Withdrawal,
        2 => disgrams::transaction::TransactionType::Transfer,
        _ => {
            return Err(anyhow!("invalid transaction type: {tx_type_num}"));
        }
    };

    Ok(Transaction {
        account_id,
        amount,
        tx_type,
    })
}
