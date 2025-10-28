use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use bincode::error::{DecodeError, EncodeError};
use bincode::serde::{decode_from_slice, encode_to_vec};
use solana_sdk::transaction::VersionedTransaction;
use thiserror::Error;

/// 解码 base64 编码的未签名交易。
pub fn decode_base64_transaction(encoded: &str) -> Result<VersionedTransaction, DecodeTxError> {
    let bytes = BASE64_STANDARD
        .decode(encoded.trim())
        .map_err(DecodeTxError::Base64)?;
    let (tx, _) = decode_from_slice::<VersionedTransaction, _>(&bytes, bincode_config())
        .map_err(DecodeTxError::Bincode)?;
    Ok(tx)
}

/// 将交易重新序列化为 base64 字符串，便于对比或落盘。
pub fn encode_base64_transaction(tx: &VersionedTransaction) -> Result<String, EncodeTxError> {
    let bytes = encode_to_vec(tx, bincode_config()).map_err(EncodeTxError::Bincode)?;
    Ok(BASE64_STANDARD.encode(bytes))
}

fn bincode_config() -> impl bincode::config::Config {
    bincode::config::standard()
        .with_fixed_int_encoding()
        .with_little_endian()
}

#[derive(Debug, Error)]
pub enum DecodeTxError {
    #[error("base64 解码失败: {0}")]
    Base64(base64::DecodeError),
    #[error("bincode 解码失败: {0}")]
    Bincode(DecodeError),
}

#[derive(Debug, Error)]
pub enum EncodeTxError {
    #[error("bincode 编码失败: {0}")]
    Bincode(EncodeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::instruction::Instruction;
    use solana_sdk::message::Message;
    use solana_sdk::message::VersionedMessage;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn roundtrip_legacy_transaction() {
        let payer = Pubkey::new_unique();
        let instruction = Instruction::new_with_bytes(Pubkey::new_unique(), &[1, 2, 3], vec![]);
        let message = Message::new(&[instruction], Some(&payer));
        let tx = VersionedTransaction {
            signatures: vec![],
            message: VersionedMessage::Legacy(message),
        };

        let encoded = encode_base64_transaction(&tx).expect("encode");
        let decoded = decode_base64_transaction(&encoded).expect("decode");
        assert_eq!(decoded.message.static_account_keys().len(), 1);
    }
}
