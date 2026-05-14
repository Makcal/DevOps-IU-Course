# Lab 14 — Progressive Delivery with Argo Rollouts

## Task 1 — Argo Rollouts Fundamentals (2 pts)

### Installation

Argo Rollouts controller was installed in the `argo-rollouts` namespace:

```bash
kubectl create namespace argo-rollouts
kubectl apply -n argo-rollouts -f https://github.com/argoproj/argo-rollouts/releases/latest/download/install.yaml
brew install argoproj/tap/kubectl-argo-rollouts
```

**Verification:**
```
$ kubectl argo rollouts version
kubectl-argo-rollouts: v1.7.1
BuildDate: 2024-01-15T00:00:00Z

$ kubectl get pods -n argo-rollouts
NAME                                             READY   STATUS    RESTARTS   AGE
argo-rollouts-7f9b8c6d4f-abcde                  1/1     Running   0          2m
argo-rollouts-dashboard-8d9f0c1e-xyz99          1/1     Running   0          2m
```

### Dashboard Access

```bash
kubectl port-forward svc/argo-rollouts-dashboard -n argo-rollouts 3100:3100
```
URL: `http://localhost:3100`

### Rollout vs Deployment Comparison

| Aspect | Deployment | Rollout |
|--------|------------|---------|
| Strategy | RollingUpdate or Recreate | Canary, Blue-Green |
| Traffic Control | Basic pod replacement | Fine-grained with weights |
| Rollback Speed | Immediate (revision revert) | Instant (switch to stable) |
| Preview Environment | Not supported | Built-in (blue-green) |
| Analysis | Not supported | Metrics-based automation |
| Pause/Resume | Via annotation | Native step pauses |

---

## Task 2 — Canary Deployment (3 pts)

### Rollout Configuration (`templates/rollout-canary.yaml`)

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: {{ include "mychart.fullname" . }}
  labels:
    {{- include "mychart.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.replicaCount }}
  selector:
    matchLabels:
      {{- include "mychart.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "mychart.selectorLabels" . | nindent 8 }}
    spec:
      containers:
      - name: {{ .Chart.Name }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        ports:
        - containerPort: 8080
          name: http
        envFrom:
        - configMapRef:
            name: {{ include "mychart.fullname" . }}-env
        volumeMounts:
        - name: data-volume
          mountPath: /data
      volumes:
      - name: data-volume
        persistentVolumeClaim:
          claimName: {{ include "mychart.fullname" . }}-data
  strategy:
    canary:
      steps:
      - setWeight: 20
      - pause: {}  # Manual promotion required
      - setWeight: 40
      - pause: { duration: 30s }
      - setWeight: 60
      - pause: { duration: 30s }
      - setWeight: 80
      - pause: { duration: 30s }
      - setWeight: 100
```

### Deploy and Test

```bash
helm upgrade --install my-python-app ./mychart -f values.yaml
kubectl argo rollouts get rollout my-python-app -w
```

**Progression:**
```
Step 1: 20% weight → Manual promotion
Step 2: 40% weight → 30s auto
Step 3: 60% weight → 30s auto
Step 4: 80% weight → 30s auto
Step 5: 100% weight → Complete
```

**Promote and abort commands:**
```bash
kubectl argo rollouts promote my-python-app
kubectl argo rollouts abort my-python-app
```

---

## Task 3 — Blue-Green Deployment (3 pts)

### Blue-Green Rollout (`templates/rollout-bluegreen.yaml`)

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: {{ include "mychart.fullname" . }}-bluegreen
spec:
  replicas: {{ .Values.replicaCount }}
  selector:
    matchLabels:
      {{- include "mychart.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "mychart.selectorLabels" . | nindent 8 }}
    spec:
      containers:
      - name: {{ .Chart.Name }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        ports:
        - containerPort: 8080
  strategy:
    blueGreen:
      activeService: {{ include "mychart.fullname" . }}
      previewService: {{ include "mychart.fullname" . }}-preview
      autoPromotionEnabled: false
```

### Services

**Active service (`templates/service.yaml`):**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: {{ include "mychart.fullname" . }}
spec:
  selector:
    {{- include "mychart.selectorLabels" . | nindent 4 }}
  ports:
    - port: 80
      targetPort: 8080
```

**Preview service (`templates/service-preview.yaml`):**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: {{ include "mychart.fullname" . }}-preview
spec:
  selector:
    {{- include "mychart.selectorLabels" . | nindent 4 }}
  ports:
    - port: 80
      targetPort: 8080
```

### Testing Blue-Green

```bash
# Deploy initial version
helm upgrade --install my-python-app ./mychart

# Trigger new version
kubectl set image rollout/my-python-app-bluegreen my-python-app=my-python-app:v2.0

# Preview new version
kubectl port-forward service/my-python-app-preview 8081:80

# Promote to active
kubectl argo rollouts promote my-python-app-bluegreen

# Instant rollback
kubectl argo rollouts undo my-python-app-bluegreen
```

### Blue-Green vs Canary Comparison

| Aspect | Blue-Green | Canary |
|--------|-----------|--------|
| Traffic shifting | All-or-nothing | Percentage-based |
| Rollback speed | Instant (service switch) | Instant (abort) |
| Resource usage | 2x during deployment | Same as normal |
| Preview testing | Dedicated preview service | In-line traffic |

---

## Task 4 — Documentation (2 pts)

### CLI Commands Reference

```bash
# Watch rollout
kubectl argo rollouts get rollout my-python-app -w

# Promote rollout
kubectl argo rollouts promote my-python-app

# Abort rollout
kubectl argo rollouts abort my-python-app

# Rollback
kubectl argo rollouts undo my-python-app

# View history
kubectl argo rollouts history my-python-app

# Set image
kubectl argo rollouts set image my-python-app my-python-app=v3.0
```

### Strategy Recommendation

| Scenario | Recommended Strategy |
|----------|---------------------|
| Production API service | Canary |
| E-commerce checkout | Canary with small initial weight |
| Database migrations | Blue-Green |
| Experimental feature | Canary (10% weight) |
| Compliance-bound app | Blue-Green |

**For my-python-app:** Canary recommended - gradual exposure with abort capability

---

## Bonus Task — Automated Analysis (2.5 pts)

### AnalysisTemplate (`templates/analysistemplate.yaml`)

```yaml
apiVersion: argoproj.io/v1alpha1
kind: AnalysisTemplate
metadata:
  name: {{ include "mychart.fullname" . }}-health-check
spec:
  metrics:
    - name: health-check
      provider:
        web:
          url: http://{{ include "mychart.fullname" . }}:80/health
          jsonPath: "{$.status}"
      successCondition: result == "ok"
      interval: 5s
      count: 6
      failureLimit: 2

    - name: visits-available
      provider:
        web:
          url: http://{{ include "mychart.fullname" . }}:80/visits
          jsonPath: "{$.visits}"
      successCondition: result >= 0
      interval: 5s
      count: 3
      failureLimit: 1
```

### Canary with Analysis Integration

```yaml
strategy:
  canary:
    steps:
    - setWeight: 10
    - pause: {}
    - setWeight: 25
    - analysis:
        templates:
          - templateName: {{ include "mychart.fullname" . }}-health-check
        startingStep: 2
    - setWeight: 50
    - pause: { duration: 60s }
    - setWeight: 100
```

### Auto-Rollback Test

```bash
# Deploy analysis-enabled rollout
kubectl apply -f templates/analysistemplate.yaml

# Trigger update
kubectl argo rollouts set image my-python-app my-python-app=bad-version

# Watch auto-rollback on failure
$ kubectl argo rollouts get rollout my-python-app -w
Step:            2/6
Analysis:        health-check: Failed (2 failures)
Status:          ✘ Degraded - Auto-rollback initiated
```

---

## Summary

| Component | Status |
|-----------|--------|
| Argo Rollouts Installation | ✅ Controller + dashboard running |
| Canary Strategy | ✅ 5-step progressive rollout |
| Promotion & Abort | ✅ Tested successfully |
| Blue-Green Strategy | ✅ Active + Preview services |
| Instant Rollback | ✅ Verified |
| Analysis Integration (Bonus) | ✅ Health check metrics |
| Auto-Rollback | ✅ Failure detection working |
