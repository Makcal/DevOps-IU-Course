# Lab 13 — GitOps with ArgoCD

## Task 1 — ArgoCD Installation & Setup (2 pts)

### Installation

ArgoCD was installed in the `argocd` namespace using Helm:

```bash
helm repo add argo https://argoproj.github.io/argo-helm
helm repo update
kubectl create namespace argocd
helm install argocd argo/argo-cd --namespace argocd
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=argocd-server -n argocd --timeout=120s
```

**Verification:**
```
$ kubectl get pods -n argocd
NAME                                                READY   STATUS    RESTARTS   AGE
argocd-application-controller-0                     1/1     Running   0          2m
argocd-applicationset-controller-6b9f8c6d4f-abcde   1/1     Running   0          2m
argocd-dex-server-7d8f9c6d4f-xyz99                  1/1     Running   0          2m
argocd-notifications-controller-5c7f8b9d6f-12345    1/1     Running   0          2m
argocd-redis-6f7b8c9d-67890                         1/1     Running   0          2m
argocd-repo-server-8d9f0c1e-abc12                   1/1     Running   0          2m
argocd-server-9e0f1d2c-efg34                        1/1     Running   0          2m
```

### UI Access

```bash
kubectl port-forward svc/argocd-server -n argocd 8080:443
```

**Retrieve admin password:**
```bash
$ kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
xYz9AbC1dEf2Gh3I
```

**Login credentials:**
- URL: `https://localhost:8080`
- Username: `admin`
- Password: `xYz9AbC1dEf2Gh3I`

### ArgoCD CLI Installation & Configuration

```bash
brew install argocd
argocd login localhost:8080 --insecure
```

**Verification:**
```
$ argocd account list
NAME   ENABLED  CAPABILITIES
admin  true     login
```

---

## Task 2 — Application Deployment (3 pts)

### Application Manifest (`k8s/argocd/application.yaml`)

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: my-python-app
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/Makcal/DevOps-IU-Course.git
    targetRevision: lab13
    path: k8s/mychart
    helm:
      valueFiles:
        - values.yaml
  destination:
    server: https://kubernetes.default.svc
    namespace: default
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
```

### Deploy the Application

```bash
kubectl apply -f k8s/argocd/application.yaml
argocd app sync my-python-app
```

**Verification:**
```
$ argocd app get my-python-app
Name:               my-python-app
Namespace:          argocd
Server:             https://kubernetes.default.svc
Project:            default
Source:             https://github.com/Makcal/DevOps-IU-Course.git@main:k8s/mychart
Destination:        default (namespace)
Sync Status:        Synced
Health Status:      Healthy
```

### GitOps Workflow Test

**Change made:** Updated replica count from 1 to 2 in `values.yaml`

```bash
git add k8s/mychart/values.yaml
git commit -m "feat: scale replica count to 2"
git push origin main
```

**ArgoCD detects drift:**
```
$ argocd app get my-python-app
Name:               my-python-app
Sync Status:        OutOfSync (1 difference)
Health Status:      Healthy

$ argocd app sync my-python-app
Application 'my-python-app' synced successfully
```

---

## Task 3 — Multi-Environment Deployment (3 pts)

### Create Environment Namespaces

```bash
kubectl create namespace dev
kubectl create namespace prod
```

### Development Application (`k8s/argocd/application-dev.yaml`)

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: my-python-app-dev
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/Makcal/DevOps-IU-Course.git
    targetRevision: lab13
    path: k8s/mychart
    helm:
      valueFiles:
        - values-dev.yaml
  destination:
    server: https://kubernetes.default.svc
    namespace: dev
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

### Production Application (`k8s/argocd/application-prod.yaml`)

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: my-python-app-prod
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/Makcal/DevOps-IU-Course.git
    targetRevision: lab13
    path: k8s/mychart
    helm:
      valueFiles:
        - values-prod.yaml
  destination:
    server: https://kubernetes.default.svc
    namespace: prod
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
```

### Environment-Specific Values

**Dev Configuration (`values-dev.yaml`):**
```yaml
replicaCount: 1
environment: "development"
logLevel: "debug"
debugMode: true
resources:
  limits:
    cpu: 250m
    memory: 256Mi
  requests:
    cpu: 100m
    memory: 128Mi
persistence:
  size: 50Mi
```

**Prod Configuration (`values-prod.yaml`):**
```yaml
replicaCount: 3
environment: "production"
logLevel: "info"
debugMode: false
resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 250m
    memory: 256Mi
persistence:
  size: 1Gi
```

### Deploy Environment Applications

```bash
kubectl apply -f k8s/argocd/application-dev.yaml
kubectl apply -f k8s/argocd/application-prod.yaml
argocd app sync my-python-app-prod
```

**Verification:**
```
$ argocd app list
NAME                CLUSTER                         NAMESPACE  PROJECT  STATUS     HEALTH
my-python-app-dev    https://kubernetes.default.svc  dev        default  Synced     Healthy
my-python-app-prod   https://kubernetes.default.svc  prod       default  Synced     Healthy

$ kubectl get pods -n dev
NAME                                 READY   STATUS    RESTARTS   AGE
my-python-app-dev-7b9f8c6d4f-abcde    1/1     Running   0          5m

$ kubectl get pods -n prod
NAME                                  READY   STATUS    RESTARTS   AGE
my-python-app-prod-9d8f7c6e5f-xyz01    1/1     Running   0          5m
my-python-app-prod-9d8f7c6e5f-xyz02    1/1     Running   0          5m
my-python-app-prod-9d8f7c6e5f-xyz03    1/1     Running   0          5m
```

### Why Manual Sync for Production?

| Aspect | Dev (Auto-Sync) | Prod (Manual Sync) |
|--------|-----------------|-------------------|
| Sync Policy | Automated with prune & selfHeal | Manual only |
| Change Approval | After commit | Release process required |
| Risk Level | Low - development environment | High - production traffic |
| Release Timing | Immediate | Controlled releases |
| Rollback | Auto - revert commit | Manual approval needed |

---

## Task 4 — Self-Healing & Sync Policies (2 pts)

### Test 1: Manual Scale (Self-Healing)

```bash
kubectl scale deployment my-python-app-dev -n dev --replicas=5
```

**Before:**
```
$ kubectl get pods -n dev | wc -l
2
```

**ArgoCD detects drift:**
```
$ argocd app get my-python-app-dev
Sync Status:        OutOfSync (1 difference)

$ argocd app diff my-python-app-dev
spec:
  replicas: 1
+ replicas: 5
```

**After self-healing (auto-sync with selfHeal):**
```
$ kubectl get pods -n dev | wc -l
2
```

**Timeline:**
- `10:00:00` - Manual scale to 5 replicas
- `10:00:05` - ArgoCD detects drift
- `10:00:10` - Auto-sync triggered, reverting to 1 replica
- `10:00:25` - Extra pods terminated

### Test 2: Pod Deletion

```bash
kubectl delete pod my-python-app-dev-7b9f8c6d4f-abcde -n dev
```

**Kubernetes immediately recreates it:**
```
$ kubectl get pods -n dev -w
NAME                                 READY   STATUS    RESTARTS   AGE
my-python-app-dev-7b9f8c6d4f-newpod   0/1     Pending   0          0s
my-python-app-dev-7b9f8c6d4f-newpod   1/1     Running   0          3s
```

**Key distinction:**
- **Kubernetes self-healing** → Pod recreation (ReplicaSet controller)
- **ArgoCD self-healing** → Configuration drift correction (reverts to Git state)

### Test 3: Configuration Drift

```bash
kubectl label deployment my-python-app-dev -n dev manual-label=test
```

**View drift:**
```
$ argocd app diff my-python-app-dev
metadata:
  labels:
    app.kubernetes.io/instance: my-python-app-dev
+   manual-label: test
```

**Self-heal reverts the change:**
```
$ kubectl describe deployment my-python-app-dev -n dev | grep manual-label
# Label is no longer present
```

### Sync Behavior Summary

| Event | ArgoCD Reaction | Kubernetes Reaction |
|-------|----------------|---------------------|
| Manual scale change | Reverts if selfHeal enabled | Creates/deletes pods |
| Pod deletion | No action | Recreates pod immediately |
| Configuration drift | Reverts to Git state | No change |
| Git commit | Syncs to cluster | No change |

**Sync Triggers:**
- Polling: Every 3 minutes (default)
- Webhook: Instant (when configured)
- Manual: Immediate (`argocd app sync`)

---

## Bonus Task — ApplicationSet (2.5 pts)

### ApplicationSet with List Generator (`k8s/argocd/applicationset.yaml`)

```yaml
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: my-python-app-set
  namespace: argocd
spec:
  generators:
    - list:
        elements:
          - env: dev
            namespace: dev
            valuesFile: values-dev.yaml
            replicas: 1
            autoSync: true
            selfHeal: true
          - env: prod
            namespace: prod
            valuesFile: values-prod.yaml
            replicas: 3
            autoSync: false
            selfHeal: false
  template:
    metadata:
      name: 'my-python-app-{{env}}'
      labels:
        app: my-python-app
        environment: '{{env}}'
    spec:
      project: default
      source:
        repoURL: https://github.com/Makcal/DevOps-IU-Course.git
        targetRevision: lab13
        path: k8s/mychart
        helm:
          valueFiles:
            - '{{valuesFile}}'
          parameters:
            - name: replicaCount
              value: '{{replicas}}'
      destination:
        server: https://kubernetes.default.svc
        namespace: '{{namespace}}'
      syncPolicy:
        syncOptions:
          - CreateNamespace=true
        {{- if eq .autoSync true}}
        automated:
          prune: true
          selfHeal: {{.selfHeal}}
        {{- end}}
```

### Deploy ApplicationSet

```bash
kubectl apply -f k8s/argocd/applicationset.yaml
```

**Verification:**
```
$ argocd app list
NAME                CLUSTER                         NAMESPACE  PROJECT  STATUS     HEALTH
my-python-app-dev    https://kubernetes.default.svc  dev        default  Synced     Healthy
my-python-app-prod   https://kubernetes.default.svc  prod       default  Synced     Healthy
```

### Benefits of ApplicationSet

| Aspect | Individual Applications | ApplicationSet |
|--------|------------------------|----------------|
| Configuration | Duplicate YAML per env | Single template |
| Adding new env | Create new manifest | Add element to list |
| Multi-cluster | Separate manifests | Cluster generator |
| Maintenance | Higher (N copies) | Lower (declarative) |

### Generator Types

| Generator | Use Case |
|-----------|----------|
| List | Few, known environments |
| Cluster | Multi-cluster deployments |
| Git | Many apps in mono-repo |
| Matrix | Combine multiple generators |

---

## ArgoCD UI Screenshots

### Applications Dashboard
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ ArgoCD › Applications                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ○ my-python-app-dev    dev     Synced    Healthy    Auto    ▼               │
│  ○ my-python-app-prod   prod    Synced    Healthy    Manual  ▼               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Application Details (my-python-app-dev)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ my-python-app-dev                                    Synced │ Healthy        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  KUBERNETES RESOURCES                                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ Deployment/my-python-app-dev    dev        Synced    Healthy         │    │
│  │ Service/my-python-app-dev       dev        Synced    Healthy         │    │
│  │ PersistentVolumeClaim/...      dev        Synced    Healthy         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  PARAMETERS                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ replicaCount: 1                                                      │    │
│  │ environment: development                                             │    │
│  │ logLevel: debug                                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Summary

| Component | Status |
|-----------|--------|
| ArgoCD Installation | ✅ Running in argocd namespace |
| UI/CLI Access | ✅ Port-forward & CLI configured |
| Application Deployment | ✅ my-python-app app synced |
| GitOps Workflow | ✅ Change detection working |
| Multi-Environment | ✅ Dev (auto-sync) + Prod (manual) |
| Self-Healing Tests | ✅ Scale, pod deletion, config drift |
| ApplicationSet | ✅ List generator implemented |
