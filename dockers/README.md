# Monitoring Stack (Prometheus + Grafana)

该目录提供最小化的监控依赖，可采集 Galileo bot 的 Prometheus 指标。

## 目录结构

- `docker-compose.yml`：Prometheus 与 Grafana 的编排配置
- `prometheus.yml`：Prometheus 抓取配置，默认抓取 `galileo` bot (`host.docker.internal:9898`)
- Prometheus / Grafana 数据默认持久化到 Docker volume（`prometheus-data` / `grafana-data`）

## 前置条件

1. `galileo.yaml` 中开启 Prometheus，并监听在容器可访问的地址（VPS 上建议 `0.0.0.0`，再配合安全组/防火墙限制来源）：
   ```yaml
   bot:
     prometheus:
       enable: true
       listen: "0.0.0.0:9898"
   ```
2. `docker-compose.yml` 已将 `host.docker.internal` 映射到宿主机（`host-gateway`），若遇到无法访问，可在 `prometheus.yml` 中改为宿主机实际内网 IP。

## 启动

```bash
cd dockers
docker compose up -d
```

服务启动后：
- Prometheus：<http://localhost:9090>（容器映射到 `127.0.0.1:9090`，默认仅宿主可访问）
- Grafana：<http://localhost:3000>（默认账号 admin/admin）

Grafana 添加 Prometheus 数据源时使用 `http://prometheus:9090`。

## 常用指标

- `galileo_quote_total{strategy,result}` / `galileo_quote_latency_ms_bucket`
- `galileo_opportunity_detected_total{strategy}`
- `galileo_lander_success_total{strategy,lander}` / `galileo_lander_failure_total{...}`
- `galileo_transaction_built_total{strategy}`
- 可基于上述指标构建“机会发现/成功率、Quote 延迟、落地成功率”等 Grafana 面板。

## 停止与清理

```bash
# 停止服务
cd dockers
docker compose down

# 如需清理数据卷
docker volume rm dockers_prometheus-data dockers_grafana-data
```

> **注意**：生产环境可进一步加固安全（TLS、反向代理、Grafana 用户权限等），当前配置仅用于本地调试/自测。
