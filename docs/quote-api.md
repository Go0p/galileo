# 🪐 Jupiter Swap 参数说明（完整文档）

本文档描述了 Jupiter Swap API 的所有可用参数、类型、默认值以及行为说明。

---

## 📋 参数总览

| 参数名 | 类型 | 是否必填 | 默认值 | 说明 |
|--------|------|-----------|--------|------|
| **inputMint** | `string` | ✅ 必填 | — | 输入代币的 mint 地址 |
| **outputMint** | `string` | ✅ 必填 | — | 输出代币的 mint 地址 |
| **amount** | `uint64` | ✅ 必填 | — | 交换的原始数量（未除以 decimals）<br>若 `swapMode = ExactIn`，为输入数量；<br>若 `swapMode = ExactOut`，为输出数量。 |
| **slippageBps** | `uint16` | 可选 | — | 滑点（以 bps 为单位，1 bps = 0.01%）<br>在 `ExactIn` 模式下滑点作用于输出代币；<br>在 `ExactOut` 模式下滑点作用于输入代币。 |
| **swapMode** | `string` | 可选 | `ExactIn` | 可选值：`ExactIn` / `ExactOut`<br>🔹 `ExactIn`：输入数量固定，输出受滑点限制。<br>🔹 `ExactOut`：输出数量固定，输入可能上浮（仅部分 AMM 支持）。<br>支持 `ExactOut` 的 DEX：Orca Whirlpool、Raydium CLMM、Raydium CPMM。 |
| **dexes** | `string[]` | 可选 | — | 限定可使用的 DEX 列表（用逗号分隔）。<br>示例：`dexes=Raydium,Orca+V2,Meteora+DLMM` |
| **excludeDexes** | `string[]` | 可选 | — | 指定要排除的 DEX 列表（用逗号分隔）。<br>示例：`excludeDexes=Raydium,Orca+V2,Meteora+DLMM` |
| **restrictIntermediateTokens** | `boolean` | 可选 | `true` | 限制中间路由代币为稳定资产，以降低高滑点风险。 |
| **onlyDirectRoutes** | `boolean` | 可选 | `false` | 仅允许单跳（Direct）路由，可能导致更差的报价。 |
| **asLegacyTransaction** | `boolean` | 可选 | `false` | 使用传统（legacy）事务而非 versioned transaction。 |
| **platformFeeBps** | `uint16` | 可选 | — | 平台手续费（以 bps 计），需与 `/swap` 接口中的 `feeAccount` 配合使用。 |
| **maxAccounts** | `uint64` | 可选 | `64` | 估算路由所使用的最大账户数，用于更精确的资源评估。 |
| **dynamicSlippage** | `boolean` | 可选 | `false` | 若为 `true`，`slippageBps` 会被动态滑点计算覆盖（返回值由 `/swap` 接口提供）。 |

---

## 🧠 模式说明

| 模式 | 含义 | 滑点方向 | 典型用途 |
|------|------|-----------|-----------|
| **ExactIn** | 固定输入数量 | 输出代币数量浮动 | 常规 swap、套利 |
| **ExactOut** | 固定输出数量 | 输入代币数量浮动 | 支付类场景（确保用户收到确切金额） |

### 📘 示例
- **ExactIn 模式**  
  你给出 1 SOL，系统计算你能获得多少 USDC（输出浮动）。  
- **ExactOut 模式**  
  你希望收到正好 100 USDC，系统计算你需要付出多少 SOL（输入浮动）。


