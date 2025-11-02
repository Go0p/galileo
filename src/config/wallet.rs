use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine as _, engine::general_purpose};
use bs58;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
    terminal,
};
use serde_json;
use solana_sdk::signature::Keypair;
use tracing::{info, warn};
use zeroize::{Zeroize, Zeroizing};

use super::ConfigError;
use crate::config::{GalileoConfig, WalletKeyEntry};

const MAGIC: &[u8; 8] = b"GLWALLET";
const FORMAT_VERSION: u8 = 1;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const MAX_PASSWORD_ATTEMPTS: usize = 3;

pub struct WalletProcessingResult {
    pub config_updated: bool,
    pub selected_remark: Option<String>,
}

pub fn process_wallet_keys(
    config: &mut GalileoConfig,
    config_path: Option<&Path>,
) -> Result<WalletProcessingResult, ConfigError> {
    if config.wallet_keys.is_empty() {
        info!(target: "config", "wallet_keys 为空，进入私钥录入流程");
        return interactive_add_wallet_entry_to_config(config, config_path);
    }

    let selected =
        select_wallet_entry(&config.wallet_keys).map_err(|message| ConfigError::Parse {
            path: config_path
                .map(Path::to_path_buf)
                .unwrap_or_else(default_wallet_path),
            message,
        })?;

    let encoded = config.wallet_keys[selected].encrypted.trim();
    let cipher = general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map_err(|err| ConfigError::Parse {
            path: config_path
                .map(Path::to_path_buf)
                .unwrap_or_else(default_wallet_path),
            message: format!("wallet_keys[{selected}] Base64 解码失败: {err}"),
        })?;

    let mut attempts = 0usize;
    loop {
        attempts += 1;
        let password = obtain_existing_password().map_err(|message| ConfigError::Parse {
            path: config_path
                .map(Path::to_path_buf)
                .unwrap_or_else(default_wallet_path),
            message,
        })?;

        match decrypt_wallet_bytes(&cipher, password.as_ref()) {
            Ok(decrypted) => {
                config.private_key = decrypted;
                break;
            }
            Err(message) => {
                let remaining = MAX_PASSWORD_ATTEMPTS.saturating_sub(attempts);
                warn!(
                    target: "config",
                    attempts,
                    remaining,
                    remark = %config.wallet_keys[selected].remark,
                    "钱包解密失败: {message}"
                );
                if remaining == 0 {
                    return Err(ConfigError::Parse {
                        path: config_path
                            .map(Path::to_path_buf)
                            .unwrap_or_else(default_wallet_path),
                        message: format!("{message}（连续 {attempts} 次失败，已终止尝试）"),
                    });
                }
                println!("密码错误，还有 {} 次机会。", remaining);
            }
        }
    }

    Ok(WalletProcessingResult {
        config_updated: false,
        selected_remark: Some(config.wallet_keys[selected].remark.clone()),
    })
}

fn interactive_add_wallet_entry_to_config(
    config: &mut GalileoConfig,
    config_path: Option<&Path>,
) -> Result<WalletProcessingResult, ConfigError> {
    let private_key = prompt_private_key_segments().map_err(|message| ConfigError::Parse {
        path: config_path
            .map(Path::to_path_buf)
            .unwrap_or_else(default_wallet_path),
        message,
    })?;
    let remark = prompt_wallet_remark().map_err(|message| ConfigError::Parse {
        path: config_path
            .map(Path::to_path_buf)
            .unwrap_or_else(default_wallet_path),
        message,
    })?;
    if config
        .wallet_keys
        .iter()
        .any(|entry| entry.remark.eq_ignore_ascii_case(&remark))
    {
        return Err(ConfigError::Parse {
            path: config_path
                .map(Path::to_path_buf)
                .unwrap_or_else(default_wallet_path),
            message: format!("备注名 \"{remark}\" 已存在，请使用其它名称"),
        });
    }
    let password = obtain_new_password().map_err(|message| ConfigError::Parse {
        path: config_path
            .map(Path::to_path_buf)
            .unwrap_or_else(default_wallet_path),
        message,
    })?;
    let encrypted =
        encrypt_wallet_key(private_key.as_bytes(), password.as_ref()).map_err(|message| {
            ConfigError::Parse {
                path: config_path
                    .map(Path::to_path_buf)
                    .unwrap_or_else(default_wallet_path),
                message,
            }
        })?;
    let encoded = general_purpose::STANDARD.encode(encrypted);

    config.private_key = private_key.clone();
    config.wallet_keys.push(WalletKeyEntry {
        remark: remark.clone(),
        encrypted: encoded.clone(),
    });

    if let Some(path) = config_path {
        persist_wallet_keys(path, &config.wallet_keys)?;
        info!(
            target: "config",
            path = %path.display(),
            "已在配置中写入加密后的 wallet_keys 条目 \"{remark}\""
        );
    }

    Ok(WalletProcessingResult {
        config_updated: true,
        selected_remark: Some(remark),
    })
}

// 用于 CLI wallet add 命令的辅助函数
pub fn add_wallet_to_config(
    config: &mut GalileoConfig,
    config_path: Option<&Path>,
) -> Result<WalletProcessingResult, ConfigError> {
    interactive_add_wallet_entry_to_config(config, config_path)
}

pub fn parse_keypair_string(raw: &str) -> Result<Keypair, anyhow::Error> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("keypair string empty");
    }

    if trimmed.starts_with('[') {
        let bytes: Vec<u8> = serde_json::from_str(trimmed)?;
        Ok(Keypair::try_from(bytes.as_slice())?)
    } else if trimmed.contains(',') {
        let bytes = trimmed
            .split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Keypair::try_from(bytes.as_slice())?)
    } else {
        let data = bs58::decode(trimmed).into_vec()?;
        Ok(Keypair::try_from(data.as_slice())?)
    }
}

fn persist_wallet_keys(path: &Path, entries: &[WalletKeyEntry]) -> Result<(), ConfigError> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let had_trailing_newline = contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    let formatted = format_wallet_keys_lines(entries, path)?;

    let mut start_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("wallet_keys:") {
            start_idx = Some(idx);
            break;
        }
    }

    if let Some(start) = start_idx {
        let mut end = start + 1;
        while end < lines.len() {
            let current = &lines[end];
            if current.trim().is_empty() {
                end += 1;
                continue;
            }
            if !current.starts_with(' ') && !current.starts_with('\t') {
                break;
            }
            end += 1;
        }
        lines.splice(start..end, formatted.clone());
    } else {
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }
        lines.extend(formatted.clone());
    }

    let mut output = lines.join("\n");
    if had_trailing_newline || output.is_empty() {
        output.push('\n');
    } else {
        output.push('\n');
    }

    fs::write(path, output).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn format_wallet_keys_lines(
    entries: &[WalletKeyEntry],
    path: &Path,
) -> Result<Vec<String>, ConfigError> {
    let mut lines = Vec::new();
    lines.push("wallet_keys:".to_string());

    if entries.is_empty() {
        lines.push("  # 尚未配置加密私钥，启动时会提示录入。".to_string());
    } else {
        let serialized = serde_yaml::to_string(entries).map_err(|err| ConfigError::Parse {
            path: path.to_path_buf(),
            message: format!("序列化 wallet_keys 失败: {err}"),
        })?;
        for line in serialized.trim_end_matches('\n').lines() {
            lines.push(format!("  {}", line));
        }
    }

    Ok(lines)
}

fn prompt_private_key_segments() -> Result<String, String> {
    println!(
        "请输入三段私钥内容，格式为 <内容>:<顺序>，例如 xxxxx:1。顺序号 1、2、3 可按任意顺序输入。"
    );
    let mut segments: Vec<(usize, String)> = Vec::new();
    let mut looks_like_json = false;

    while segments.len() < 3 {
        let prompt = format!("第 {} 段: ", segments.len() + 1);
        let input = prompt_line(&prompt)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("输入不能为空，请重新输入。");
            continue;
        }

        match parse_segment(trimmed) {
            Ok((order, value)) => {
                if value.contains(',') || value.contains('[') || value.contains(']') {
                    looks_like_json = true;
                }
                if order == 0 || order > 3 {
                    println!("顺序号必须在 1~3 之间。");
                    continue;
                }
                if segments.iter().any(|(existing, _)| *existing == order) {
                    println!("顺序号 {order} 已录入，请输入不同的顺序。");
                    continue;
                }
                segments.push((order, value));
            }
            Err(message) => println!("{message}"),
        }
    }

    segments.sort_by_key(|(order, _)| *order);
    let mut combined = String::new();
    if looks_like_json {
        let mut prepend_bracket = false;
        let mut append_bracket = false;

        for (idx, (_, value)) in segments.iter().enumerate() {
            let mut piece = value.trim();
            if idx == 0 && piece.starts_with('[') {
                prepend_bracket = true;
                piece = piece.trim_start_matches('[');
            }
            if idx + 1 == segments.len() && piece.ends_with(']') {
                append_bracket = true;
                piece = piece.trim_end_matches(']');
            }

            let cleaned = piece.trim();
            if cleaned.is_empty() {
                continue;
            }

            let cleaned = cleaned.trim_start_matches(',');
            if combined.is_empty() {
                combined.push_str(cleaned);
            } else {
                if !combined.ends_with(',') && !cleaned.starts_with(',') {
                    combined.push(',');
                }
                combined.push_str(cleaned);
            }
        }

        let combined = combined.trim_matches(',');
        let reconstructed = if prepend_bracket || append_bracket || looks_like_json {
            format!("[{}]", combined)
        } else {
            combined.to_string()
        };
        Ok(reconstructed)
    } else {
        for (_, value) in segments {
            combined.push_str(value.trim());
        }
        Ok(combined.trim().to_string())
    }
}

fn parse_segment(raw: &str) -> Result<(usize, String), String> {
    let (value, order_str) = raw
        .rsplit_once(':')
        .ok_or_else(|| "格式错误，应为 <内容>:<顺序>".to_string())?;

    let order = order_str
        .trim()
        .parse::<usize>()
        .map_err(|_| "顺序号必须是数字".to_string())?;
    let segment = value.trim();
    if segment.is_empty() {
        return Err("私钥内容不能为空".to_string());
    }

    Ok((order, segment.to_string()))
}

fn prompt_wallet_remark() -> Result<String, String> {
    loop {
        let remark = prompt_line("请输入该私钥的备注名: ")?;
        let trimmed = remark.trim();
        if trimmed.is_empty() {
            println!("备注名不能为空，请重新输入。");
            continue;
        }
        return Ok(trimmed.to_string());
    }
}

fn select_wallet_entry(entries: &[WalletKeyEntry]) -> Result<usize, String> {
    if entries.is_empty() {
        return Err("wallet_keys 列表为空".to_string());
    }

    if entries.len() == 1 {
        println!("检测到唯一加密私钥，默认使用 \"{}\"。", entries[0].remark);
        return Ok(0);
    }

    // 尝试使用交互式选择
    if io::stderr().is_terminal() {
        return interactive_select_wallet_entry(entries);
    }

    println!("检测到多个加密私钥，请输入序号：");
    for (idx, entry) in entries.iter().enumerate() {
        println!("  [{}] {}", idx + 1, entry.remark);
    }

    loop {
        let input = prompt_line("请输入序号: ")?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("序号不能为空，请重试。");
            continue;
        }

        match trimmed.parse::<usize>() {
            Ok(value) if value >= 1 && value <= entries.len() => return Ok(value - 1),
            _ => println!("无效的序号，请输入 1-{} 之间的数字。", entries.len()),
        }
    }
}

fn interactive_select_wallet_entry(entries: &[WalletKeyEntry]) -> Result<usize, String> {
    let mut current = 0usize;
    let mut typed = String::new();
    let mut stderr = io::stderr();

    // 启用 raw mode
    terminal::enable_raw_mode().map_err(|err| format!("启用终端原始模式失败: {err}"))?;

    // 渲染和循环
    let result = select_wallet_loop(&mut stderr, entries, &mut current, &mut typed);

    // 恢复终端
    let _ = terminal::disable_raw_mode();

    result
}

fn select_wallet_loop(
    stderr: &mut io::Stderr,
    entries: &[WalletKeyEntry],
    current: &mut usize,
    typed: &mut String,
) -> Result<usize, String> {
    // 初始渲染
    render_wallet_menu(stderr, entries, *current, typed)?;

    loop {
        let evt = event::read().map_err(|err| format!("读取终端输入失败: {err}"))?;

        match evt {
            Event::Key(key_event) => {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }
                match key_event.code {
                    KeyCode::Up => {
                        typed.clear();
                        if *current == 0 {
                            *current = entries.len() - 1;
                        } else {
                            *current -= 1;
                        }
                        clear_and_redraw(stderr, entries, *current, typed)?;
                    }
                    KeyCode::Down => {
                        typed.clear();
                        *current = (*current + 1) % entries.len();
                        clear_and_redraw(stderr, entries, *current, typed)?;
                    }
                    KeyCode::Char('k') | KeyCode::Char('K') => {
                        typed.clear();
                        if *current == 0 {
                            *current = entries.len() - 1;
                        } else {
                            *current -= 1;
                        }
                        clear_and_redraw(stderr, entries, *current, typed)?;
                    }
                    KeyCode::Char('j') | KeyCode::Char('J') => {
                        typed.clear();
                        *current = (*current + 1) % entries.len();
                        clear_and_redraw(stderr, entries, *current, typed)?;
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        typed.push(c);
                        if let Ok(value) = typed.parse::<usize>() {
                            if value >= 1 && value <= entries.len() {
                                *current = value - 1;
                            }
                        }
                        clear_and_redraw(stderr, entries, *current, typed)?;
                    }
                    KeyCode::Backspace => {
                        typed.pop();
                        clear_and_redraw(stderr, entries, *current, typed)?;
                    }
                    KeyCode::Enter => {
                        let chosen = if let Ok(value) = typed.parse::<usize>() {
                            if value >= 1 && value <= entries.len() {
                                value - 1
                            } else {
                                *current
                            }
                        } else {
                            *current
                        };
                        // 清除菜单
                        clear_menu(stderr, entries.len() + 3)?;
                        return Ok(chosen);
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        clear_menu(stderr, entries.len() + 3)?;
                        return Err("已取消钱包选择".to_string());
                    }
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        clear_menu(stderr, entries.len() + 3)?;
                        return Err("用户取消选择".to_string());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn clear_and_redraw(
    stderr: &mut io::Stderr,
    entries: &[WalletKeyEntry],
    current: usize,
    typed: &str,
) -> Result<(), String> {
    clear_menu(stderr, entries.len() + 3)?;
    render_wallet_menu(stderr, entries, current, typed)
}

fn clear_menu(stderr: &mut io::Stderr, lines: usize) -> Result<(), String> {
    for _ in 0..lines {
        execute!(
            stderr,
            cursor::MoveUp(1),
            cursor::MoveToColumn(0),
            terminal::Clear(terminal::ClearType::CurrentLine)
        )
        .map_err(|err| format!("清除菜单失败: {err}"))?;
    }
    Ok(())
}

fn render_wallet_menu(
    stderr: &mut io::Stderr,
    entries: &[WalletKeyEntry],
    current: usize,
    typed: &str,
) -> Result<(), String> {
    execute!(
        stderr,
        Print("请选择要解锁的钱包（↑/↓ 切换，回车确认）：\n")
    )
    .map_err(|err| format!("终端输出失败: {err}"))?;

    for (idx, entry) in entries.iter().enumerate() {
        if idx == current {
            execute!(
                stderr,
                Print(format!("  ➤ [{}] {}\n", idx + 1, entry.remark))
            )
            .map_err(|err| format!("终端输出失败: {err}"))?;
        } else {
            execute!(
                stderr,
                Print(format!("    [{}] {}\n", idx + 1, entry.remark))
            )
            .map_err(|err| format!("终端输出失败: {err}"))?;
        }
    }

    if typed.is_empty() {
        execute!(stderr, Print("  （也可直接输入序号并回车确认）\n"))
            .map_err(|err| format!("终端输出失败: {err}"))?;
    } else {
        execute!(stderr, Print(format!("  当前输入序号: {}\n", typed)))
            .map_err(|err| format!("终端输出失败: {err}"))?;
    }

    Ok(())
}

fn prompt_line(prompt: &str) -> Result<String, String> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|err| format!("刷新输出失败: {err}"))?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| format!("读取输入失败: {err}"))?;
    Ok(input.trim_end_matches(&['\r', '\n'][..]).to_string())
}

fn default_wallet_path() -> PathBuf {
    PathBuf::from("<config>")
}

fn obtain_new_password() -> Result<Zeroizing<String>, String> {
    prompt_new_password_interactive()
}

fn obtain_existing_password() -> Result<Zeroizing<String>, String> {
    prompt_existing_password_interactive()
}

fn prompt_new_password_interactive() -> Result<Zeroizing<String>, String> {
    // 检查是否在终端环境
    if !terminal::is_raw_mode_enabled().is_ok() && !io::stderr().is_terminal() {
        return Err("当前终端不支持交互式输入".to_string());
    }

    loop {
        let password = read_masked_password("🔐 设置钱包密码: ")?;
        if password.is_empty() {
            println!("密码不能为空，请重新输入。");
            continue;
        }

        let confirmation = read_masked_password("🔐 确认钱包密码: ")?;
        if password != confirmation {
            println!("两次输入的密码不一致，请重试。");
            continue;
        }

        return Ok(Zeroizing::new(password));
    }
}

fn prompt_existing_password_interactive() -> Result<Zeroizing<String>, String> {
    // 检查是否在终端环境
    if !terminal::is_raw_mode_enabled().is_ok() && !io::stderr().is_terminal() {
        return Err("当前终端不支持交互式输入".to_string());
    }

    let password = read_masked_password("🔓 请输入钱包密码: ")?;
    if password.is_empty() {
        return Err("钱包密码不能为空".to_string());
    }
    Ok(Zeroizing::new(password))
}

fn read_masked_password(prompt: &str) -> Result<String, String> {
    let mut stderr = io::stderr();

    // 输出提示
    execute!(stderr, Print(prompt)).map_err(|err| format!("写入提示失败: {err}"))?;

    // 保存提示文本后的光标位置（输入区域起点）
    execute!(stderr, cursor::SavePosition).map_err(|err| format!("保存光标位置失败: {err}"))?;

    // 启用 raw mode 以捕获键盘事件
    terminal::enable_raw_mode().map_err(|err| format!("启用终端原始模式失败: {err}"))?;

    let result = read_password_input(&mut stderr);

    // 恢复终端状态
    let _ = terminal::disable_raw_mode();

    // 输出换行
    let _ = execute!(stderr, Print("\n"));

    result
}

fn read_password_input(stderr: &mut io::Stderr) -> Result<String, String> {
    let mut buffer = String::new();
    let mut cursor_pos = 0usize; // 当前光标在 buffer 中的位置

    loop {
        // 读取事件
        let evt = event::read().map_err(|err| format!("读取输入失败: {err}"))?;

        match evt {
            Event::Key(key_event) => {
                // 忽略按键释放事件，保留按下与重复事件
                if key_event.kind == KeyEventKind::Release {
                    continue;
                }
                match key_event.code {
                    KeyCode::Enter => {
                        break;
                    }
                    KeyCode::Backspace => {
                        if cursor_pos > 0 {
                            buffer.remove(cursor_pos - 1);
                            cursor_pos -= 1;
                            redraw_masked_input(stderr, &buffer, cursor_pos)?;
                        }
                    }
                    KeyCode::Delete => {
                        if cursor_pos < buffer.len() {
                            buffer.remove(cursor_pos);
                            redraw_masked_input(stderr, &buffer, cursor_pos)?;
                        }
                    }
                    KeyCode::Left => {
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                            let _ = execute!(stderr, cursor::MoveLeft(1));
                        }
                    }
                    KeyCode::Right => {
                        if cursor_pos < buffer.len() {
                            cursor_pos += 1;
                            let _ = execute!(stderr, cursor::MoveRight(1));
                        }
                    }
                    KeyCode::Home => {
                        if cursor_pos > 0 {
                            let _ = execute!(stderr, cursor::MoveLeft(cursor_pos as u16));
                            cursor_pos = 0;
                        }
                    }
                    KeyCode::End => {
                        if cursor_pos < buffer.len() {
                            let move_right = buffer.len() - cursor_pos;
                            let _ = execute!(stderr, cursor::MoveRight(move_right as u16));
                            cursor_pos = buffer.len();
                        }
                    }
                    KeyCode::Char(c) => {
                        // Ctrl+C / Ctrl+D 退出；其余 Ctrl 组合直接忽略
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                            let lower = c.to_ascii_lowercase();
                            if lower == 'c' || lower == 'd' {
                                return Err("用户取消输入".to_string());
                            }
                            continue;
                        }
                        buffer.insert(cursor_pos, c);
                        cursor_pos += 1;
                        redraw_masked_input(stderr, &buffer, cursor_pos)?;
                    }
                    _ => {
                        // 忽略其他按键
                    }
                }
            }
            _ => {
                // 忽略非键盘事件
            }
        }
    }

    Ok(buffer)
}

fn redraw_masked_input(
    stderr: &mut io::Stderr,
    buffer: &str,
    cursor_pos: usize,
) -> Result<(), String> {
    // 使用保存的光标位置（输入区域起点）来重绘：
    // 1. 恢复到输入区域起点
    // 2. 清除到行尾
    // 3. 输出掩码
    // 4. 再次回到输入起点
    // 5. 移动光标到正确位置
    let masked = "●".repeat(buffer.len());

    execute!(
        stderr,
        cursor::RestorePosition,
        terminal::Clear(terminal::ClearType::UntilNewLine),
        Print(&masked),
        cursor::RestorePosition,
        cursor::MoveRight(cursor_pos as u16)
    )
    .map_err(|err| format!("重绘输入失败: {err}"))?;

    Ok(())
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

fn decrypt_wallet_bytes(data: &[u8], password: &str) -> Result<String, String> {
    let header_len = MAGIC.len() + 1;
    if data.len() < header_len + SALT_SIZE + NONCE_SIZE {
        return Err("wallet_keys 数据格式错误".to_string());
    }

    let (magic, rest) = data.split_at(MAGIC.len());
    if magic != MAGIC {
        return Err("检测到旧版或未知格式的 wallet 密文，请重新录入私钥".to_string());
    }

    let version = rest[0];
    if version != FORMAT_VERSION {
        return Err(format!("不支持的 wallet 密文版本: {version}"));
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
