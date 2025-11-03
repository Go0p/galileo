# 策略配置目录

本目录包含所有策略的独立配置文件。

## 📁 文件列表

| 文件 | 策略类型 | 说明 |
|------|---------|------|
| `blind_strategy.yaml` | 盲发策略 | 基于报价的自动套利策略 |
| `pure_blind_strategy.yaml` | 纯盲发策略 | 基于市场缓存的盲发策略 |
| `copy_strategy.yaml` | 跟单策略 | 监控并复制其他钱包的交易 |
| `back_run_strategy.yaml` | 后跑策略 | 监控价格波动并进行套利 |

## 🎯 使用方式

### 启用策略

在 `galileo.yaml` 中配置需要启用的策略：

```yaml
bot:
  strategies:
    enabled:
      - blind_strategy
      - copy_strategy
```

需要切换到自定义参数时，可直接在 `enabled` 中书写带有前缀的文件名，例如：

```yaml
bot:
  strategies:
    enabled:
      - blind_strategy_lain_50ip   # 对应 strategies/blind_strategy_lain_50ip.yaml
```

也支持显式写入扩展名（如 `blind_strategy_lain_50ip.yaml`），会优先加载该文件。

### 修改策略配置

直接编辑对应的策略文件即可，例如修改 `blind_strategy.yaml`：

```yaml
# 修改交易参数
base_mints:
  - mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    lanes:
      - min: 1_000_000_000  # 修改最小交易规模
        max: 2_000_000_000  # 修改最大交易规模
        count: 2            # 增加规模数量
        strategy: linear
    min_quote_profit: 2000  # 调整最小利润
```

### 配置优先级

1. **外部文件优先** - 如果 `strategies/` 目录中存在对应的策略文件，将使用该文件
2. **主配置兜底** - 如果外部文件不存在，使用 `galileo.yaml` 中的配置（如果有）
3. **默认值** - 如果都没有，使用代码中的默认配置

## 💡 最佳实践

### 1. 版本管理
```bash
# 为不同场景创建配置备份
cp blind_strategy.yaml blind_strategy.aggressive.yaml
cp blind_strategy.yaml blind_strategy.conservative.yaml
```

### 2. 快速切换
```bash
# 切换到激进配置
cp blind_strategy.aggressive.yaml blind_strategy.yaml

# 切换到保守配置
cp blind_strategy.conservative.yaml blind_strategy.yaml
```

### 3. 测试配置
```bash
# 在 dry-run 模式下测试
cargo run -- run
# 观察日志确认策略配置加载正确
```

## 🔍 调试

### 查看加载日志

设置日志级别为 debug：

```yaml
# galileo.yaml
global:
  logging:
    level: "debug"
```

运行后查看策略加载日志：

```
[config::strategy] 开始加载策略配置 enabled=["blind_strategy"] strategy_dir="strategies/"
[config::strategy] 已从外部文件加载策略配置 strategy="blind_strategy" path="strategies/blind_strategy.yaml"
```

### 常见问题

**Q: 修改了策略文件但没有生效？**

A: 
1. 确认策略已在 `galileo.yaml` 的 `bot.strategies.enabled` 中启用
2. 重启 galileo 以重新加载配置
3. 检查日志确认文件已加载

**Q: 策略文件加载失败怎么办？**

A: 
- 检查 YAML 语法是否正确
- 查看日志中的错误信息
- 系统会自动降级使用默认配置，不会中断运行

## 📝 配置说明

详细的策略配置说明请参考 `../STRATEGY_CONFIG.md`。
