
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
k8s/mychart/
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
name: my-python-app
type: application
version: 0.1.0
appVersion: "0.1.0"
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
    cpu: 200m
    memory: 256Mi
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
helm install myrelease-dev k8s/mychart -f k8s/mychart/values-dev.yaml
```

Production:

```bash
helm upgrade myrelease-dev k8s/mychart -f k8s/mychart/values-prod.yaml
```

---

## Task 4 — Helm Hooks

Two lifecycle hooks were implemented.

### Pre-install Hook

Purpose:

- Console notification

```yaml
annotations:
  "helm.sh/hook": pre-install
  "helm.sh/hook-weight": "-5"
  "helm.sh/hook-delete-policy": hook-succeeded
```

Execution command:

```bash
kubectl logs job/myrelease-pre-install
```

---

### Post-install Hook

Purpose:

- Console notification

```yaml
annotations:
  "helm.sh/hook": post-install
  "helm.sh/hook-weight": "5"
  "helm.sh/hook-delete-policy": hook-succeeded
```

Verification:

```bash
kubectl get jobs
kubectl logs job/myrelease-post-install
```

Hooks were automatically deleted after successful execution.

---

## Task 5 — Installation Evidence

### Helm Lint

```bash
$ helm lint k8s/mychart
NAME	NAMESPACE	REVISION	UPDATED                                	STATUS  	CHART              	APP VERSION
r   	default  	1       	2026-04-03 00:16:57.333950893 +0300 MSK	deployed	my-python-app-0.1.0	0.1.0
```

---

### Deployment Verification

```bash
$ kubectl get all
NAME                                  READY   STATUS    RESTARTS   AGE
pod/r-my-python-app-9bf7f4bfc-4s496   1/1     Running   0          9m40s
pod/r-my-python-app-9bf7f4bfc-7crb8   1/1     Running   0          9m40s
pod/r-my-python-app-9bf7f4bfc-p2gvj   1/1     Running   0          9m40s

NAME                              TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)        AGE
service/kubernetes                ClusterIP   10.96.0.1       <none>        443/TCP        47m
service/r-my-python-app-service   NodePort    10.99.180.214   <none>        80:30080/TCP   9m40s

NAME                              READY   UP-TO-DATE   AVAILABLE   AGE
deployment.apps/r-my-python-app   3/3     3            3           9m40s

NAME                                        DESIRED   CURRENT   READY   AGE
replicaset.apps/r-my-python-app-9bf7f4bfc   3         3         3       9m40s
```

---

### Hooks output

```
$ kubectl get jobs
NAME                                   STATUS     COMPLETIONS   DURATION   AGE
myrelease-my-python-app-post-install   Complete   1/1           7s         63s
myrelease-my-python-app-pre-install    Complete   1/1           6s         69s

$ kubectl logs jobs/myrelease-my-python-app-pre-install
Pre-install task running
Pre-install completed

$ kubectl logs jobs/myrelease-my-python-app-post-install
Post-install validation
Validation passed
```

---

## Operations

### Install

```bash
helm install myrelease k8s/mychart
```

### Upgrade

```bash
helm upgrade myrelease k8s/mychart -f values-prod.yaml
```

### Rollback

```bash
helm rollback myrelease 1
```

### Uninstall

```bash
helm uninstall myrelease
```

---

## Testing & Validation

Validation steps performed:

- Helm lint passed
```
==> Linting mychart
[INFO] Chart.yaml: icon is recommended

1 chart(s) linted, 0 chart(s) failed
```
- Template rendering verified
```bash
$ helm template test-release mychart | less
```
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
