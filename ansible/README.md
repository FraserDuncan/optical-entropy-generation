# Ansible Deployment

Playbooks for deploying the monitoring stack.

## Structure

```
ansible/
├── inventory/
│   └── hosts.yml
├── playbooks/
│   └── monitoring.yml
├── roles/
│   ├── prometheus/
│   ├── grafana/
│   ├── alertmanager/
│   └── entropy-exporter/
└── README.md
```

## Usage

```bash
# Configure your hosts
cp inventory/hosts.yml inventory/production.yml
# Edit production.yml with your host IPs

# Deploy everything
ansible-playbook -i inventory/production.yml playbooks/monitoring.yml

# Deploy specific components
ansible-playbook -i inventory/production.yml playbooks/monitoring.yml --tags prometheus
ansible-playbook -i inventory/production.yml playbooks/monitoring.yml --tags grafana
ansible-playbook -i inventory/production.yml playbooks/monitoring.yml --tags exporter
```

## Host Groups

| Group | Purpose |
|-------|---------|
| `monitoring` | Prometheus, Grafana, Alertmanager |
| `entropy_generators` | Optical entropy system |
| `development` | Local testing (localhost) |

## Variables

Override in inventory or via `--extra-vars`:

| Variable | Default |
|----------|---------|
| `prometheus_version` | 2.48.0 |
| `grafana_version` | 10.2.0 |
| `alertmanager_version` | 0.26.0 |
| `prometheus_retention_time` | 30d |
| `entropy_exporter_port` | 9090 |

## Local Development

```bash
ansible-playbook -i inventory/hosts.yml playbooks/monitoring.yml \
  --limit development \
  --connection local
```

## Troubleshooting

```bash
systemctl status prometheus
journalctl -u prometheus -f
promtool check config /etc/prometheus/prometheus.yml
```
