# Monitoring

Prometheus, Grafana, and Alertmanager configuration for entropy monitoring.

## Structure

```
monitoring/
├── prometheus/
│   ├── prometheus.yml
│   └── rules/
│       ├── alerts.yml
│       └── recording.yml
├── grafana/
│   ├── dashboards/
│   │   └── entropy-overview.json
│   └── provisioning/
│       ├── dashboards/default.yml
│       └── datasources/prometheus.yml
├── alertmanager/
│   ├── alertmanager.yml
│   └── templates/
│       └── email.tmpl
└── README.md
```

## Alerts

| Alert | Severity | Condition |
|-------|----------|-----------|
| EntropyQualityCritical | critical | Unhealthy 10+ samples for 2+ min |
| EntropyGenerationStopped | critical | No samples in 5 min |
| EntropyQualityDegraded | warning | Unhealthy for 5+ min |
| EntropyHighBitBias | warning | Bias > 0.1 for 5+ min |
| EntropyLowVariance | warning | Variance < 500 for 5+ min |
| EntropyHighAutocorrelation | warning | Autocorr > 0.3 for 5+ min |
| CSPRNGStaleReseed | warning | No reseed in 1+ hour |

## Recording Rules

Pre-aggregated metrics for dashboards:

- `optical_entropy:samples_per_second:rate1m`
- `optical_entropy:bits_per_second:rate5m`
- `optical_entropy:health_score:avg1h`
- `optical_entropy:reseeds_per_hour:rate1h`

## Usage

```bash
# Build with metrics
cargo build --release --features metrics

# Run Prometheus
prometheus --config.file=monitoring/prometheus/prometheus.yml

# Or use Docker Compose
docker-compose up
```

## Ports

- Prometheus: 9091
- Grafana: 3000 (admin/admin)
- Alertmanager: 9093
- Entropy metrics: 9090
