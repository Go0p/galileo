# 策略配置文件系统

## 📖 概述

从当前版本开始，Galileo 支持将策略配置拆分为独立的 YAML 文件，便于管理和快速切换不同的策略组合。

## 🎯 设计理念

### 优先级
1. **主配置优先**：如果在 `galileo.yaml` 中已配置策略（非默认值），则使用主配置
2. **外部文件次之**：如果主配置为空，自动从 `strategies/` 目录加载对应的策略文件
3. **默认配置兜底**：如果外部文件也不存在，使用代码中的默认配置

### 向后兼容
- ✅ 旧版配置（所有策略在 `galileo.yaml` 中）仍然有效
- ✅ 新版配置（策略拆分到独立文件）自动生效
- ✅ 混合模式（部分在主文件、部分在外部文件）也支持

## 📁 目录结构

```
galileo/
├── galileo.yaml                 # 主配置（精简）
├── strategies/                  # 策略配置目录
│   ├── blind_strategy.yaml
│   ├── pure_blind_strategy.yaml
│   ├── copy_strategy.yaml
│   └── back_run_strategy.yaml
└── presets/                     # 可选：预设策略组合
    ├── aggressive/
    │   ├── blind_strategy.yaml
    │   └── copy_strategy.yaml
    └── conservative/
        └── blind_strategy.yaml
```

## ⚙️ 配置方式

### 方式一：精简主配置 + 外部策略文件（推荐）

**galileo.yaml（精简）**
```yaml
bot:
  strategies:
    enabled:
      - blind_strategy
      - copy_strategy
    config_dir: "./strategies"  # 可选，默认为 "strategies"
```

**strategies/blind_strategy.yaml**
```yaml
memo: ""
enable_dexs: []
exclude_dexes: []
enable_landers: []
base_mints:
  - mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    lanes:
      - min: 600_000_000
        max: 1_200_000_000
        count: 1
        strategy: linear
    min_quote_profit: 1000
    sending_cooldown: 1000
    route_types:
      - "2hop"
```

### 方式二：传统方式（向后兼容）

**galileo.yaml（完整）**
```yaml
bot:
  strategies:
    enabled:
      - blind_strategy

blind_strategy:
  memo: ""
  enable_dexs: []
  # ... 完整配置
```

## 🚀 使用场景

### 场景一：快速切换策略组合

```bash
# 测试配置
bot:
  strategies:
    enabled:
      - blind_strategy
    config_dir: "./presets/test"

# 激进配置
bot:
  strategies:
    enabled:
      - blind_strategy
      - copy_strategy
      - pure_blind_strategy
    config_dir: "./presets/aggressive"

# 保守配置
bot:
  strategies:
    enabled:
      - blind_strategy
    config_dir: "./presets/conservative"
```

### 场景二：预设多套策略配置

```bash
strategies/
├── default/              # 默认配置
│   ├── blind_strategy.yaml
│   └── copy_strategy.yaml
├── high_volume/          # 高频交易配置
│   ├── blind_strategy.yaml
│   └── copy_strategy.yaml
└── low_risk/            # 低风险配置
    └── blind_strategy.yaml
```

只需修改 `config_dir` 即可切换：
```yaml
bot:
  strategies:
    enabled:
      - blind_strategy
      - copy_strategy
    config_dir: "./strategies/high_volume"  # 切换到高频配置
```

### 场景三：按环境管理

```bash
strategies/
├── production/     # 生产环境
├── staging/        # 预发布环境
└── development/    # 开发环境
```

## 📝 配置文件命名规则

策略配置文件必须按以下规则命名：

| 策略类型 | 文件名 |
|---------|--------|
| Blind Strategy | `blind_strategy.yaml` |
| Pure Blind Strategy | `pure_blind_strategy.yaml` |
| Copy Strategy | `copy_strategy.yaml` |
| Back Run Strategy | `back_run_strategy.yaml` |

## 🔍 调试

启用日志查看策略加载过程：

```yaml
global:
  logging:
    level: "debug"
```

查看日志输出：
```
[config::strategy] 开始加载策略配置 enabled=["blind_strategy", "copy_strategy"] strategy_dir="strategies/"
[config::strategy] 已从外部文件加载策略配置 strategy="blind_strategy" path="strategies/blind_strategy.yaml"
[config::strategy] 使用主配置文件中的策略配置 strategy="copy_strategy"
```

## ⚡ 性能说明

- ✅ 只加载启用的策略文件（按需加载）
- ✅ 加载失败不会中断程序（自动降级到默认配置）
- ✅ 配置文件只在启动时加载一次

## 🎓 最佳实践

1. **使用外部文件**：将策略配置拆分到 `strategies/` 目录
2. **版本控制**：为不同场景创建预设配置目录
3. **命名规范**：使用清晰的目录名（如 `production`, `test`）
4. **文档注释**：在策略文件中添加详细注释
5. **测试验证**：切换配置后在 dry-run 模式下验证

## 🐛 常见问题

### Q: 策略配置没有生效？
A: 检查以下几点：
1. 文件名是否正确（必须是 `{strategy_name}.yaml`）
2. 文件路径是否正确（相对于配置文件所在目录）
3. 查看日志确认加载状态

### Q: 如何临时覆盖某个策略的配置？
A: 直接在 `galileo.yaml` 中配置该策略，主配置会覆盖外部文件

### Q: 可以使用绝对路径吗？
A: 可以，`config_dir` 支持绝对路径和相对路径

## 📚 示例

参考 `strategies/blind_strategy.yaml` 查看完整的配置示例。

