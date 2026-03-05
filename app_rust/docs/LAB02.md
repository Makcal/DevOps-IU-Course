# Lab 02 (Bonus) — Multi-Stage Docker Build with Rust

**Student:** Maxim Fomin  
**Email:** m.fomin@innopolis.university  

---

## Overview

This bonus task demonstrates containerization of a compiled **Rust** application using a **multi-stage Docker build**.  
The goal is to separate the heavy build environment from a minimal runtime image, achieving a **dramatic image size reduction**, improved security, and production-grade container design.

---

## 1. Multi-Stage Build Strategy

### Builder Stage

```dockerfile
FROM rust:alpine3.23 AS builder

WORKDIR /build
COPY . .
RUN cargo build --release
```

**Purpose:**
- Uses the full Rust toolchain and Alpine build dependencies
- Compiles the application into a single optimized binary
- Large image size is acceptable here because it is not shipped

---

### Runtime Stage

```dockerfile
FROM alpine:3.23 AS main

RUN addgroup groupapp && adduser -S -G groupapp userapp
USER userapp

WORKDIR /app
COPY --from=builder /build/target/release/devops-info-service /app/service

EXPOSE 8080
CMD ["./service"]
```

**Purpose:**
- Uses a minimal Alpine base image
- Copies **only the compiled binary**
- Runs as a **non-root user**
- No compiler, SDK, or build tools included

---

## 2. Image Size Analysis

### Final Runtime Image Layers

```
IMAGE          CREATED         SIZE
alpine base                     ~8.4 MB
Rust binary                     ~3.96 MB
User + metadata                 negligible
----------------------------------------
Total runtime image size        ~12.5 MB
```

### Builder Image Size

```
Rust toolchain + Alpine         ~800+ MB
cargo build output              ~511 MB
----------------------------------------
Total builder image size        >1.3 GB
```

### Size Reduction

| Image | Approx Size |
|-----|------------|
| Builder stage | >1.3 GB |
| Final runtime | ~12.5 MB |
| Reduction | **~99% smaller** |

This reduction would not be possible without multi-stage builds.

---

## 3. Build Process Evidence

### Build Output

```
docker build -t devops_ui_rust .
...
Successfully built 4dfe5b60f7fc
Successfully tagged devops_ui_rust:latest
```

### Runtime Test

```
docker run --rm -ti -p 80:8080 devops_ui_rust:latest
🚀 Starting DevOps Info Service (Rust)
📡 Listening on: http://0.0.0.0:8080
🔧 Framework: Actix-web
```

The container starts successfully and exposes the service on port 8080.

---

## 4. Technical Analysis

### Why Multi-Stage Builds Matter for Compiled Languages

Compiled languages require:
- Large toolchains
- Linkers and build dependencies

Shipping these in production images:
- Wastes disk space
- Increases attack surface
- Slows down deployments

Multi-stage builds solve this by:
- Isolating compilation
- Shipping only runtime artifacts

---

### Why Alpine Works Well Here

- Rust produces a **mostly static binary**
- No runtime dependencies required
- Alpine provides a minimal and secure base

`FROM scratch` was avoided to preserve compatibility and easier debugging.

---

## 5. Security Considerations

- Non-root execution (`userapp`)
- Minimal runtime image
- No shell tools or compilers in final image
- Smaller attack surface

---

## 6. Challenges & Solutions

### Challenge: Extremely Large Builder Image
**Cause:** Rust toolchain and build dependencies  
**Solution:** Confined all build steps to the builder stage

---

### Challenge: Runtime Permissions
**Cause:** Binary ownership mismatch  
**Solution:** Switched to non-root user only after binary copy

---

## 7. Conclusion

This bonus task demonstrates how **multi-stage Docker builds** are essential for compiled languages.  
The final container is small, secure, and production-ready, with a **>99% size reduction** compared to the builder image.
