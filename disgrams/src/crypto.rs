use aes_gcm::{Aes256Gcm, KeyInit, Nonce, Tag, aead::AeadInPlace};

use crate::datagram::{HEADER_LEN, Header};
use crate::errors::{DisgramsError, Result};
use crate::transaction::{TRANSACTION_LEN, Transaction};

pub const NONCE_LEN: usize = 12;
pub const GCM_TAG_LEN: usize = 16;

pub const PACKET_LEN: usize = HEADER_LEN + NONCE_LEN + TRANSACTION_LEN + GCM_TAG_LEN;

pub fn encrypt_packet(
    key: &[u8; 32],
    mut header: Header,
    nonce_bytes: [u8; 12],
    transaction: Transaction,
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| DisgramsError::InvalidKey)?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let aad = header.to_byte_stream();
    let plaintext = transaction.to_byte_stream();

    let mut ciphertext = plaintext.to_vec();
    let tag: Tag = cipher
        .encrypt_in_place_detached(nonce, &aad, &mut ciphertext)
        .map_err(|_| DisgramsError::EncryptionFailed)?;

    let mut packet = Vec::with_capacity(PACKET_LEN);
    packet.extend_from_slice(&aad);
    packet.extend_from_slice(&nonce_bytes);
    packet.extend_from_slice(&ciphertext);
    packet.extend_from_slice(&tag);

    Ok(packet)
}

pub fn decrypt_packet(key: &[u8; 32], packet: &[u8]) -> Result<(Header, Transaction)> {
    if packet.len() != PACKET_LEN {
        return Err(DisgramsError::InvalidPacketLength(packet.len(), PACKET_LEN));
    }

    let header = Header::from_byte_stream(packet[0..HEADER_LEN].try_into().unwrap());

    let aad = &packet[0..HEADER_LEN];
    let nonce = Nonce::from_slice(&packet[HEADER_LEN..HEADER_LEN + NONCE_LEN]);
    let mut ciphertext =
        packet[HEADER_LEN + NONCE_LEN..HEADER_LEN + NONCE_LEN + TRANSACTION_LEN].to_vec();
    let tag = Tag::from_slice(&packet[HEADER_LEN + NONCE_LEN + TRANSACTION_LEN..]);

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| DisgramsError::InvalidKey)?;
    cipher
        .decrypt_in_place_detached(nonce, aad, &mut ciphertext, tag)
        .map_err(|_| DisgramsError::DecryptionFailed)?;

    let transaction = Transaction::from_byte_stream(ciphertext.try_into().unwrap())?;

    Ok((header, transaction))
}

#[cfg(test)]
mod tests {
    use super::{PACKET_LEN, decrypt_packet, encrypt_packet};
    use crate::datagram::Header;
    use crate::errors::DisgramsError;
    use crate::transaction::{Transaction, TransactionType};

    #[test]
    fn encrypted_packet_round_trips_header_and_transaction() {
        let key = [7; 32];
        let nonce = [3; 12];
        let header = Header::new(42, 1337);
        let transaction = Transaction::new(123, 45.5, TransactionType::Deposit);

        let packet = encrypt_packet(&key, header, nonce, transaction).unwrap();
        let (decoded_header, decoded_transaction) = decrypt_packet(&key, &packet).unwrap();

        assert_eq!(packet.len(), PACKET_LEN);
        assert_eq!(decoded_header.node_id, header.node_id);
        assert_eq!(decoded_header.seq, header.seq);
        assert!(decoded_header.timestamp.is_some());
        assert_eq!(decoded_transaction, transaction);
    }

    #[test]
    fn decrypt_rejects_wrong_packet_length() {
        let key = [7; 32];
        let packet = vec![0; PACKET_LEN - 1];

        let err = decrypt_packet(&key, &packet).unwrap_err();

        assert!(matches!(
            err,
            DisgramsError::InvalidPacketLength(actual, expected)
                if actual == PACKET_LEN - 1 && expected == PACKET_LEN
        ));
    }

    #[test]
    fn decrypt_rejects_wrong_key() {
        let packet = encrypt_packet(
            &[7; 32],
            Header::new(42, 1337),
            [3; 12],
            Transaction::new(123, 45.5, TransactionType::Deposit),
        )
        .unwrap();

        let err = decrypt_packet(&[8; 32], &packet).unwrap_err();

        assert!(matches!(err, DisgramsError::DecryptionFailed));
    }
}
