# Lab 02 — Docker Containerization

**Student:** Maxim Fomin
**Email:** m.fomin@innopolis.university

---

## 1. Docker Best Practices Applied

### Multi-Stage Build
```dockerfile
FROM python:3.13-slim AS builder
```
A dedicated builder stage is used to install Python dependencies. This prevents unnecessary build tools and cache files from ending up in the final image, reducing size and attack surface.

---

### Dependency Layer Caching
```dockerfile
COPY requirements.txt .
RUN pip install -r requirements.txt --user --no-cache-dir
```
Dependencies are installed before application code is copied. This allows Docker to reuse cached layers when application code changes, significantly speeding up rebuilds.

---

### Non-Root User
```dockerfile
RUN groupadd groupapp && useradd -r -m -g groupapp userapp
USER userapp
```
Running the application as a non-root user limits the impact of potential vulnerabilities and follows Docker security best practices.

---

### Minimal Base Image
```dockerfile
FROM python:3.13-slim
```
The slim variant reduces image size while remaining compatible with Python dependencies, avoiding the complexity of Alpine-based builds.

---

### Clean Dependency Installation
```dockerfile
--no-cache-dir
```
Disables pip cache storage, preventing unnecessary files from being written to the image.

---

## 2. Image Information & Decisions

- **Base Image:** python:3.13-slim
- **Reason:** Latest stable Python version with a minimal OS footprint
- **Architecture:** Multi-stage (builder → runtime)
- **Final Image Size:** ~120MB (acceptable for Python runtime images)

The builder stage installs dependencies into `/root/.local`, which are then copied into the runtime stage only.

---

## 3. Build & Run Process

### Build
```bash
docker build -t makcal3000/devops_ui:latest .
```
```
DEPRECATED: The legacy builder is deprecated and will be removed in a future release.
            Install the buildx component to build images with BuildKit:
            https://docs.docker.com/go/buildx/

Sending build context to Docker daemon   29.7kB
Step 1/12 : FROM python:3.13-slim AS builder
 ---> 7fda8cfe122c
Step 2/12 : WORKDIR /build
 ---> Using cache
 ---> 40bc42520643
Step 3/12 : COPY requirements.txt .
 ---> Using cache
 ---> 139f95e1634d
Step 4/12 : RUN pip install -r requirements.txt --user --no-cache-dir
 ---> Using cache
 ---> 9179fbb24051
Step 5/12 : FROM python:3.13-slim AS main
 ---> 7fda8cfe122c
Step 6/12 : WORKDIR /app
 ---> Using cache
 ---> 4959bbfffb07
Step 7/12 : RUN groupadd groupapp && useradd -r -m -g groupapp userapp
 ---> Using cache
 ---> 9745758681d2
Step 8/12 : USER userapp
 ---> Using cache
 ---> e3bebb844dc5
Step 9/12 : COPY --from=builder /root/.local /home/userapp/.local
 ---> Using cache
 ---> d094f017cadd
Step 10/12 : COPY . .
 ---> Using cache
 ---> 6948fd538d51
Step 11/12 : EXPOSE 5000
 ---> Using cache
 ---> 3b2ce7bb0145
Step 12/12 : CMD python app.py
 ---> Using cache
 ---> df473f416581
Successfully built df473f416581
Successfully tagged makcal3000/devops_ui:latest
```

### Run
```bash
docker run --rm -ti -p 80:5000 devops_ui:latest
```
```
INFO:     Started server process [7]
INFO:     Waiting for application startup.
INFO:     Application startup complete.
INFO:     Uvicorn running on http://0.0.0.0:5000 (Press CTRL+C to quit)
```

### Endpoint Test
```bash
curl http://localhost:80/
curl http://localhost:80/health
```

---

### Docker Hub Push Evidence

```
docker push makcal3000/devops_ui:latest
The push refers to repository [docker.io/makcal3000/devops_ui]
4ac370e80a97: Layer already exists
d78a288b70b3: Layer already exists
a770c8f86c58: Layer already exists
25c607fbb5d6: Layer already exists
61e0df330e38: Layer already exists
1dfdd9260fd4: Layer already exists
0ae7ca672022: Layer already exists
a8ff6f8cbdfd: Layer already exists
latest: digest: sha256:65b168659b9522eb1f6ac76b0bc33cdd244597144048afdce0ecb4a1bfd04e98 size: 1992
```

**Docker Hub Repository:**
https://hub.docker.com/r/makcal3000/devops_ui

---

## 4. Technical Analysis

### Why Layer Order Matters
If application files were copied before installing dependencies, every code change would invalidate the cache and force a full dependency reinstall.

### Security Considerations
- Non-root execution
- Minimal base image
- Reduced runtime contents via multi-stage build

### .dockerignore Impact
Excluding development artifacts reduces build context size, speeds up builds, and prevents leaking unnecessary files into the image.

---

## 5. Challenges & Solutions

### Issue: Permission Errors with Dependencies
**Cause:** Dependencies installed as root were inaccessible to non-root user.
**Solution:** Installed dependencies with `--user` and copied them explicitly to the non-root user's home directory.

---

### Issue: Image Size Concerns
**Cause:** Python runtime overhead.
**Solution:** Used slim base image and multi-stage build to minimize final image size.

---

## Conclusion

This lab demonstrated a full Docker workflow including secure container design, image optimization, and Docker Hub publishing. The resulting container is production-ready and behaves consistently across environments.
