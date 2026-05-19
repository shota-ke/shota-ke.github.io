+++
date = 2026-05-20
draft = false
title = "Publishing an Ansible Playbook for a Small Slurm Cluster"
description = "A note on cleaning up and publishing an Ansible playbook for a small Slurm cluster, with Docker Compose validation and monitoring screenshots."
tags = ["Slurm", "Ansible", "Docker", "Prometheus", "Grafana", "HPC"]
categories = ["Notes"]
+++

I cleaned up and published an Ansible playbook for provisioning a small Ubuntu-based Slurm cluster.

Repository:

- [shota-ke/slurm_ansible](https://github.com/shota-ke/slurm_ansible)

The playbook started as infrastructure code for a local lab environment. Before publishing it, I removed site-specific inventories, host variables, generated hardware facts, SSH keys, Munge keys, local environment files, and old reference material. The public repository now keeps those files out of Git and includes a Docker Compose environment for validating the roles safely.

## Scope

The repository provisions:

- a Slurm controller,
- CPU and GPU compute nodes,
- Munge authentication,
- SlurmDBD accounting,
- Prometheus,
- Grafana,
- Node Exporter,
- optional DCGM Exporter for GPU telemetry.

The useful part is that the same Ansible structure can be tested locally before touching real machines.

## Docker validation

The local test environment can be started with:

```bash
task dev:up
task dev:deploy
task status ENV=docker
```

The Docker inventory creates one controller and three compute containers:

| Node | Role |
| --- | --- |
| `controller` | Slurm controller, SlurmDBD, Prometheus, Grafana |
| `gpu1` | example GPU compute node |
| `gpu2` | example GPU compute node |
| `cpu1` | example CPU compute node |

The Docker-only SSH and Munge keys are generated under `secrets/`, which is ignored by Git.

## Slurm resource view

I also restored a small reference command named `sres`, which prints a compact view of Slurm node resources. In the Docker cluster, it produced:

```text
┌───────────┬────┬─────────┬──────────────┬──────────────────────┐
│      node │ ST │   CPU   │    memory    │       CPU-JOBS       │
├───────────┼────┼─────────┼──────────────┼──────────────────────┤
│      cpu1 │ 🐟 │  6 / 6  │  23 / 23  GB │       CPU free       │
├───────────┼────┼─────────┼──────────────┼──────────────────────┤
│      gpu1 │ 🐟 │  6 / 6  │  23 / 23  GB │       CPU free       │
├───────────┼────┼─────────┼──────────────┼──────────────────────┤
│      gpu2 │ 🐟 │  6 / 6  │  23 / 23  GB │       CPU free       │
└───────────┴────┴─────────┴──────────────┴──────────────────────┘
```

## Monitoring

Prometheus is configured to scrape the controller and compute nodes.

{{< figure src="prometheus-targets.png" alt="Prometheus targets showing node exporter endpoints are up" caption="Prometheus targets in the Docker validation environment." >}}

Grafana is provisioned with dashboards for the Slurm cluster and node metrics.

{{< figure src="grafana-slurm-dashboard-overview.png" alt="Grafana Slurm dashboard showing node status, CPU usage, and memory usage" caption="Grafana Slurm dashboard rendered from the Docker validation environment." >}}

Keeping the Grafana provisioning files and dashboard JSON in Git makes the monitoring layer easier to reproduce after rebuilding the environment.

## Publishing checklist

The highest-risk part of publishing an Ansible repository is usually not the playbook itself, but the surrounding local state. I checked for:

- SSH private keys,
- copied public keys from real hosts,
- `known_hosts`,
- Munge keys,
- real host variables and IP addresses,
- `.env` files,
- internal URLs embedded in dashboards,
- old reference files.

The repository also has a [security and publishing checklist](https://github.com/shota-ke/slurm_ansible/blob/main/docs/security.md).

## References

- [Repository: shota-ke/slurm_ansible](https://github.com/shota-ke/slurm_ansible)
- [Ansible documentation](https://docs.ansible.com/ansible/latest/)
- [Ansible playbooks](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_intro.html)
- [Ansible roles](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_reuse_roles.html)
- [Slurm Quick Start Administrator Guide](https://slurm.schedmd.com/quickstart_admin.html)
- [Slurm accounting and resource limits](https://slurm.schedmd.com/accounting.html)
- [Slurm GRES scheduling](https://slurm.schedmd.com/gres.html)
- [Docker Compose documentation](https://docs.docker.com/compose/)
- [Taskfile documentation](https://taskfile.dev/docs/getting-started)
- [Prometheus getting started](https://prometheus.io/docs/prometheus/latest/getting_started/)
- [Prometheus Node Exporter guide](https://prometheus.io/docs/guides/node-exporter/)
- [Grafana provisioning documentation](https://grafana.com/docs/grafana/latest/administration/provisioning/)
- [NVIDIA DCGM Exporter documentation](https://docs.nvidia.com/datacenter/dcgm/latest/gpu-telemetry/dcgm-exporter.html)

