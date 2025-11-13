use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use tracing::{info, warn};

use crate::config::{
    IntermediumConfig, JsonMintSource, JsonSelectorType, MintFileSource, MintFileSourceKind,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MintSourceError {
    #[error("读取 {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("解析 JSON {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{path}: {message}")]
    Invalid { path: PathBuf, message: String },
}

pub fn hydrate_mints_from_sources(
    cfg: &mut IntermediumConfig,
    base_dir: Option<&Path>,
) -> Result<(), MintSourceError> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for mint in std::mem::take(&mut cfg.mints) {
        let trimmed = mint.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            merged.push(trimmed.to_string());
        }
    }

    let mut total_loaded = 0usize;
    if !cfg.load_mints_from_files.is_empty() {
        for source in &cfg.load_mints_from_files {
            let (path, label, mints) = match source {
                MintFileSource::TextPath(path) => load_text_source(path, None, base_dir)
                    .map(|(path, mints)| (path, None, mints))?,
                MintFileSource::Detailed(kind) => match kind {
                    MintFileSourceKind::Text(text) => {
                        let name = text.name.clone();
                        let (path, mints) =
                            load_text_source(&text.path, text.name.as_deref(), base_dir)?;
                        (path, name, mints)
                    }
                    MintFileSourceKind::Json(json) => {
                        let name = json.name.clone();
                        let (path, mints) = load_json_source(json, base_dir)?;
                        (path, name, mints)
                    }
                },
            };

            if mints.is_empty() {
                warn!(
                    target: "intermedium::loader",
                    path = %path.display(),
                    name = label.as_deref().unwrap_or(""),
                    "未从该源读取到 mint，检查 selector 配置是否正确"
                );
                continue;
            }

            let mut added = 0usize;
            for mint in mints {
                if seen.insert(mint.clone()) {
                    merged.push(mint);
                    added += 1;
                    total_loaded += 1;
                }
            }

            info!(
                target: "intermedium::loader",
                path = %path.display(),
                name = label.as_deref().unwrap_or(""),
                added,
                "load_mints_from_files 已加载条目"
            );
        }
    }

    if total_loaded > 0 {
        info!(
            target: "intermedium::loader",
            total = total_loaded,
            "intermedium mints 已追加来自文件的条目"
        );
    }

    apply_limit(cfg.max_tokens_limit, &mut merged);

    cfg.mints = merged;
    Ok(())
}

fn apply_limit(limit: u32, mints: &mut Vec<String>) {
    let limit = limit as usize;
    if limit == 0 {
        return;
    }
    if mints.len() > limit {
        let dropped = mints.len() - limit;
        mints.truncate(limit);
        info!(
            target: "intermedium::loader",
            limit,
            dropped,
            "intermedium mints 已根据 max_tokens_limit 截断"
        );
    }
}

fn load_text_source(
    raw_path: &str,
    name: Option<&str>,
    base_dir: Option<&Path>,
) -> Result<(PathBuf, Vec<String>), MintSourceError> {
    let resolved = resolve_path(raw_path, base_dir);
    let content = fs::read_to_string(&resolved).map_err(|source| MintSourceError::Io {
        path: resolved.clone(),
        source,
    })?;

    let mints = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    info!(
        target: "intermedium::loader",
        path = %resolved.display(),
        name = name.unwrap_or(""),
        count = mints.len(),
        "text 源读取完成"
    );

    Ok((resolved, mints))
}

fn load_json_source(
    spec: &JsonMintSource,
    base_dir: Option<&Path>,
) -> Result<(PathBuf, Vec<String>), MintSourceError> {
    let resolved = resolve_path(&spec.path, base_dir);
    let raw = fs::read_to_string(&resolved).map_err(|source| MintSourceError::Io {
        path: resolved.clone(),
        source,
    })?;

    let value: Value = serde_json::from_str(&raw).map_err(|source| MintSourceError::Json {
        path: resolved.clone(),
        source,
    })?;

    let nodes = if let Some(selector) = spec.selector.as_ref() {
        let expr = selector.expr.trim();
        if expr.is_empty() {
            return Err(MintSourceError::Invalid {
                path: resolved.clone(),
                message: "selector.expr 不能为空".to_string(),
            });
        }
        match selector.selector_type {
            JsonSelectorType::JsonPath => {
                let ops = parse_json_path(expr).map_err(|message| MintSourceError::Invalid {
                    path: resolved.clone(),
                    message,
                })?;
                apply_json_path(&value, &ops)
            }
            JsonSelectorType::JsonPointer => {
                if expr == "/" {
                    vec![&value]
                } else {
                    value.pointer(expr).map(|node| vec![node]).ok_or_else(|| {
                        MintSourceError::Invalid {
                            path: resolved.clone(),
                            message: format!("pointer {expr} 未命中任何节点"),
                        }
                    })?
                }
            }
        }
    } else {
        vec![&value]
    };

    let mints = collect_scalar_strings(&nodes, &resolved)?;

    Ok((resolved, mints))
}

fn collect_scalar_strings(nodes: &[&Value], path: &Path) -> Result<Vec<String>, MintSourceError> {
    let mut result = Vec::new();
    for node in nodes {
        push_scalar(node, &mut result, path)?;
    }
    Ok(result)
}

fn push_scalar(node: &Value, output: &mut Vec<String>, path: &Path) -> Result<(), MintSourceError> {
    match node {
        Value::String(value) => {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                output.push(trimmed.to_string());
            }
            Ok(())
        }
        Value::Array(items) => {
            for item in items {
                push_scalar(item, output, path)?;
            }
            Ok(())
        }
        Value::Number(num) => {
            output.push(num.to_string());
            Ok(())
        }
        Value::Bool(flag) => {
            output.push(flag.to_string());
            Ok(())
        }
        Value::Null => Ok(()),
        Value::Object(_) => Err(MintSourceError::Invalid {
            path: path.to_path_buf(),
            message: "selector 命中对象，请直接指向包含 mint 的字段".to_string(),
        }),
    }
}

fn resolve_path(raw: &str, base_dir: Option<&Path>) -> PathBuf {
    let trimmed = raw.trim();
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        path
    } else if let Some(base) = base_dir {
        base.join(path)
    } else {
        path
    }
}

#[derive(Debug, Clone)]
enum JsonPathOp {
    Field(String),
    Index(usize),
    Wildcard,
}

fn parse_json_path(expr: &str) -> Result<Vec<JsonPathOp>, String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err("json_path 不能为空".to_string());
    }
    let mut chars = trimmed.chars().peekable();
    match chars.next() {
        Some('$') => {}
        _ => return Err("json_path 需要以 $ 开头".to_string()),
    }

    let mut ops = Vec::new();
    while let Some(&ch) = chars.peek() {
        match ch {
            '.' => {
                chars.next();
                let mut field = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '.' || c == '[' {
                        break;
                    }
                    field.push(c);
                    chars.next();
                }

                if field.is_empty() {
                    return Err("字段名不能为空".to_string());
                }
                ops.push(JsonPathOp::Field(field));
            }
            '[' => {
                chars.next();
                let mut buf = String::new();
                let mut closed = false;
                while let Some(c) = chars.next() {
                    if c == ']' {
                        closed = true;
                        break;
                    }
                    buf.push(c);
                }
                if !closed {
                    return Err("缺少 ]".to_string());
                }

                let selector = buf.trim();
                if selector == "*" {
                    ops.push(JsonPathOp::Wildcard);
                } else if let Ok(index) = selector.parse::<usize>() {
                    ops.push(JsonPathOp::Index(index));
                } else {
                    return Err(format!("不支持的 selector [{selector}]"));
                }
            }
            ' ' | '\n' | '\t' | '\r' => {
                chars.next();
            }
            _ => {
                return Err(format!("无法解析字符 {ch}"));
            }
        }
    }

    Ok(ops)
}

fn apply_json_path<'a>(root: &'a Value, ops: &[JsonPathOp]) -> Vec<&'a Value> {
    let mut current = vec![root];
    for op in ops {
        let mut next = Vec::new();
        match op {
            JsonPathOp::Field(name) => {
                for node in &current {
                    if let Value::Object(map) = node {
                        if let Some(value) = map.get(name) {
                            next.push(value);
                        }
                    }
                }
            }
            JsonPathOp::Index(idx) => {
                for node in &current {
                    if let Value::Array(items) = node {
                        if let Some(value) = items.get(*idx) {
                            next.push(value);
                        }
                    }
                }
            }
            JsonPathOp::Wildcard => {
                for node in &current {
                    match node {
                        Value::Array(items) => next.extend(items.iter()),
                        Value::Object(map) => next.extend(map.values()),
                        _ => {}
                    }
                }
            }
        }
        current = next;
    }
    current
}
