# Jupiter Swap API 参数详解

本页面整理 `/swap` 与 `/swap-instructions` 相关字段，结合官方说明与实操经验，帮助你在自托管场景下理解每个参数的意义、适用场景以及互斥关系。建议在调用前先完成 `/quote` 流程，并保留完整的 quote 响应。  
> 我们的机器人默认走 `/swap-instructions` 路径，获取原始指令后自行拼装交易并发送到链上，本文档仍保留 `/swap` 字段说明以便参考与对比。

---

## 请求体结构总览

```jsonc
{
  "userPublicKey": "...",               // 必填：实际签名用户
  "payer": "...",                       // 可选：手续费支付者
  "wrapAndUnwrapSol": true,
  "useSharedAccounts": true,
  "feeAccount": "...",
  "trackingAccount": "...",
  "prioritizationFeeLamports": { ... }, // 优先费配置（与 Jito tip 互斥）
  "priorityLevelWithMaxLamports": { ... },
  "jitoTipLamports": 0,
  "asLegacyTransaction": false,
  "destinationTokenAccount": "...",
  "dynamicComputeUnitLimit": false,
  "skipUserAccountsRpcCalls": false,
  "dynamicSlippage": false,
  "computeUnitPriceMicroLamports": 0,
  "blockhashSlotsToExpiry": 0,
  "quoteResponse": { ... }              // 必填：/quote 的原始响应
}
```

---

## 核心账号参数

| 字段 | 类型 | 是否必填 | 说明 |
|------|------|----------|------|
| `userPublicKey` | `string` | ✅ | 交易签名者（最终收到输入/输出代币的主体）。 |
| `payer` | `string` | ❌ | 自定义支付者，承担 rent/手续费。若用户自行关闭 ATA，你需要在费用策略中预留重新创建的成本。 |
| `destinationTokenAccount` | `string` | ❌ | 指定输出代币接收账户（需已初始化）。不填时使用 `userPublicKey` 对应 ATA。 |
| `trackingAccount` | `string` | ❌ | 任意属于你的公钥，用于区分监控数据（例如 Dune、Flipside）。 |

---

## SOL / 账户管理相关

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `wrapAndUnwrapSol` | `bool` | `true` | `true`：自动 wrap/unwrap SOL，input 视为 SOL；`false`：直接操作 WSOL，要求已有 WSOL ATA。若设置了 `destinationTokenAccount`，该参数会被忽略。 |
| `useSharedAccounts` | `bool` | 动态决定 | 允许 Jupiter 使用共享中间账户，通常能减少账户创建开销。部分新 AMM 不支持共享账户，必要时可设为 `false`。 |
| `skipUserAccountsRpcCalls` | `bool` | `false` | 跳过预检查，直接在交易中尝试创建缺失账户（会多一步模拟/失败重试）。 |

---

## 费用与优先级策略

### 设置优先费的方式（互斥关系）：

1. **`prioritizationFeeLamports`**：使用内置策略（按等级或固定上限）自动补充优先费。
2. **`priorityLevelWithMaxLamports`**：仅在启用策略时设定封顶 lamports。
3. **`jitoTipLamports`**：给 Jito Bundles 的固定 tip（单位 lamports）。与 `prioritizationFeeLamports` 互斥，需要搭配 Jito RPC。
4. **`computeUnitPriceMicroLamports`**：直接指定 compute unit 单价（推荐改用 `prioritizationFeeLamports + dynamicComputeUnitLimit`）。

> ⚠️ 在自托管环境下，如需同时给优先费和 Jito tip，可考虑改用 `/swap-instructions` 手工组合。

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `prioritizationFeeLamports` | `object` | — | Jupiter 标准优先费配置，对应“优先费模板 + 最大值”等结构。 |
| `priorityLevelWithMaxLamports` | `object` | — | 配合上面字段使用，直接限制最大付费。 |
| `jitoTipLamports` | `uint64` | — | 固定 tip 数额，需使用 Jito RPC，参考官方 percentile 建议。 |
| `computeUnitPriceMicroLamports` | `uint64` | — | 精确指定 `compute unit price`（微 lamports）。 |
| `dynamicComputeUnitLimit` | `bool` | `false` | 启用时会先模拟交易，动态写入 CU 限额，提高成功率但多一次 RPC。 |

### 平台费用 & 追踪

| 字段 | 类型 | 说明 |
|------|------|------|
| `feeAccount` | `string` | 平台费收取账户，必须是输入或输出代币的 ATA。需与 `/quote` 请求中的 `platformFeeBps` 配合。 |

---

## 交易格式与有效期

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `asLegacyTransaction` | `bool` | `false` | 是否构建 Legacy 交易（配合 `/quote` 同步开启），防止部分钱包不兼容 Versioned Tx。 |
| `blockhashSlotsToExpiry` | `uint8` | `0` | 交易有效 slot 数（0 代表使用默认 150 slots）。例如设为 10，约等于 ~4 秒。 |

---

## 动态滑点

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `dynamicSlippage` | `bool` | `false` | 若为 `true`，/swap 会重新评估滑点并覆盖 quote 中的 `slippageBps`，需与 `/quote` 的同名参数一起使用。 |

---

## quoteResponse 字段详解

`quoteResponse` 必须是 `/quote` 的完整响应，常见字段如下：

| 字段 | 类型 | 是否必填 | 说明 |
|------|------|----------|------|
| `inputMint` | `string` | ✅ | 输入代币 mint。 |
| `outputMint` | `string` | ✅ | 输出代币 mint。 |
| `inAmount` | `string` | ✅ | 输入数量（原始单位）。 |
| `outAmount` | `string` | ✅ | 预期输出数量。 |
| `otherAmountThreshold` | `string` | ✅ | 按设置滑点计算后的最小输出。/swap 不直接使用，但需保留。 |
| `swapMode` | `"ExactIn" \| "ExactOut"` | ✅ | 交易模式，与 `/quote` 保持一致。 |
| `slippageBps` | `uint16` | ✅ | 滑点（基点）。 |
| `priceImpactPct` | `string` | ✅ | 价格冲击。 |
| `routePlan` | `array` | ✅ | 具体路线，每个元素对应一次 AMM 路由，内部包含 `swapInfo`、`feeMint` 等细节。 |
| `contextSlot` | `uint64` | ❌ | 报价使用的 slot。 |
| `timeTaken` | `number` | ❌ | 报价耗时（毫秒）。 |
| `platformFee` | `object` | ❌ | 包含 `amount` 与 `feeBps`，代表平台费配置。 |

### `routePlan.swapInfo` 内部字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `ammKey` | `string` | 使用的 AMM program key。 |
| `label` | `string` | AMM 标签（如 Orca、Raydium）。 |
| `inputMint` / `outputMint` | `string` | 当前小路由的输入/输出 mint。 |
| `inAmount` / `outAmount` | `string` | 当前小路由的输入/输出数量。 |
| `feeAmount` / `feeMint` | `string` | 手续费金额及币种。 |
| `percent` | `uint8` | 该路由占整体百分比。 |
| `bps` | `uint16` | 占比对应的基点数。 |

---

## 常见场景建议

1. **本地自托管 & 自动落地**  
   - `wrapAndUnwrapSol = true`（默认即可），确保用户只需持有 SOL。  
   - `useSharedAccounts = true`（默认），减少账户创建。  
   - 优先费建议使用 `prioritizationFeeLamports + dynamicComputeUnitLimit`，同时可在 `lander.yaml` 中对 Jito tip 做更细粒度控制。

2. **策略自备 WSOL 账户**  
   - 将 `wrapAndUnwrapSol` 改为 `false`，避免反复 wrap/unwrap。  
   - 结合自定义 `destinationTokenAccount` 指向策略拉链后的回收账户。

3. **需要 Legacy 交易或精准滑点控制**  
   - `/quote` + `/swap` 均设置 `asLegacyTransaction = true`。  
   - 若要让 Jupiter 自动动态滑点，`dynamicSlippage` 双端同时开启。

4. **整合平台手续费**  
   - `/quote` 指定 `platformFeeBps`，`/swap` 提供 `feeAccount`。注意 fee account 的 mint 必须是输入/输出之一。

---

## 调试技巧

- 如果 `/swap` 返回校验错误，优先确认 `quoteResponse` 是否完整传入，字段不要裁剪。
- `skipUserAccountsRpcCalls = true` 会减少接口阻塞，但若账户缺失，实际链上交易可能失败，需要配合重试/模拟。
- 使用 `blockhashSlotsToExpiry` 限制交易有效期时，务必确保后续发送逻辑也在这个时间窗口内完成。
- 当 `disable_local_binary = false` 且 `jupiter_url` 指向本地，`galileo strategy` 会自动启动 Jupiter；如果 URL 为外部节点，则跳过本地启动逻辑。

---

通过以上参数组合，可以灵活控制交易优先级、费用策略和链上账户行为。建议在正式部署前先在测试环境对不同场景进行演练，确保策略与落地逻辑一致。*** End Patch*** End Patch to=functions.apply_patch  Vaporpress:
