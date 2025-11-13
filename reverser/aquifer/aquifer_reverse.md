# Aquifer 账户解析流程

脚本：`reverser/aquifer/aquifer_decode.py`  
目标：给定 Aquifer 池子实例（`instance`），自动推导 `dex`、base / quote mint、coin state、vault，并整理构造 swap 指令所需的全部账户。

## 数据来源

脚本只依赖 Solana RPC，未使用任何暴力爆破或离线字典。主要调用：

1. `getAccountInfo`  
   - 读取 `dex`（~8.3KB）、`instance`（~8.2KB）状态；  
   - 可选读取 `user` 状态用于附加输出。
2. `getProgramAccounts`  
   - 针对程序 `fastC7gqs2WUXgcyNna2BZAe9mte4zcTGprv3mv18N3`，过滤 `dataSize = 128` 拿到 coin state；  
   - 针对程序 `AQU1FRd7papthgdrwPTTq5JacJh8YtwEXaBfKU3bTz45`，过滤 `dataSize = 1056` 拿到 vault info。
3. `getTokenAccountsByOwner`  
   - 以 vault info PDA 为 owner 查询 SPL Token 账户，从而找到真实的金库 ATA。

## 解析逻辑要点

### 1. Coin State (`fastC7g` 程序，128 bytes)

- `offset 24..56`：mint  
- `offset 56..88`：dex 全局账户  
- `offset 88..120`：Aquifer 全局状态（`7rhxn…`）  
- `offset 32..64`：vault info PDA  
- `offset 0..8`：递增索引 / bump，用于错误输出或后续校验  

脚本筛选所有 coin state，只保留 `dex` 匹配的项，再与池子自动推导/用户指定的 base / quote mint 对应上。

### 2. Vault Info (`AQU1…` 程序，1056 bytes)

- `offset 952..984`：mint  
- `offset 960..992`：回指 coin state（与上一步的 `offset 32..64` 相同）  
- `offset 1016..1048`：instance 账户  

脚本同样遍历全部 vault info，按 instance 过滤后即可直接拿到该池子支持的所有 mint；当池子只含两个 mint 时会自动选择 base/quote，否则需手动指定。

### 3. Vault Token Account

对于每个 vault info PDA，调用 `getTokenAccountsByOwner(owner=vault_info, programId=Tokenkeg…)`。当前实现只返回 balance 最大的一条记录——即 swap 中实际使用的金库 SPL Token 账户。

## 输出

运行示例：

```bash
# 兼容旧用法：显式传入 dex + instance
python3 reverser/aquifer/aquifer_decode.py <dex-account> <instance-account>

# 新用法：只传池子（instance），脚本会自动读取 dex
python3 reverser/aquifer/aquifer_decode.py --pool <instance-account>

# 若池子存在多个 mint，可额外指定 --base-mint / --quote-mint
# 仅在池子 >=3 个 mint，或需要强制方向时才必填。
```

输出内容：

- `full_account_list`：按 swap 指令真实顺序列出 16 个 AccountMeta（含 Sysvar、Token Program、coin state、vault info 与金库 ATA）。  
- `coin_states` / `vault_infos`：详细列出解析到的链上状态，包含各字段的原值，脚本会校验 `fast_state ↔ vault_info` 的互相指针，避免串池。  
- `available_vaults`：罗列该 instance 下所有 vault（若 >=3 需手动指定 base/quote），便于快速查看池子支持的 token 集。  
- 其它字段如 `dex_summary`、`instance_summary` 供排查使用。

> 注意：脚本只需要 RPC 访问，不依赖本地反编译结果；依赖的三个 RPC 接口已在代码中标注。
