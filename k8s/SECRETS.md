
# LAB11 — Kubernetes Secrets & Vault Report

## Kubernetes Secrets

Secret created:

```bash
kubectl create secret generic my-secret --from-literal=key=value
```

Viewed:

```
$ kubectl get secret my-secret -o yaml
apiVersion: v1
data:
  key: dmFsdWU=
kind: Secret
metadata:
  creationTimestamp: "2026-04-09T19:36:11Z"
  name: my-secret
  namespace: default
  resourceVersion: "7339"
  uid: e04dcfda-06f8-4bfd-aebc-43a3608d7893
type: Opaque
```

Decoded base64 successfully.

Explanation:

Base64 encoding is NOT encryption.
Secrets must be protected with RBAC and etcd encryption.

---

## Helm Secret Integration

Secret template created.

Deployment updated with:

```yaml
envFrom:
- secretRef:
    name: {{ include "mychart.fullname" . }}-secret
```

Secrets injected successfully.
```
$ kubectl exec myrelease-my-python-app-87977bb78-9fgjb -- bash -c 'echo $username'
admin
```

---

## Resource Limits

Configured:

```yaml
requests:
  cpu: 100m

limits:
  cpu: 200m
```

Requests guarantee minimum.
Limits restrict maximum.

---

## Vault Installation

```bash
helm install vault hashicorp/vault --set server.dev.enabled=true --set injector.enabled=true
```

Pods running successfully.

---

## Vault Secret Injection

Vault configured.

Policy created.

Role created.

Secrets injected into `/vault/secrets`

Verified inside container.

```
$ kubectl get pods
NAME                                         READY   STATUS      RESTARTS   AGE
myrelease-my-python-app-69c7b97649-b42tp     2/2     Running     0          87s
myrelease-my-python-app-69c7b97649-gxsh5     2/2     Running     0          105s
myrelease-my-python-app-69c7b97649-tjkfn     2/2     Running     0          96s
myrelease-my-python-app-post-install-8v42j   0/1     Completed   0          6d23h
myrelease-my-python-app-pre-install-9f8r7    0/1     Completed   0          6d23h
vault-0                                      1/1     Running     0          25m
vault-agent-injector-848dd747d7-k5j26        1/1     Running     0          25m
$ kubectl exec myrelease-my-python-app-69c7b97649-b42tp -- ls /vault/secrets
Defaulted container "my-python-app" out of: my-python-app, vault-agent, vault-agent-init (init)
config
```

---

## Sidecar Pattern

Vault Agent runs alongside app container.

It retrieves secrets securely
and stores them in shared volume.

---

## Security Analysis

Kubernetes Secrets:

Simple but encoded only.

Vault:

Encrypted, centralized, production-ready.

---

## Summary

This lab demonstrated:

- Kubernetes Secrets usage
- Helm secret integration
- Vault deployment
- Secure secret injection
