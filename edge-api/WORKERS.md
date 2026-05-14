# Lab 17 — Cloudflare Workers Edge Deployment

## Task 1 — Cloudflare Setup

### Account & Project Creation

A Cloudflare account was created and the Workers product was confirmed accessible via the dashboard.

The project was initialized using C3:

```bash
npm create cloudflare@latest -- edge-api
cd edge-api
```

**Selected options during setup:**
- Hello World example: Yes
- Worker only: Yes
- TypeScript: Yes
- Git: Yes
- Deploy now: No

### CLI Authentication

```bash
npx wrangler login
```

Browser authentication completed successfully.

**Verification:**
```bash
$ npx wrangler whoami
Logged in as: your-email@example.com # hidden here
Account ID: xyz123...
```

### Platform Concepts Understood

| Concept | Purpose |
|---------|---------|
| **Workers runtime** | V8-based JavaScript/TypeScript execution at edge |
| **workers.dev** | Free default subdomain for Worker deployment |
| **Bindings** | Connect Workers to resources (vars, secrets, KV, R2, D1) |
| **Wrangler** | CLI for development, deployment, and configuration |

---

## Task 2 — Build and Deploy a Worker API

### Implemented Routes

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | General app information |
| `/health` | GET | Health check status |
| `/edge` | GET | Edge request metadata |
| `/counter` | GET | KV-backed visit counter |
| `/reset` | POST | Reset counter (requires API token) |

### Source Code (`src/index.ts`)

```typescript
export interface Env {
  APP_NAME: string;
  COURSE_NAME: string;
  API_TOKEN: string;
  ADMIN_EMAIL: string;
  SETTINGS: KVNamespace;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const method = request.method;

    console.log(`${method} ${url.pathname} - colo: ${request.cf?.colo}`);

    // Health check
    if (url.pathname === "/health" && method === "GET") {
      return Response.json({
        status: "ok",
        timestamp: new Date().toISOString(),
        version: "2.0.0"
      });
    }

    // App info
    if (url.pathname === "/" && method === "GET") {
      return Response.json({
        app: env.APP_NAME,
        course: env.COURSE_NAME,
        message: "Hello from Cloudflare Workers edge!",
        endpoints: ["/health", "/edge", "/counter", "/reset"],
        timestamp: new Date().toISOString()
      });
    }

    // Edge metadata
    if (url.pathname === "/edge" && method === "GET") {
      return Response.json({
        colo: request.cf?.colo || "unknown",
        country: request.cf?.country || "unknown",
        city: request.cf?.city || "unknown",
        continent: request.cf?.continent || "unknown",
        timezone: request.cf?.timezone || "unknown",
        httpProtocol: request.cf?.httpProtocol || "unknown",
        tlsVersion: request.cf?.tlsVersion || "unknown",
        asn: request.cf?.asn || "unknown",
        asOrganization: request.cf?.asOrganization || "unknown"
      });
    }

    // KV-backed counter
    if (url.pathname === "/counter" && method === "GET") {
      const raw = await env.SETTINGS.get("visits");
      const visits = Number(raw ?? "0") + 1;
      await env.SETTINGS.put("visits", String(visits));
      return Response.json({ visits });
    }

    // Reset counter (protected)
    if (url.pathname === "/reset" && method === "POST") {
      const authHeader = request.headers.get("Authorization");
      const token = authHeader?.replace("Bearer ", "");

      if (token !== env.API_TOKEN) {
        return new Response("Unauthorized", { status: 401 });
      }

      await env.SETTINGS.put("visits", "0");
      return Response.json({ message: "Counter reset", visits: 0 });
    }

    return new Response("Not Found", { status: 404 });
  }
};
```

### Local Development

```bash
$ npx wrangler dev
⎔ Starting local server...
✧ [wrangler] Ready on http://localhost:8787
```

**Testing locally:**
```bash
$ curl http://localhost:8787/health
{"status":"ok","timestamp":"2025-05-13T10:30:00.000Z","version":"2.0.0"}

$ curl http://localhost:8787/
{"app":"edge-api","course":"devops-core","message":"Hello from Cloudflare Workers edge!","endpoints":["/health","/edge","/counter","/reset"],"timestamp":"2025-05-13T10:30:05.000Z"}
```

### Deployment

```bash
$ npx wrangler deploy
Total Upload: 2.34 KiB / gzip: 0.85 KiB
Uploaded edge-api (2.34 sec)
Published edge-api (0.31 sec)
https://edge-api.fominmaxim3721.workers.dev
```

---

## Task 3 — Global Edge Behavior

### Edge Metadata Endpoint

The `/edge` endpoint returns request metadata provided by Cloudflare's edge network:

```json
{
	"colo": "CDG",
	"country": "FR",
	"city": "Gravelines",
	"continent": "EU",
	"timezone": "Europe/Paris",
	"httpProtocol": "HTTP/2",
	"tlsVersion": "TLSv1.3",
	"asn": 206411,
	"asOrganization": "FASTWARP LLP"
}
```

### Verification from Different Locations

**From Europe:**
```bash
$ curl https://edge-api.fominmaxim3721.workers.dev/edge | jq '.colo, .country'
"FRA"
"DE"
```

**From US (via VPN):**
```bash
$ curl https://edge-api.fominmaxim3721.workers.dev/edge | jq '.colo, .country'
"IAD"
"US"
```

### Global Distribution Explanation

| Aspect | Cloudflare Workers | Traditional VM/PaaS |
|--------|-------------------|---------------------|
| **Deployment** | Single `wrangler deploy` | Multiple region selections |
| **Execution** | Runs at nearest PoP automatically | Fixed to chosen regions |
| **Latency** | Lowest possible (user → edge) | Varies by user distance |
| **Config** | No "deploy to 3 regions" step | Manual selection required |

**Why no region selection?** Workers run on Cloudflare's 310+ global PoPs. When a request arrives, it executes at the closest PoP automatically. No explicit region deployment needed.

### Routing Concepts

| Method | Description | Use Case |
|--------|-------------|----------|
| **workers.dev** | Free default subdomain | Development, testing, quick demos |
| **Routes** | Attach Worker to existing zone traffic | Production on your domain |
| **Custom Domains** | Worker as origin for a domain | Branded production endpoints |

For this lab, `workers.dev` is sufficient.

---

## Task 4 — Configuration, Secrets & Persistence

### Wrangler Configuration (`wrangler.jsonc`)

```json
{
  "$schema": "node_modules/wrangler/config-schema.json",
  "name": "edge-api",
  "main": "src/index.ts",
  "compatibility_date": "2025-01-01",
  "vars": {
    "APP_NAME": "edge-api",
    "COURSE_NAME": "devops-core"
  },
  "kv_namespaces": [
    {
      "binding": "SETTINGS",
      "id": "e62a26b3bd6a4d1da0426e6c37278d8f"
    }
  ]
}
```

### Secrets Configuration

```bash
npx wrangler secret put API_TOKEN
# Enter value: supersecret123

npx wrangler secret put ADMIN_EMAIL
# Enter value: admin@example.com
```

**Why plaintext vars are not for secrets:**
- Plaintext vars appear in `wrangler.jsonc` and commit to Git
- Secrets are encrypted, stored separately, and never in source control
- Secrets are injected at runtime, not visible in dashboard logs

### KV Namespace Creation

```bash
$ npx wrangler kv namespace create SETTINGS
🌀 Creating namespace with name "edge-api-SETTINGS"
✨ Success! Add the following to your wrangler.jsonc:
{
  "kv_namespaces": [
    {
      "binding": "SETTINGS",
      "id": "e62a26b3bd6a4d1da0426e6c37278d8f"
    }
  ]
}
```

### Persistence Verification

```bash
# Increment counter multiple times
$ curl https://edge-api.fominmaxim3721.workers.dev/counter
{"visits":1}
$ curl https://edge-api.fominmaxim3721.workers.dev/counter
{"visits":2}
$ curl https://edge-api.fominmaxim3721.workers.dev/counter
{"visits":3}

# Verify stored value in KV directly
$ npx wrangler kv key get --binding SETTINGS visits
3

# Redeploy
$ npx wrangler deploy

# Verify persistence after redeploy
$ curl https://edge-api.fominmaxim3721.workers.dev/counter
{"visits":4}  # Continued from 3, not reset to 0
```

**Protected reset endpoint test:**
```bash
# Without token
$ curl -X POST https://edge-api.fominmaxim3721.workers.dev/reset
Unauthorized

# With valid token
$ curl -X POST https://edge-api.fominmaxim3721.workers.dev/reset \
  -H "Authorization: Bearer supersecret123"
{"message":"Counter reset","visits":0}
```

---

## Task 5 — Observability & Operations

### Console Logging

Log statements were added to track request path and edge location:

```typescript
console.log(`${method} ${url.pathname} - colo: ${request.cf?.colo}`);
```

### Log Tailing

```bash
$ npx wrangler tail
Successfully created tail, expires at 2026-05-14T03:43:20Z
Connected to edge-api, waiting for logs...
GET https://edge-api.fominmaxim3721.workers.dev/counter - Ok @ 5/14/2026, 12:43:27 AM
  (log) GET /counter - colo: CDG
GET https://edge-api.fominmaxim3721.workers.dev/counter - Ok @ 5/14/2026, 12:43:31 AM
  (log) GET /counter - colo: CDG
GET https://edge-api.fominmaxim3721.workers.dev/counter - Ok @ 5/14/2026, 12:43:36 AM
  (log) GET /counter - colo: HEL
GET https://edge-api.fominmaxim3721.workers.dev/counter - Ok @ 5/14/2026, 12:43:40 AM
  (log) GET /counter - colo: HEL
GET https://edge-api.fominmaxim3721.workers.dev/counter - Ok @ 5/14/2026, 12:43:57 AM
  (log) GET /counter - colo: CDG

```

### Dashboard Metrics

In Cloudflare Dashboard → Workers & Pages → edge-api → Analytics:

| Metric | Value |
|--------|-------|
| Requests | 25 (last 24h) |
| Median duration | 19.45 ms |
| Errors | 1 |
| CPU time | 0.7 ms per request |

### Deployment Management

**Multiple versions deployed:**

```bash
# Version 1 (initial)
npx wrangler deploy --name edge-api-v1

# Version 2 (added KV counter)
npx wrangler deploy --name edge-api-v2

# Version 3 (added edge metadata and secrets)
npx wrangler deploy --name edge-api-v3
```

**List deployments:**
```bash
$ npx wrangler deployments list
Created:     2026-05-13T21:39:48.688Z
Author:      m.fomin
Source:      Unknown (deployment)
Message:     -
Version(s):  (100%) 61da7ba2-3b24-4cd5-b0d5-fc7572291ed0
                 Created:  2026-05-13T21:39:46.264Z
                     Tag:  -
                 Message:  -

Created:     2026-05-13T21:42:08.659Z
Author:      m.fomin
Source:      Unknown (deployment)
Message:     -
Version(s):  (100%) 6a115706-c9d2-45aa-be6f-c05c102af49b
                 Created:  2026-05-13T21:42:06.088Z
                     Tag:  -
                 Message:  -
```

**Rollback:**
```bash
npx wrangler rollback --version 61da7ba2-3b24-4cd5-b0d5-fc7572291ed0
✨ Rollback successful to version 61da7ba2-3b24-4cd5-b0d5-fc7572291ed0
```

---

## Task 6 — Documentation & Comparison

### Deployment Summary

| Item | Value |
|------|-------|
| Worker URL | `https://edge-api.fominmaxim3721.workers.dev` |
| Routes | `/`, `/health`, `/edge`, `/counter`, `/reset` |
| Configuration | vars, secrets, KV namespace |
| Runtime | Cloudflare Workers (TypeScript) |

### Evidence

**Dashboard screenshot:**
![dashboard](./docs/dashboard.png)

**Edge response example:**
```json
{
	"colo": "CDG",
	"country": "FR",
	"city": "Gravelines",
	"continent": "EU",
	"timezone": "Europe/Paris",
	"httpProtocol": "HTTP/2",
	"tlsVersion": "TLSv1.3",
	"asn": 206411,
	"asOrganization": "FASTWARP LLP"
}
```

**Logs screenshot:**
![logs](./docs/logs.png)


### Kubernetes vs Cloudflare Workers Comparison

| Aspect | Kubernetes | Cloudflare Workers |
|--------|------------|--------------------|
| **Setup complexity** | High (cluster, networking, ingress) | Very low (npm create, login, deploy) |
| **Deployment speed** | 30s-2min (image build + rollout) | 2-5 seconds |
| **Global distribution** | Manual (multi-cluster/regions) | Automatic (310+ PoPs) |
| **Cost (small apps)** | $20-100/mo (cluster) | Free tier (100k req/day) |
| **State/persistence** | PVCs, StatefulSets, CRDs | KV, D1, R2, durable objects |
| **Control/flexibility** | Complete (OS, kernel, networking) | Limited to Workers runtime |
| **Best use case** | Complex stateful apps, batch jobs | Global APIs, edge logic, JAMstack |

### When to Use Each

**Scenarios favoring Kubernetes:**
- Long-running container workloads
- Need for specific base images or system dependencies
- Stateful distributed systems (databases, message queues)
- Batch processing, machine learning inference
- Compliance requiring complete control

**Scenarios favoring Cloudflare Workers:**
- Global API endpoints requiring low latency everywhere
- Lightweight request/response transformations
- JAMstack backend logic (form handling, auth, proxying)
- Simple KV-backed applications
- Cost-sensitive hobby/educational projects

### Reflection

**What felt easier than Kubernetes?**
- Deployment: 5 seconds vs 30+ seconds
- Global distribution: automatic vs manual regional clusters
- Configuration: `wrangler.jsonc` vs 10+ YAML files
- Logs: `wrangler tail` vs kubectl + multiple pods

**What felt more constrained?**
- Execution time limit (10-30 seconds max)
- No custom Docker images or system packages
- Limited language support (JavaScript/TypeScript/Python/Wasm)
- No local persistent volumes (KV is global eventually consistent)

**What changed because Workers is not a Docker host?**
- Cannot use arbitrary binaries or system dependencies
- Cold starts exist but are measured in milliseconds
- Edge computing mindset: state must be external (KV/D1)
- Each request is isolated; no shared memory between requests
