# Monitoring (Loki + Promtail + Prometheus + cAdvisor + Grafana)

This project includes a self-hosted log stack:

- `Loki` for log storage and querying
- `Promtail` for collecting application logs
- `Prometheus` for metrics storage/query
- `cAdvisor` for Docker container CPU/memory metrics
- `Grafana` for dashboards and analysis

## What is collected

Promtail tails JSON log files written by:

- `backend_rust` from `/app/logs` (mounted as `backend-logs` volume)
- `tgbot_rust` from `/app/logs` (mounted as `bot-logs` volume)

## Start monitoring

Start app + monitoring together:

```bash
docker compose \
  -f docker-compose.yml \
  -f docker-compose.monitoring.yml \
  up -d backend bot loki promtail prometheus cadvisor grafana
```

For split deployment:

- core host: use `docker-compose.core.yml` + `docker-compose.monitoring.yml`
- bot host: use `docker-compose.bot.yml` + `docker-compose.monitoring.yml`

Example (core host):

```bash
docker compose \
  -f docker-compose.core.yml \
  -f docker-compose.monitoring.yml \
  up -d backend loki promtail prometheus cadvisor grafana
```

## Access

- Grafana: `http://localhost:${GRAFANA_PORT:-3001}` (`admin/admin` by default)
- Loki API: `http://localhost:${LOKI_PORT:-3100}`
- Prometheus: `http://localhost:${PROMETHEUS_PORT:-9090}`

Provisioned automatically on startup:

- Dashboard: `FRBKTG / FRBKTG Observability`
- Alert group: `FRBKTG Alerts / frbktg-loki-alerts`
- Contact point: `frbktg-default-contact` (email receiver)
- Container CPU/Memory panels: `CPU Usage (cores)` / `Memory Usage (bytes)`

Environment overrides:

- `GRAFANA_PORT`
- `GRAFANA_ADMIN_USER`
- `GRAFANA_ADMIN_PASSWORD`
- `LOKI_PORT`
- `PROMETHEUS_PORT`

## Useful LogQL queries

All backend logs:

```logql
{app="backend_rust"}
```

All bot logs:

```logql
{app="tgbot_rust"}
```

Slow backend HTTP requests (`TraceLayer`):

```logql
{app="backend_rust"} |= "request completed" | json | fields_latency_ms >= 1000
```

p95 backend request latency over 5 minutes:

```logql
avg(quantile_over_time(0.95, {app="backend_rust"} |= "request completed" | json | span_uri != "/healthz" | unwrap fields_latency_ms [5m]))
```

Slow callback handling in bot:

```logql
{app="tgbot_rust"} |= "Slow callback handling"
```

p95 callback total latency over 5 minutes:

```logql
avg(quantile_over_time(0.95, {app="tgbot_rust"} |= "Callback handled" | json | unwrap fields_total_elapsed_ms [5m]))
```

Queue wait outliers in dispatch pipeline:

```logql
{app="tgbot_rust"} |= "Slow dispatched message processing" | json | fields_queue_wait_ms >= 500
```

## Default alert thresholds

- `Bot slow callbacks spike`: `count_over_time(...) > 20` over 5m for 2m
- `Backend HTTP p95 high`: `p95(latency_ms) > 1000` for 5m
- `Bot dispatch queue wait high`: `p95(queue_wait_ms) > 500` for 3m
- `Backend 5xx spike`: `request failed > 15` over 5m for 2m
- `Bot dispatch errors spike`: `dispatcher/dispatch errors > 8` over 5m for 2m
- `Panic detected`: any panic line in backend or bot logs in 5m window

## Configure notifications

By default, alert provisioning uses:

- Contact point name: `frbktg-default-contact`
- Receiver type: `email`
- Placeholder address: `ops@example.com`

Change `monitoring/grafana/alerting/contact-points.yml` to your real recipient
before production use (email, Telegram webhook, Slack webhook, etc.).

## Notes

- The logs are process-local files, so each host aggregates its own containers.
- In multi-host setups, aggregate all hosts into one central Loki if you need global analysis.
