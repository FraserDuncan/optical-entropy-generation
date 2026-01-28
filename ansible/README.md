# Ansible Deployment for Optical Entropy Monitoring

Ansible playbooks and roles for deploying the complete monitoring stack.

## Directory Structure

```
ansible/
├── inventory/
│   └── hosts.yml           # Inventory with host groups
├── playbooks/
│   └── monitoring.yml      # Main deployment playbook
├── roles/
│   ├── prometheus/         # Prometheus installation
│   ├── grafana/            # Grafana installation
│   ├── alertmanager/       # Alertmanager installation
│   └── entropy-exporter/   # Entropy generator with metrics
└── README.md
```

## Quick Start

1. **Configure inventory:**
   ```bash
   cp inventory/hosts.yml inventory/production.yml
   # Edit production.yml with your host details
   ```

2. **Run the playbook:**
   ```bash
   ansible-playbook -i inventory/production.yml playbooks/monitoring.yml
   ```

3. **Deploy specific components:**
   ```bash
   # Prometheus only
   ansible-playbook -i inventory/production.yml playbooks/monitoring.yml --tags prometheus

   # Grafana only
   ansible-playbook -i inventory/production.yml playbooks/monitoring.yml --tags grafana

   # Update configs only (no reinstall)
   ansible-playbook -i inventory/production.yml playbooks/monitoring.yml --tags config
   ```

## Host Groups

| Group | Purpose |
|-------|---------|
| `monitoring` | Runs Prometheus, Grafana, Alertmanager |
| `entropy_generators` | Runs optical entropy system with metrics |
| `development` | Local development (localhost) |

## Role Variables

### Prometheus

| Variable | Default | Description |
|----------|---------|-------------|
| `prometheus_version` | 2.48.0 | Prometheus version |
| `prometheus_retention_time` | 30d | Data retention period |
| `prometheus_retention_size` | 10GB | Data retention size |
| `prometheus_entropy_targets` | [...] | Entropy generator targets |

### Grafana

| Variable | Default | Description |
|----------|---------|-------------|
| `grafana_version` | 10.2.0 | Grafana version |
| `grafana_admin_password` | admin | Initial admin password |
| `grafana_allow_anonymous` | true | Allow anonymous access |

### Alertmanager

| Variable | Default | Description |
|----------|---------|-------------|
| `alertmanager_version` | 0.26.0 | Alertmanager version |
| `alertmanager_slack_webhook_url` | "" | Slack webhook URL |
| `alertmanager_smtp_smarthost` | ... | SMTP server |

### Entropy Exporter

| Variable | Default | Description |
|----------|---------|-------------|
| `entropy_exporter_port` | 9090 | Metrics port |
| `entropy_rust_features` | metrics | Cargo features |
| `entropy_repo_version` | main | Git branch/tag |

## Development Deployment

Deploy to localhost for testing:

```bash
ansible-playbook -i inventory/hosts.yml playbooks/monitoring.yml \
  --limit development \
  --connection local
```

## Health Checks

After deployment, the playbook verifies:
- Prometheus is responding on `:9091/-/ready`
- Grafana is responding on `:3000/api/health`
- Alertmanager is responding on `:9093/-/ready`
- Entropy exporter is responding on `:9090/health`

## Security Notes

- All services run as non-root users
- Systemd services use security hardening options
- Change default passwords after deployment
- Configure firewall rules for your environment

## Troubleshooting

**Service won't start:**
```bash
systemctl status prometheus
journalctl -u prometheus -f
```

**Configuration issues:**
```bash
# Validate Prometheus config
promtool check config /etc/prometheus/prometheus.yml

# Validate Alertmanager config
amtool check-config /etc/alertmanager/alertmanager.yml
```

**Re-run specific tasks:**
```bash
ansible-playbook -i inventory/production.yml playbooks/monitoring.yml \
  --tags prometheus \
  --start-at-task="Copy Prometheus configuration"
```
