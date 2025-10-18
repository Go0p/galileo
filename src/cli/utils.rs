use crate::api::QuoteRequest;
use crate::config::JupiterQuoteConfig;

/// 解析 `key=value` 形式的额外查询参数。
pub fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| "参数格式需为 key=value".to_string())?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// 根据配置补全报价默认值，避免 CLI 漏填导致策略行为不一致。
pub fn apply_quote_defaults(request: &mut QuoteRequest, defaults: &JupiterQuoteConfig) {
    if !request.only_direct_routes.unwrap_or(false) && defaults.only_direct_routes {
        request.only_direct_routes = Some(true);
    }

    if request.restrict_intermediate_tokens.unwrap_or(true)
        && !defaults.restrict_intermediate_tokens
    {
        request.restrict_intermediate_tokens = Some(false);
    }
}
