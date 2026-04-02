
# LAB10 — Helm Package Manager

## Overview

In this lab, Kubernetes manifests created in the previous lab were converted into a Helm chart to enable reusable and configurable deployments. The chart supports multiple environments (development and production) and includes lifecycle hooks for installation validation.

---

## Task 1 — Helm Fundamentals

### Helm Installation

Helm CLI was installed locally and verified:

```bash
helm version
```

Helm repositories were added:

```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update
helm search repo prometheus
```

### Helm Value Proposition

Helm simplifies Kubernetes deployments by:

- Providing reusable templates
- Supporting environment-based configuration
- Managing application lifecycle
- Supporting rollbacks and upgrades
- Enabling consistent deployments

---

## Task 2 — Helm Chart Creation

### Chart Structure

```
k8s/myapp/
├── Chart.yaml
├── values.yaml
├── values-dev.yaml
├── values-prod.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── _helpers.tpl
│   └── hooks/
│       ├── pre-install-job.yaml
│       └── post-install-job.yaml
```

### Chart.yaml

```yaml
apiVersion: v2
name: myapp
description: Python FastAPI application Helm chart
type: application
version: 0.1.0
appVersion: "1.0"
```

### Deployment Template Highlights

```yaml
replicas: {{ .Values.replicaCount }}

containers:
  - name: {{ .Chart.Name }}
    image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
```

### Service Template Highlights

```yaml
type: {{ .Values.service.type }}

ports:
  - port: {{ .Values.service.port }}
```

Health probes were preserved and made configurable via values.

---

## Task 3 — Multi-Environment Support

Two environment-specific values files were created.

### values-dev.yaml

```yaml
replicaCount: 1

service:
  type: NodePort

resources:
  limits:
    cpu: 100m
    memory: 128Mi
```

### values-prod.yaml

```yaml
replicaCount: 3

service:
  type: LoadBalancer

resources:
  limits:
    cpu: 500m
    memory: 512Mi
```

### Deployment Commands

Development:

```bash
helm install myapp-dev k8s/myapp -f k8s/myapp/values-dev.yaml
```

Production:

```bash
helm upgrade myapp-dev k8s/myapp -f k8s/myapp/values-prod.yaml
```

---

## Task 4 — Helm Hooks

Two lifecycle hooks were implemented.

### Pre-install Hook

Purpose:

- Validate environment readiness
- Simulate migration or dependency check

```yaml
annotations:
  "helm.sh/hook": pre-install
  "helm.sh/hook-weight": "-5"
  "helm.sh/hook-delete-policy": hook-succeeded
```

Execution command:

```bash
kubectl logs job/myapp-pre-install
```

---

### Post-install Hook

Purpose:

- Run smoke test
- Verify application availability

```yaml
annotations:
  "helm.sh/hook": post-install
  "helm.sh/hook-weight": "5"
  "helm.sh/hook-delete-policy": hook-succeeded
```

Verification:

```bash
kubectl get jobs
kubectl logs job/myapp-post-install
```

Hooks were automatically deleted after successful execution.

---

## Task 5 — Installation Evidence

### Helm Lint

```bash
helm lint k8s/myapp
```

Chart passed validation without errors.

---

### Template Rendering

```bash
helm template myapp k8s/myapp
```

Templates rendered successfully.

---

### Dry Run

```bash
helm install --dry-run --debug myapp k8s/myapp
```

All Kubernetes resources were generated correctly.

---

### Deployment Verification

```bash
helm list
kubectl get all
```

Example output:

```
NAME        READY   STATUS    RESTARTS
myapp-xxx   1/1     Running   0
```

---

## Operations

### Install

```bash
helm install myapp k8s/myapp
```

### Upgrade

```bash
helm upgrade myapp k8s/myapp -f values-prod.yaml
```

### Rollback

```bash
helm rollback myapp 1
```

### Uninstall

```bash
helm uninstall myapp
```

---

## Testing & Validation

Validation steps performed:

- Helm lint passed
- Template rendering verified
- Dry-run executed successfully
- Application accessible via Kubernetes service
- Hook jobs executed correctly
- Multi-environment configuration validated

---

## Summary

This lab demonstrated how to:

- Convert Kubernetes manifests into Helm templates
- Configure environment-specific deployments
- Implement lifecycle hooks
- Manage Helm release lifecycle
- Validate chart functionality

The Helm chart is reusable, configurable, and suitable for deployment across multiple environments.
