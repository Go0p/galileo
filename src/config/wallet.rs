use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use console::Term;
use tracing::{info, warn};
use zeroize::{Zeroize, Zeroizing};

use super::ConfigError;

const ENC_FILE_NAME: &str = "wallet.enc";
const MAGIC: &[u8; 8] = b"GLWALLET";
const FORMAT_VERSION: u8 = 1;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const PASSWORD_ENV: &str = "GALILEO_WALLET_PASSWORD";
const PASSWORD_NEW_ENV: &str = "GALILEO_WALLET_PASSWORD_NEW";
const MAX_PASSWORD_ATTEMPTS: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PasswordSource {
    Env,
    Interactive,
}

struct PasswordCandidate {
    value: Zeroizing<String>,
    source: PasswordSource,
}

impl PasswordCandidate {
    fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

pub struct WalletProcessingResult {
    pub sanitized_config: bool,
}

pub fn process_wallet(
    wallet: &mut crate::config::WalletConfig,
    encrypted_path: &Path,
) -> Result<WalletProcessingResult, ConfigError> {
    let mut sanitized_config = false;
    let provided = wallet.private_key.trim();

    if !provided.is_empty() {
        info!(
            target: "config",
            path = %encrypted_path.display(),
            "检测到配置中的私钥，准备加密写入 wallet.enc"
        );
        let password = obtain_new_password().map_err(|message| ConfigError::Parse {
            path: encrypted_path.to_path_buf(),
            message,
        })?;
        let encrypted =
            encrypt_wallet_key(provided.as_bytes(), password.as_ref()).map_err(|message| {
                ConfigError::Parse {
                    path: encrypted_path.to_path_buf(),
                    message,
                }
            })?;
        write_encrypted_key(encrypted_path, &encrypted).map_err(|source| ConfigError::Io {
            path: encrypted_path.to_path_buf(),
            source,
        })?;
        wallet.private_key = provided.to_string();
        sanitized_config = true;
    } else if encrypted_path.exists() {
        info!(
            target: "config",
            path = %encrypted_path.display(),
            "检测到 wallet.enc，需要输入密码解锁"
        );
        let mut attempts = 0usize;
        loop {
            attempts += 1;
            let candidate = obtain_existing_password().map_err(|message| ConfigError::Parse {
                path: encrypted_path.to_path_buf(),
                message,
            })?;

            let is_env = matches!(candidate.source, PasswordSource::Env);
            match decrypt_wallet_file(encrypted_path, candidate.as_str()) {
                Ok(decrypted) => {
                    wallet.private_key = decrypted;
                    break;
                }
                Err(message) => {
                    if is_env {
                        return Err(ConfigError::Parse {
                            path: encrypted_path.to_path_buf(),
                            message,
                        });
                    }
                    let remaining = MAX_PASSWORD_ATTEMPTS.saturating_sub(attempts);
                    warn!(
                        target: "config",
                        attempts,
                        remaining,
                        "钱包解密失败: {message}"
                    );
                    if remaining == 0 {
                        return Err(ConfigError::Parse {
                            path: encrypted_path.to_path_buf(),
                            message: format!("{message}（连续 {attempts} 次失败，已终止尝试）"),
                        });
                    }
                    println!("密码错误，还有 {} 次机会。", remaining);
                }
            }
        }
    }

    Ok(WalletProcessingResult { sanitized_config })
}

pub fn sanitize_config_file(config_path: &Path, original_value: &str) -> Result<bool, ConfigError> {
    let Ok(contents) = fs::read_to_string(config_path) else {
        return Ok(false);
    };

    let had_trailing_newline = contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    let mut replaced = false;

    for line in &mut lines {
        if replaced {
            break;
        }

        let trimmed_start = line.trim_start();
        if trimmed_start.starts_with('#') {
            continue;
        }

        let Some(rest) = trimmed_start.strip_prefix("private_key:") else {
            continue;
        };

        let indent_len = line.len() - trimmed_start.len();
        let indent = &line[..indent_len];
        let rest_trimmed = rest.trim_start();
        let (value_segment, comment_segment) = split_inline_comment(rest_trimmed);
        let value_without_trailing = value_segment.trim_end();
        let trailing_ws = &value_segment[value_without_trailing.len()..];
        let value_trimmed = value_without_trailing.trim_start();

        if let Some(stripped) = extract_quoted(value_trimmed) {
            if stripped == original_value {
                let comment_out = comment_segment
                    .map(|c| {
                        if trailing_ws.is_empty() {
                            format!(" {c}")
                        } else {
                            c.to_string()
                        }
                    })
                    .unwrap_or_default();
                *line = format!(
                    "{indent}private_key: \"\"{ws}{comment}",
                    indent = indent,
                    ws = trailing_ws,
                    comment = comment_out
                );
                replaced = true;
            }
        }
    }

    if replaced {
        let mut new_contents = lines.join("\n");
        if had_trailing_newline {
            new_contents.push('\n');
        }
        fs::write(config_path, new_contents).map_err(|source| ConfigError::Io {
            path: config_path.to_path_buf(),
            source,
        })?;
    }

    Ok(replaced)
}

pub(crate) fn encrypted_wallet_path(config_path: Option<&Path>) -> PathBuf {
    match config_path.and_then(|path| path.parent()) {
        Some(dir) => dir.join(ENC_FILE_NAME),
        None => PathBuf::from(ENC_FILE_NAME),
    }
}

fn obtain_new_password() -> Result<Zeroizing<String>, String> {
    if let Some(value) = env_password(PASSWORD_NEW_ENV) {
        return Ok(value);
    }
    if let Some(value) = env_password(PASSWORD_ENV) {
        return Ok(value);
    }

    prompt_new_password_interactive()
}

fn obtain_existing_password() -> Result<PasswordCandidate, String> {
    if let Some(value) = env_password(PASSWORD_ENV) {
        return Ok(PasswordCandidate {
            value,
            source: PasswordSource::Env,
        });
    }

    prompt_existing_password_interactive().map(|value| PasswordCandidate {
        value,
        source: PasswordSource::Interactive,
    })
}

fn env_password(name: &str) -> Option<Zeroizing<String>> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .map(Zeroizing::new)
}

fn prompt_new_password_interactive() -> Result<Zeroizing<String>, String> {
    let term = Term::stderr();
    if !term.is_term() {
        return Err("当前终端不支持交互式输入".to_string());
    }

    loop {
        let password = read_masked_password(&term, "🔐 设置钱包密码: ")?;
        if password.is_empty() {
            println!("密码不能为空，请重新输入。");
            continue;
        }

        let confirmation = read_masked_password(&term, "🔐 确认钱包密码: ")?;
        if password != confirmation {
            println!("两次输入的密码不一致，请重试。");
            continue;
        }

        return Ok(Zeroizing::new(password));
    }
}

fn prompt_existing_password_interactive() -> Result<Zeroizing<String>, String> {
    let term = Term::stderr();
    if !term.is_term() {
        return Err("当前终端不支持交互式输入".to_string());
    }

    let password = read_masked_password(&term, "🔓 请输入钱包密码: ")?;
    if password.is_empty() {
        return Err("钱包密码不能为空".to_string());
    }
    Ok(Zeroizing::new(password))
}

fn read_masked_password(term: &Term, prompt: &str) -> Result<String, String> {
    if let Err(err) = term.write_str(prompt) {
        return Err(format!("写入提示失败: {err}"));
    }
    if let Err(err) = term.flush() {
        return Err(format!("刷新输出失败: {err}"));
    }

    let mut buffer = String::new();
    loop {
        let ch = term
            .read_char()
            .map_err(|err| format!("读取输入失败: {err}"))?;

        match ch {
            '\n' | '\r' => {
                if let Err(err) = term.write_str("\n") {
                    return Err(format!("写入换行失败: {err}"));
                }
                if let Err(err) = term.flush() {
                    return Err(format!("刷新输出失败: {err}"));
                }
                break;
            }
            '\u{7f}' | '\u{8}' => {
                if !buffer.is_empty() {
                    buffer.pop();
                    if term.clear_chars(1).is_err() {
                        // 如果无法回退光标，退而求其次打印退格覆盖
                        let _ = term.write_str("\u{8} \u{8}");
                    }
                    let _ = term.flush();
                }
            }
            c if c.is_control() => {
                // 忽略其它控制字符
            }
            _ => {
                buffer.push(ch);
                if let Err(err) = term.write_str("*") {
                    return Err(format!("写入掩码失败: {err}"));
                }
                if let Err(err) = term.flush() {
                    return Err(format!("刷新输出失败: {err}"));
                }
            }
        }
    }

    Ok(buffer)
}

fn encrypt_wallet_key(plaintext: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);

    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);

    let mut encryption_key = derive_encryption_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&encryption_key)
        .map_err(|err| format!("初始化加密器失败: {err}"))?;

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|err| format!("加密钱包私钥失败: {err}"))?;

    encryption_key.zeroize();

    let mut data = Vec::with_capacity(MAGIC.len() + 1 + SALT_SIZE + NONCE_SIZE + ciphertext.len());
    data.extend_from_slice(MAGIC);
    data.push(FORMAT_VERSION);
    data.extend_from_slice(&salt);
    data.extend_from_slice(&nonce);
    data.extend_from_slice(&ciphertext);

    Ok(data)
}

fn decrypt_wallet_file(path: &Path, password: &str) -> Result<String, String> {
    let data = fs::read(path).map_err(|err| format!("读取钱包文件失败: {err}"))?;
    decrypt_wallet_bytes(&data, password)
}

fn decrypt_wallet_bytes(data: &[u8], password: &str) -> Result<String, String> {
    let header_len = MAGIC.len() + 1;
    if data.len() < header_len + SALT_SIZE + NONCE_SIZE {
        return Err("wallet.enc 文件格式错误".to_string());
    }

    let (magic, rest) = data.split_at(MAGIC.len());
    if magic != MAGIC {
        return Err("检测到旧版或未知格式的 wallet.enc，请删除该文件并重新导入私钥".to_string());
    }

    let version = rest[0];
    if version != FORMAT_VERSION {
        return Err(format!("不支持的 wallet.enc 版本: {version}"));
    }

    let rest = &rest[1..];

    let mut salt = [0u8; SALT_SIZE];
    salt.copy_from_slice(&rest[..SALT_SIZE]);

    let mut nonce = [0u8; NONCE_SIZE];
    nonce.copy_from_slice(&rest[SALT_SIZE..SALT_SIZE + NONCE_SIZE]);

    let ciphertext = &rest[SALT_SIZE + NONCE_SIZE..];

    let mut encryption_key = derive_encryption_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&encryption_key)
        .map_err(|err| format!("初始化解密器失败: {err}"))?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext)
        .map_err(|_| "钱包密码错误或数据已损坏".to_string())?;

    encryption_key.zeroize();

    String::from_utf8(plaintext).map_err(|err| format!("解析钱包私钥失败: {err}"))
}

fn derive_encryption_key(password: &str, salt: &[u8; SALT_SIZE]) -> Result<[u8; 32], String> {
    let params =
        Params::new(128 * 1024, 3, 4, Some(32)).map_err(|err| format!("Argon2 参数无效: {err}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|err| format!("派生加密密钥失败: {err}"))?;
    Ok(key)
}

fn write_encrypted_key(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = File::create(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        file.set_permissions(perms)?;
    }
    file.write_all(data)
}

fn split_inline_comment(value: &str) -> (&str, Option<&str>) {
    if let Some(pos) = value.find('#') {
        let (value_part, comment) = value.split_at(pos);
        (value_part, Some(comment))
    } else {
        (value, None)
    }
}

fn extract_quoted(value: &str) -> Option<&str> {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return Some(&value[1..value.len() - 1]);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    use crate::config::WalletConfig;

    fn with_password_env<F: FnOnce()>(password: &str, action: F) {
        unsafe {
            std::env::set_var(PASSWORD_NEW_ENV, password);
            std::env::set_var(PASSWORD_ENV, password);
        }
        action();
        unsafe {
            std::env::remove_var(PASSWORD_NEW_ENV);
            std::env::remove_var(PASSWORD_ENV);
        }
    }

    #[test]
    fn process_wallet_encrypts_and_decrypts() {
        with_password_env("test-password-123!", || {
            let dir = tempdir().expect("temp dir");
            let config_path = dir.path().join("galileo.yaml");
            fs::write(&config_path, sample_config("SOME_PRIVATE_KEY")).expect("write config");

            let mut wallet = WalletConfig {
                private_key: "SOME_PRIVATE_KEY".to_string(),
                ..Default::default()
            };
            let enc_path = encrypted_wallet_path(Some(config_path.as_path()));

            let result = process_wallet(&mut wallet, &enc_path).expect("process wallet");
            assert!(result.sanitized_config);
            assert_eq!(wallet.private_key, "SOME_PRIVATE_KEY");
            assert!(enc_path.exists());

            unsafe {
                std::env::remove_var(PASSWORD_NEW_ENV);
            }

            let mut wallet_reload = WalletConfig::default();
            let reload_result =
                process_wallet(&mut wallet_reload, &enc_path).expect("reload wallet from enc file");
            assert!(!reload_result.sanitized_config);
            assert_eq!(wallet_reload.private_key, "SOME_PRIVATE_KEY");
        });
    }

    #[test]
    fn sanitize_config_file_clears_private_key_with_comment() {
        let dir = tempdir().expect("temp dir");
        let config_path = dir.path().join("galileo.yaml");
        fs::write(
            &config_path,
            "global:\n  wallet:\n    private_key: \"SOME_PRIVATE_KEY\" # comment\n",
        )
        .expect("write config");

        let changed =
            sanitize_config_file(&config_path, "SOME_PRIVATE_KEY").expect("sanitize config");
        assert!(changed);
        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(updated.contains("private_key: \"\" # comment"));
        assert!(updated.ends_with('\n'));
    }

    #[test]
    fn sanitize_config_file_clears_private_key_without_comment() {
        let dir = tempdir().expect("temp dir");
        let config_path = dir.path().join("galileo.yaml");
        fs::write(
            &config_path,
            "global:\n  wallet:\n    private_key: \"SOME_PRIVATE_KEY\"\n",
        )
        .expect("write config");

        let changed =
            sanitize_config_file(&config_path, "SOME_PRIVATE_KEY").expect("sanitize config");
        assert!(changed);
        let updated = fs::read_to_string(&config_path).expect("read config");
        assert!(updated.contains("private_key: \"\""));
    }

    fn sample_config(key: &str) -> String {
        format!(
            "global:\n  wallet:\n    private_key: \"{key}\"\n",
            key = key
        )
    }
}
