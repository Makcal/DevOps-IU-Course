# Lab 12 — ConfigMaps & Persistent Volumes

## Task 1 — Application Persistence Upgrade

### Visits Counter Implementation

The application now includes a persistent visit counter with the following features:
- Stores visit count in `/data/visits` file
- Increments counter on each root endpoint request
- Provides `/visits` endpoint to query current count
- Uses atomic write operations for thread safety

### New Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/`       | GET | Returns welcome message and increments visit counter |
| `/visits` | GET | Returns current visit count without incrementing |
| `/health` | GET | Health check endpoint |

### Local Testing with Docker

```bash
max in ~ λ curl localhost:5000/visits
0%
max in ~ λ curl localhost:5000/
{"service":{"name":"devops-info-service","version":"1.0.0","description":"DevOps course info service","framework":"FastAPI"},"system":{"hostname":"2809836b9a8c","platform":"Linux","platform_version":"Debian GNU/Linux 13 (trixie)","architecture":"x86_64","cpu_count":12,"python_version":"3.13.12"},"runtime":{"uptime_seconds":4,"uptime_human":"0 hours, 0 minutes","current_time":"2026-04-16T20:53:24.732585+00:00","timezone":"UTC"},"request":{"client_ip":"172.17.0.1","user_agent":"curl/8.19.0","method":"GET","path":"/"},"endpoints":[{"path":"/","method":"GET","description":"Service information"},{"path":"/health","method":"GET","description":"Health check"}]}%
max in ~ λ curl localhost:5000/visits
1%
```

---

## Task 2 — ConfigMaps

### Configuration File (`files/config.json`)

```json
{
  "app_name": "myapp",
  "environment": "dev",
  "version": "1.0.0",
  "features": {
    "enable_visits": true,
    "rate_limit": 100
  },
  "logging": {
    "level": "info",
    "format": "json"
  }
}
```

### ConfigMap Template

`templates/configmap.yaml`:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "mychart.fullname" . }}-config
  labels:
    {{- include "mychart.labels" . | nindent 4 }}
data:
  config.json: |-
{{ .Files.Get "files/config.json" | indent 4 }}
```

`templates/configmap-env.yaml`:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ include "mychart.fullname" . }}-env
  labels:
    {{- include "mychart.labels" . | nindent 4 }}
data:
  DEBUGMODE: {{ .Values.debugMode | quote }}
  NAMESPACE: {{ .Release.Namespace | quote }}
  POD_NAME: {{ include "mychart.fullname" . }}-{{ .Values.replicaCount }}
```

### Mounting in Deployment

```yaml
volumes:
- name: config-volume
  configMap:
    name: {{ include "mychart.fullname" . }}-config
containers:
- volumeMounts:
  - name: config-volume
    mountPath: /config
    readOnly: true
envFrom:
- configMapRef:
    name: {{ include "mychart.fullname" . }}-env
```

### Verification Outputs

```
$ kubectl exec myrelease-my-python-app-f87f4d84b-4j4qb -- cat /config/config.json
{
  "app_name": "VisitCounterApp",
  "environment": {{ .Values.environment | quote }},
  "version": "1.0.0",
  "features": {
    "enable_visits": true,
    "rate_limit": 100
  },
  "logging": {
    "level": "{{ .Values.logLevel }}",
    "format": "json"
  }
}%
```

---

## Task 3 — Persistent Volumes

### PVC Template (`templates/pvc.yaml`)

```yaml
{{- if .Values.persistence.enabled }}
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: {{ include "mychart.fullname" . }}-data
spec:
  accessModes:
    - {{ .Values.persistence.accessMode }}
  resources:
    requests:
      storage: {{ .Values.persistence.size }}
  {{- if .Values.persistence.storageClass }}
  storageClassName: {{ .Values.persistence.storageClass }}
  {{- end }}
{{- end }}
```

### Volume Mount Configuration

```yaml
volumes:
- name: data-volume
  persistentVolumeClaim:
    claimName: {{ include "mychart.fullname" . }}-data
containers:
- volumeMounts:
  - name: data-volume
    mountPath: /data
```

### Values Configuration

```yaml
persistence:
  enabled: true
  size: 1ki
  storageClass: ""
  mountPath: /data
  accessMode: ReadWriteOnce
```

### Persistence Test Evidence

```
(venv) max in ~/dev/innopolis/devops on lab12 ● ● λ kubectl exec myrelease-my-python-app-f87f4d84b-4j4qb -- cat /
(venv) max in ~/dev/innopolis/devops on lab12 ● ● λ gc -m "feat: do lab 12"
(venv) max in ~/dev/innopolis/devops on lab12 ● ● λ kubectl exec myrelease-my-python-app-f87f4d84b-4j4qb -- cat /data/visits
4%                                                                                                                                                                      (venv) max in ~/dev/innopolis/devops on lab12 ● ● λ kubectl scale deployment/myrelease-my-python-app --replicas=0
deployment.apps/myrelease-my-python-app scaled
(venv) max in ~/dev/innopolis/devops on lab12 ● ● λ kubectl get pods
NAME                                         READY   STATUS      RESTARTS   AGE
myrelease-my-python-app-post-install-g897f   0/1     Completed   0          21m
myrelease-my-python-app-pre-install-d9vm2    0/1     Completed   0          22m
(venv) max in ~/dev/innopolis/devops on lab12 ● ● λ kubectl scale deployment/myrelease-my-python-app --replicas=3
deployment.apps/myrelease-my-python-app scaled
(venv) max in ~/dev/innopolis/devops on lab12 ● ● λ kubectl exec myrelease-my-python-app-f87f4d84b-4j4qb -- cat /data/visits
Error from server (NotFound): pods "myrelease-my-python-app-f87f4d84b-4j4qb" not found
(venv) max in ~/dev/innopolis/devops on lab12 ● ● λ kubectl exec myrelease-my-python-app-f87f4d84b-85cr9 -- cat /data/visits
4%
```

---

## Task 4 — Documentation

### ConfigMap vs Secret

| Aspect | ConfigMap | Secret |
|--------|-----------|--------|
| **Purpose** | Non-sensitive configuration data | Sensitive data (passwords, tokens, keys) |
| **Encoding** | Plain text | Base64 encoded |
| **Use Cases** | Environment settings, feature flags, config files | Passwords, API keys, TLS certificates |
| **Security** | No special protection | Can be encrypted at rest |
| **Logging** | Values may appear in logs | Values are redacted |

### When to use ConfigMap:
- Application configuration parameters
- Feature flags and toggles
- Environment-specific settings
- Non-sensitive configuration files

### When to use Secret:
- Database credentials
- API authentication tokens
- TLS/SSL certificates
- Any sensitive information

### Deployment Commands

```bash
# Deploy with development configuration
helm install visitcounter ./mychart -f values-dev.yaml

# Deploy with production configuration
helm install visitcounter ./mychart -f values-prod.yaml

# Upgrade deployment
helm upgrade visitcounter ./mychart --set logLevel=debug

# Verify all resources
kubectl get all,configmap,pvc -l app.kubernetes.io/instance=visitcounter
```

---

## Bonus Task — ConfigMap Hot Reload (2.5 pts)

### Default Update Behavior

**Observed delay:** ~74 seconds for kubelet sync + cache propagation

```bash
# Monitor config changes
$ while true; do
    kubectl exec pod-name -- cat /config/config.json | grep debug_mode
    sleep 5
  done
```

### subPath Limitation

When using `subPath`, the mounted file is a copy rather than a symlink, preventing auto-updates:

```yaml
# This WON'T auto-update
volumeMounts:
- name: config
  mountPath: /config.json
  subPath: config.json
```

**Solution:** Mount the directory instead:
```yaml
# This WILL auto-update
volumeMounts:
- name: config
  mountPath: /config
```

### Checksum Annotation Pattern

```yaml
metadata:
  annotations:
    checksum/config: {{ include (print $.Template.BasePath "/configmap.yaml") . | sha256sum }}
```

This triggers pod restart when ConfigMap changes during `helm upgrade`.

### Helm Upgrade Pattern

```bash
# Update configuration
$ helm upgrade myrelease ./mychart --set debugMode=true

# Pods automatically restart with new config
$ kubectl get pods -w
NAME                                READY   STATUS        RESTARTS   AGE
myrelease-my-python-app-7b9f8c6d4f-abcde   1/1     Terminating   0          10m
myrelease-my-python-app-7b9f8c6d4f-xyz99   0/1     Pending       0          0s
myrelease-my-python-app-7b9f8c6d4f-xyz99   0/1     Running       0          2s
```
