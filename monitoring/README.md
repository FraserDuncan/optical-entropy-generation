# Monitoring Configuration

This directory contains configuration files for the Prometheus/Grafana monitoring stack.

## Directory Structure

```
monitoring/
├── prometheus/
│   ├── prometheus.yml      # Main Prometheus configuration
│   └── rules/
│       ├── alerts.yml      # Alerting rules
│       └── recording.yml   # Recording rules for pre-aggregation
├── grafana/
│   ├── dashboards/
│   │   └── entropy-overview.json   # Main operational dashboard
│   └── provisioning/
│       ├── dashboards/default.yml  # Dashboard provisioning
│       └── datasources/prometheus.yml
└── README.md
```

## Prometheus Configuration

### Scrape Configuration

The entropy generator exposes metrics on port 9090 at `/metrics`. Prometheus scrapes these every 5 seconds for real-time monitoring.

```yaml
scrape_configs:
  - job_name: 'optical-entropy'
    static_configs:
      - targets: ['entropy-generator:9090']
    scrape_interval: 5s
```

### Retention

Default retention is set to 30 days with a 10GB size limit. Adjust in `prometheus.yml` based on your storage capacity.

## Alerting Rules

### Critical Alerts (Immediate Response)

| Alert | Condition | Description |
|-------|-----------|-------------|
| EntropyQualityCritical | Unhealthy for 10+ samples, 2+ min | Entropy quality dangerously low |
| EntropyGenerationStopped | No samples in 5 min | Generator not producing entropy |

### Warning Alerts (Investigate Soon)

| Alert | Condition | Description |
|-------|-----------|-------------|
| EntropyQualityDegraded | Unhealthy for 5+ min | Quality below threshold |
| EntropyHighBitBias | Bit bias > 0.1 for 5+ min | Potential bias in entropy |
| EntropyLowVariance | Variance < 500 for 5+ min | Insufficient randomness |
| EntropyHighAutocorrelation | Autocorr > 0.3 for 5+ min | Predictable patterns |
| CSPRNGStaleReseed | No reseed in 1+ hour | CSPRNG may need fresh entropy |

## Recording Rules

Pre-computed metrics for efficient dashboard queries:

- `optical_entropy:samples_per_second:rate1m` - Sample processing rate
- `optical_entropy:bits_per_second:rate5m` - Entropy accumulation rate
- `optical_entropy:health_score:avg1h` - % of time healthy (last hour)
- `optical_entropy:reseeds_per_hour:rate1h` - CSPRNG reseed frequency

## Quick Start

1. Build the entropy generator with metrics feature:
   ```bash
   cargo build --release --features metrics
   ```

2. Start Prometheus with this configuration:
   ```bash
   prometheus --config.file=monitoring/prometheus/prometheus.yml
   ```

3. Access Prometheus UI at http://localhost:9091

## Grafana Dashboards

### Entropy Overview Dashboard

The main operational dashboard provides real-time visibility into:

| Panel | Description |
|-------|-------------|
| Health Status | Current healthy/unhealthy state (color-coded) |
| Entropy Rate | Bits per second over time (1m and 5m averages) |
| Healthy/Unhealthy Streak | Consecutive sample counts |
| Total Reseeds | CSPRNG reseed counter |
| Bytes Since Reseed | Data generated since last reseed |
| Bit Bias | Statistical test with threshold lines |
| Byte Variance | Statistical test with threshold lines |
| Autocorrelation | Statistical test with threshold lines |
| Pool Size | Gauge showing current pool fill level |
| Total Extractions/Samples | Counter statistics |

### Dashboard Provisioning

Dashboards are automatically loaded via Grafana provisioning. Place JSON files in `grafana/dashboards/` and they'll be available on startup.

## Docker Compose Example

```yaml
services:
  entropy-generator:
    build: .
    ports:
      - "9090:9090"

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./monitoring/prometheus:/etc/prometheus
    ports:
      - "9091:9090"
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'

  grafana:
    image: grafana/grafana:latest
    volumes:
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning
      - ./monitoring/grafana/dashboards:/var/lib/grafana/dashboards
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_AUTH_ANONYMOUS_ENABLED=true
      - GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer
```
