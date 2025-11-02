use solana_sdk::pubkey::Pubkey;

/// Lighthouse 主网程序 ID。
pub const LIGHTHOUSE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    4, 223, 173, 121, 98, 255, 177, 221, 146, 93, 10, 159, 181, 230, 208, 12, 230, 25, 91, 168,
    187, 58, 145, 253, 7, 239, 152, 96, 197, 233, 123, 184,
]);

/// SPL Token 账户中 `amount` 字段的偏移（单位：字节）。
#[allow(dead_code)]
pub const TOKEN_ACCOUNT_AMOUNT_OFFSET: u16 = 64;
/// SPL Token 账户中 `amount` 字段长度（单位：字节）。
#[allow(dead_code)]
pub const TOKEN_ACCOUNT_AMOUNT_SIZE: u16 = 8;

/// system program ID（写成常量便于在测试环境中序列化）。
pub const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0u8; 32]);

/// Lighthouse 日志等级枚举。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum LogLevel {
    Silent = 0,
    PlaintextMessage = 1,
    EncodedMessage = 2,
    EncodedNoop = 3,
    FailedPlaintextMessage = 4,
    FailedEncodedMessage = 5,
    FailedEncodedNoop = 6,
}

/// Lighthouse 整数比较运算符。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum IntegerOperator {
    Equal = 0,
    NotEqual = 1,
    GreaterThan = 2,
    LessThan = 3,
    GreaterThanOrEqual = 4,
    LessThanOrEqual = 5,
    Contains = 6,
    DoesNotContain = 7,
}

pub(crate) fn push_compact_u64(buffer: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buffer.push(byte);
        if value == 0 {
            break;
        }
    }
}
