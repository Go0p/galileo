use crate::api::QuoteRequest;
use crate::config::RequestParamsConfig;

/// 解析 `key=value` 形式的额外查询参数。
pub fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| "参数格式需为 key=value".to_string())?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// 根据配置补全报价默认值，避免 CLI 漏填导致策略行为不一致。
pub fn apply_request_defaults_to_quote(request: &mut QuoteRequest, params: &RequestParamsConfig) {
    if !request.only_direct_routes.unwrap_or(false) && params.only_direct_routes {
        request.only_direct_routes = Some(true);
    }

    if request.restrict_intermediate_tokens.unwrap_or(true) && !params.restrict_intermediate_tokens
    {
        request.restrict_intermediate_tokens = Some(false);
    }
}
