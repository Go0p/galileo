# Monitoring Stack (Prometheus + Grafana)

该目录提供最小化的监控依赖，可采集 galileo bot 与 Jupiter 自托管二进制的 Prometheus 指标。

## 目录结构

- `docker-compose.yml`：Prometheus 与 Grafana 的编排配置
- `prometheus.yml`：Prometheus 抓取配置，默认抓取
  - `galileo` 服务 (`host.docker.internal:9898`)
  - `jupiter` 服务 (`host.docker.internal:18081`)
- Prometheus / Grafana 数据默认持久化到 Docker volume（`prometheus-data` / `grafana-data`）

## 前置条件

1. `galileo.yaml` 中开启 Prometheus：
   ```yaml
   bot:
     prometheus:
       enable: true
       listen: "0.0.0.0:9898"
   ```
2. Jupiter 二进制以默认 `metrics_port` 暴露指标（18081，可在 `jupiter.core.metrics_port` 调整）。
3. 如在 Linux 上运行，请确认 Docker 支持 `host.docker.internal`；若不支持，可在 `prometheus.yml` 或环境变量中改成宿主机实际 IP。

## 启动

```bash
cd dockers
# 如需自定义抓取目标，可导出：
# export GALILEO_PROM_TARGET=10.0.0.5:9898
# export JUPITER_PROM_TARGET=10.0.0.6:18081
# 自定义 Grafana 管理员账号：
# export GRAFANA_ADMIN_USER=alice
# export GRAFANA_ADMIN_PASSWORD=secret

# 启动 Prometheus 与 Grafana
docker compose up -d
```

服务启动后：
- Prometheus：<http://localhost:9090>
- Grafana：<http://localhost:3000>（默认账号 admin/admin）

Grafana 添加 Prometheus 数据源时使用 `http://prometheus:9090`。

## 常用指标

- `galileo_quote_total{strategy,result}` / `galileo_quote_latency_ms_bucket`
- `galileo_opportunity_detected_total{strategy}`
- `galileo_lander_success_total{strategy,lander}` / `galileo_lander_failure_total{...}`
- `galileo_transaction_built_total{strategy}`
- Jupiter 自带指标位于 `jupiter_*`、`service_requests_total` 等命名空间

可基于上述指标构建“机会发现/成功率、Quote 延迟、落地成功率”等 Grafana 面板。

## 环境变量说明

- `GALILEO_PROM_TARGET`：Prometheus 抓取 galileo 指标的地址，默认为 `host.docker.internal:9898`
- `JUPITER_PROM_TARGET`：Prometheus 抓取 Jupiter 指标的地址
- `PROMETHEUS_UID`/`PROMETHEUS_GID`：Prometheus 运行用户 UID/GID（默认 65534）
- `GRAFANA_UID`/`GRAFANA_GID`：Grafana 运行用户 UID/GID（默认 472/0）
- `GRAFANA_ADMIN_USER`/`GRAFANA_ADMIN_PASSWORD`：Grafana 初始管理员账号
- `GRAFANA_DOMAIN`：Grafana 对外域名（影响登录回调）

## 停止与清理

```bash
# 停止服务
cd dockers
docker compose down

# 如需清理数据卷
docker volume rm dockers_prometheus-data dockers_grafana-data
```

> **注意**：生产环境可进一步加固安全（TLS、反向代理、Grafana 用户权限等），当前配置仅用于本地调试/自测。
