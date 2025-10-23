# x-client 请求头生成说明

该脚本复现了前端在 `reverser/dflow/1.js` (`m6e` 函数) 中生成 `x-client-timestamp` 与 `x-client-request-id` 的逻辑，便于在本地调试及后续用 Rust 实现。

## 生成流程
- 取当前毫秒时间戳 `timestamp = Date.now()` 并转成 8 字节 **小端序**（`BigUint64Array` 在浏览器中使用主机序，实测为 little endian）；
- 将 `"{path}5_{body}k"` 用 UTF-8 编码，与时间戳字节拼接后做 `SHA-256`，只保留前 15 个字节并转成 30 位十六进制小写串；
- 随机生成 UUID，抽取第 15、20 个字符分别插入到哈希的第 13、16 字符位置；
- 按 UUID 模式切段：`8-4-4-4-12`，组合成最终的 `x-client-request-id`；
- `x-client-timestamp` 直接使用上述毫秒时间戳的字符串。

## Python 实现
位于 `reverser/dflow/client_headers.py`。
调用示例：

```bash
python reverser/dflow/client_headers.py \
  --path /auth/token \
  --body '{"turnstileToken":"token","userPublicKey":"pk"}'
```

支持以 `@file.json` 读取请求体。

## Rust 实现提示
- 使用 `sha2::Sha256` 计算哈希（记得只取前 15 字节）；
- 时间戳可通过 `std::time::SystemTime::now()` 转换为毫秒；
- UUID 可用 `uuid::Uuid::new_v4()`，直接访问 `uuid.as_simple().encode_lower()` 获得 32 位十六进制串；
- 最终分段拼接建议使用 `format!`/`write!` 避免堆分配。
