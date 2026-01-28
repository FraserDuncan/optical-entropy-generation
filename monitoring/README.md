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
```
