use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use sha2::{Digest, Sha256};

use super::ConfigError;

const ENC_FILE_NAME: &str = "wallet.enc";
const AES_KEY_ENV: &str = "GALILEO_AES_KEY";
const DEFAULT_KEY_SEED: &str = "galileo_fixed_key";
const NONCE_SIZE: usize = 12;

pub struct WalletProcessingResult {
    pub sanitized_config: bool,
}

pub fn process_wallet(
    wallet: &mut crate::config::WalletConfig,
    encrypted_path: &Path,
) -> Result<WalletProcessingResult, ConfigError> {
    let key = derive_key();
    let mut sanitized_config = false;

    let provided = wallet.private_key.trim();
    if !provided.is_empty() {
        let ciphertext = encrypt_wallet_key(provided.as_bytes(), &key).map_err(|message| {
            ConfigError::Parse {
                path: encrypted_path.to_path_buf(),
                message,
            }
        })?;
        write_encrypted_key(encrypted_path, &ciphertext).map_err(|source| ConfigError::Io {
            path: encrypted_path.to_path_buf(),
            source,
        })?;
        wallet.private_key = provided.to_string();
        sanitized_config = true;
    } else if encrypted_path.exists() {
        let decrypted =
            read_encrypted_key(encrypted_path, &key).map_err(|message| ConfigError::Parse {
                path: encrypted_path.to_path_buf(),
                message,
            })?;
        wallet.private_key = decrypted;
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

fn derive_key() -> [u8; 32] {
    let seed = std::env::var(AES_KEY_ENV).ok();
    let material = seed.unwrap_or_else(|| DEFAULT_KEY_SEED.to_string());
    let digest = Sha256::digest(material.as_bytes());
    digest.into()
}

fn encrypt_wallet_key(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|err| err.to_string())?;
    let mut nonce = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    let nonce_slice = aes_gcm::Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(nonce_slice, plaintext)
        .map_err(|err| err.to_string())?;

    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

fn read_encrypted_key(path: &Path, key: &[u8; 32]) -> Result<String, String> {
    let mut file = File::open(path).map_err(|err| err.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| err.to_string())?;

    if buffer.len() <= NONCE_SIZE {
        return Err("加密数据格式错误".to_string());
    }

    let (nonce_bytes, ciphertext) = buffer.split_at(NONCE_SIZE);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|err| err.to_string())?;
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|err| err.to_string())?;

    let decoded = String::from_utf8(plaintext).map_err(|err| err.to_string())?;
    Ok(decoded)
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
    use std::fs;
    use tempfile::tempdir;

    use super::*;
    use crate::config::WalletConfig;

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.previous {
                unsafe {
                    std::env::set_var(self.key, value);
                }
            } else {
                unsafe {
                    std::env::remove_var(self.key);
                }
            }
        }
    }

    #[test]
    fn process_wallet_encrypts_and_decrypts() {
        let _guard = EnvGuard::set(AES_KEY_ENV, "unit-test-key");

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

        let mut wallet_reload = WalletConfig::default();
        let reload_result =
            process_wallet(&mut wallet_reload, &enc_path).expect("reload wallet from enc file");
        assert!(!reload_result.sanitized_config);
        assert_eq!(wallet_reload.private_key, "SOME_PRIVATE_KEY");
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
