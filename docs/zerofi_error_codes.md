# ZeroFi 自定义错误码速查

> 数据来源：`reverser/zerofi/asm/disassembly.out` 与 `immediate_data_table.out`。  
> 通过反汇编可见，ZeroFi 合约遇到断言失败时会调用同一套报错逻辑，并返回 `Custom(<code>)`。下表按 `custom program error: 0x?` 对应的十进制值整理常见触发场景。

## Custom(5)

围绕 `prepare_v2` 配置检查与价差缓存状态的一组断言：

- `config.quantity_skew_strength >= 0`
- `config.quantity_skew_limit_bps >= 0`
- `config.gradual_aggressive_skew_step_millibps >= 0`
- `state.has_saved_spread_modifiers.value()` / `!state.has_saved_spread_modifiers.value()`
- `quoting_version != QuotingVersion::V1 as u8`

**排查建议**

1. 拉取池子的 `market.config`，确认所有 skew 相关参数均为非负，尤其是机器人侧自填的量化参数。
2. 若曾调用过 `save_spread_modifiers` 类入口，需保证落地顺序正确：保存之前状态应为 “未保存”，恢复结束后应重置标志位。
3. 确认行情配置使用 ZeroFi 当前支持的 `quoting_version`（V2 及以上），避免遗留 V1 配置。

## Custom(9)

出现在订单期限、浮点运算与报价版本的校验逻辑中：

- `!self.is_nan()`：浮点结果异常（多段计算后的结果为 NaN）
- `min <= max`：层级参数或价格区间出现颠倒
- `args.order_full_expiry_slots.is_none()` / `args.order_full_expiry_seconds.is_none()`：两种超时字段必须只有一个有效
- `quoting_version != QuotingVersion::V1 as u8`、`quoting_version == QuotingVersion::V1 as u8`：不同路径对版本的互斥检查

**排查建议**

1. 检查调用方是否同时设置了 `expiry_slots` 与 `expiry_seconds`，ZeroFi 要求这两个字段二选一。
2. 校验所有传入的 `min/max`、`lower/upper` 结构，确保最小值不大于最大值。
3. 若是价格/权重经过浮点换算后触发 NaN，重点排查除数是否为 0 或存在未初始化的系数。
4. 同样确认 `quoting_version` 与路由入口匹配。

## Custom(13)

- `layer.expiry_fast_start >= layer.expiry_slow_start`

说明单层策略的“快路径”起始时间比“慢路径”还早，违背合约预设。

**排查建议**：重新生成层配置，确保 `fast_start`（抢先执行时间）不领先 `slow_start`。

## Custom(17)

- `layer.spread >= prev_spread`

层与层之间的价差要求单调不减。

**排查建议**：按照层序排序后核对所有 `spread` 值，确保后续层的价差不低于前一层。

## 附：阅读日志的小技巧

- ZeroFi 不会输出人类可读的断言信息，需结合本表对照 `Custom(n)`。
- 若遇到其它 `ProgramError::*`（例如 `InsufficientFunds` 等标准错误），可直接参考 Solana 官方枚举；这些并非 ZeroFi 自定义错误。
- 发生 `Custom` 错误时建议同步记录 `swap` 输入参数、池子配置信息，方便定位具体断言。

