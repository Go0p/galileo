# Jito Bundle: HumidiFi ↔ SolFi V2 WSOL-USDC 套利

四笔交易按顺序封装，穿桥 HumidiFi 与 SolFi V2 两个 WSOL/USDC 池完成闭环套利。本文整合链上数据与 dex metadata，方便逐项核对。

## 交易总览

| 顺序 | 签名 | 摘要 |
| --- | --- | --- |
| 1 | `iBz63i1jpQDTjQPebNQqzocukGjqc1MH9SKeLCpPBmhMAoUdHqv23YwasBiEnvZs544M1w6scdsQmTNjbS2UXVF` | 资金注入，创建临时 ATA，迁移 87.137864099 WSOL |
| 2 | `2JtqxeqstEmoe8T64WVs3hriZBX6YzxwM4qasJBQwk8GjoMkb3aQttk4ivnqrrWoQvQBuCdSMamJYWPKKZV3JTPc` | 第一腿：在 HumidiFi WSOL/USDC 池卖出 87.137864099 WSOL，获得 16 096.973203 USDC |
| 3 | `62oAJMgUbgMP8NoRH41GeRT5BPchnTrM2fR3EaqNt6mwZmQXxJcuGcNLkb6fTFqN2C73Wescmg54DHQzswUncgua` | 第二腿：在 SolFi V2 池买回 87.140655568 WSOL，并用 Jupiter 对冲库存 |
| 4 | `5cajdQK9CXzqJnBn54p4rwBFC1Aze72vtWmSrwV42VevqLXehU83yrnvm5qHHBrBf5AMxEo1DxTzUThUHRQkr1Uv` | 归还本金 / rent，关闭临时账户，支付 Jito tip |

## 关键账户与 owner

| 地址 | 角色 | 所属程序 (owner) |
| --- | --- | --- |
| `2uVaeXJu9QjZ7JgzUFCwJWLZ3jgBrLkDhxtoNDh2Zm9Q` | Funding wallet (pays gas / receives profit) | System Program |
| `EHb1GFAo3nnctjS2LSRrDWfNW4zPgjpYbBQrgqo6iJ3K` | Scratch signer wallet (closed at end) | — (account closed at bundle end) |
| `2afj1RQnpRyJdSq3hD7nWaQhz86QTbowZgzD8iQBinWL` | Temporary USDC ATA (owner = scratch wallet) | — (account closed at bundle end) |
| `Ew8NdrRrxiAdAMmS9DrevjMh9cVcRgKcif88kN9zpiSS` | Temporary WSOL ATA (owner = scratch wallet) | — (account closed at bundle end) |
| `ATur6F13xF2aSkySrCoaDTmZ2ugdp2g8zp8faEzRmv7X` | Funding wallet WSOL ATA (initial source) | 9wg6jEjNCRc7v96SG847KDnyJvXmYeEPMzeNpCBmBqGA |
| `2jHgHDRgtwvXRE54hGtDqfTs397R2a8AooUaYtaoJAo5` | Funding wallet USDC ATA (unused) | SPL Token Program |
| `9wg6jEjNCRc7v96SG847KDnyJvXmYeEPMzeNpCBmBqGA` | Helper program controlling wrap/sync/cleanup | Upgradeable Loader |
| `DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH` | DFlow entry dispatcher (leg #1) | Upgradeable Loader |
| `9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp` | HumidiFi router program (🦈) | Upgradeable Loader |
| `6n9VhCwQ7EwK6NqFDjnHPzEk6wZdRBTfh43RFgHQWHuQ` | HumidiFi WSOL-USDC market (program-owned state) | HumidiFi Router Program |
| `Cv9St5tDTGwpbG5UVvM6QvFmf3FYSXc14W9BYvQN5wAZ` | — | SPL Token Program |
| `7Rf8Gu8YemSoGjZT3z1cL5BT9HLbGywcyaz8Mrbhd1MH` | HumidiFi WSOL vault (token account) | SPL Token Program |
| `SV2EYYJyRz2YhfXwXnhNAevDEui5Q6yrfyo13WtupPF` | SolFi V2 router program | Upgradeable Loader |
| `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4` | Jupiter v6 router | Upgradeable Loader |
| `65ZHSArs5XxPseKQbB1B4r16vDxMWnCxHMzogDAqiDUc` | SolFi V2 WSOL-USDC market account | SolFi V2 Router Program |
| `GhFfLFSprPpfoRaWakPMmJTMJBHuz6C694jYwxy2dAic` | SolFi V2 market USDC vault | SPL Token Program |
| `CRo8DBwrmd97DJfAnvCv96tZPL5Mktf2NZy2ZnhDer1A` | SolFi V2 market WSOL vault | SPL Token Program |
| `D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf` | Jupiter route config account | System Program |
| `9yPwvfTgHve2tqgAfN9S15B1pHArfpE4rr1ykykYUJgk` | Funding wallet primary WSOL ATA | SPL Token Program |
| `DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL` | Jito tip receiver | Jito Tip Router |
| `jitodontfron11111111111111111JustUseJupiter` | DFlow anti-front-running PDA | — (account closed at bundle end) |
| `2ny7eGyZCoeEVTkNLf5HcnJFBKkyA4p4gcrtb3b8y8ou` | SolFi V2 config account | SolFi V2 Config Program |
| `FmxXDSR9WvpJTCh738D1LEDuhMoA8geCtZgHb3isy7Dp` | SolFi V2 market metadata | SolFi V2 Router Program |

> HumidiFi 与 SolFi V2 的市场 / vault 布局可对照 `src/dexes/humidifi/decoder.rs` 与 `src/dexes/solfi_v2/decoder.rs`。

## 流程拆解

### 1. 资金准备 (`iBz63i1…`)

- `2uVae…` → `EHb1…`: 转入 0.01 SOL 用于租金与手续费。
- 创建两只 ATA（owner 均为 `EHb1…`）：
  - `2afj…`：USDC ATA；
  - `Ew8…`：WSOL ATA（native 刚同步）。
- 通过 `9wg6…` 程序：
  - 从 `ATur6…` (资金原 WSOL ATA) 向 `Ew8…` 转入 **87.137864099 WSOL**；
  - 资金钱包余量降至 30.673536514 WSOL，用作本次套利本金。

### 2. 第一腿：HumidiFi WSOL→USDC (`2Jtqxeq…`)

- 入口 `DF1ow4…` 调用 HumidiFi 主程序 `9H6tua…`。该程序根据 `src/dexes/humidifi/decoder.rs` 解析市场状态。
- 相关账户：
  - 市场账号：`6n9VhC…` (HumidiFi WSOL-USDC Market state)；
  - WSOL vault：`7Rf8Gu…`；USDC vault：`Cv9St5…`；
  - 用户输入 ATA：`Ew8…` (WSOL)；输出 ATA：`2afj…` (USDC)。
- Swap 内部执行：
  1. HumidiFi program 消耗市场状态 (`data: 22BYiN…`)，完成流动性结算。
  2. SPL Token `transfer`：`Ew8…` → `Cv9St5…` 扣除 **87.137864099 WSOL**；
  3. SPL Token `transfer`：`7Rf8Gu…` → `2afj…` 支付 **16 096.973203 USDC**。
- 换言之，第 2 笔交易的“精确输出”由 HumidiFi 池和路由合约在链下模拟确定，链上只完成最终资金划转。

### 3. 第二腿：SolFi V2 USDC→WSOL (`62oAJMg…`)

- 入口 `SV2EYY…`（SolFi V2 Router）读取市场状态：
  - 市场账号：`65ZHS…`；
  - USDC vault：`GhFfLF…`；WSOL vault：`CRo8DB…`；
  - 这些布局可对照 `src/dexes/solfi_v2/decoder.rs`。
- Swap 过程：
  1. `SV2EYY…` 解码市场并计算所需 tick，`KDzfvS…` 指令完成状态更新；
  2. SPL Token `transfer`：`2afj…` → `GhFfLF…` 归还 **16 096.973203 USDC**；
  3. SPL Token `transfer`：`CRo8DB…` → `Ew8…` 支付 **87.140655568 WSOL**；
  4. Jupiter `JUP6Lk…` 以 `D8cy…` 配置跑公开路由，帮 SolFi V2 vault 对冲风险。
- 至此临时钱包持有的 WSOL 增至 87.140655568，较第一腿拿出的 87.137864099 增加 0.002791469 WSOL。

### 4. 收尾 (`5cajdQK9…`)

- `9wg6…` 负责清理 + 返还租金，资金钱包向 `DttWaMu…` 支付 0.00041872 SOL tip。
- Inner instructions：
  - `Ew8…` → `9yPwvf…` (资金主 WSOL ATA) 归还 **87.140655568 WSOL**；
  - 关闭 `2afj…`/`Ew8…`，rent 退给 `EHb1…`；
  - `EHb1…` → `2uVae…` 返还剩余 0.00999 SOL。

## 资金变化总结

- 投入：87.137864099 WSOL（来自 HumidiFi 入口）。
- 回收：87.140655568 WSOL（经 SolFi V2 + Jupiter 回补）。
- 净赚：**0.002791469 WSOL** (~0.28%)；附加成本仅为 gas + 0.00041872 SOL Jito tip。