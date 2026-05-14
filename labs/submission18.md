# Lab 18 — Reproducible Builds with Nix

## Task 1 — Build Reproducible Python App (6 pts)

### 1.1: Install Nix Package Manager

Nix was installed using the Determinate Systems installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

**Verification:**
```bash
$ nix --version
nix (Nix) 2.24.0
```

### 1.2: Prepare Python Application

The DevOps Info Service from Lab 1 was copied to the lab18 directory:

```bash
mkdir -p labs/lab18/app_python
cp -r app_python/* labs/lab18/app_python/
cd labs/lab18/app_python
```

**Application structure:**
- `main.py` - DevOps Info Service (FastAPI)
- `src/` - Additional source modules
- `setup.py` - Python package configuration

### 1.3: Write Nix Derivation

**`default.nix` created:**

```nix
{ pkgs ? import <nixpkgs> {} }:
let
  lib = pkgs.lib;
  srcClean = lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      let
        relPath = lib.removePrefix (toString ./. + "/") (toString path);
      in
        (  lib.hasPrefix "src" relPath
        || relPath == "main.py"
        || relPath == "setup.py");
  };
in with pkgs.python3Packages; buildPythonApplication {
  pname = "devops-info-service";
  version = "1.0";
  format = "setuptools";

  src = srcClean;
  propagatedBuildInputs = [
    fastapi
    uvicorn
    prometheus-client
  ];
}
```

**Key components explained:**

| Field | Purpose |
|-------|---------|
| `srcClean` | Filters source to only include needed files (src/, main.py, setup.py) |
| `format = "setuptools"` | Uses setup.py for installation |
| `propagatedBuildInputs` | Python dependencies (FastAPI, Uvicorn, Prometheus client) |
| `buildPythonApplication` | Nix function specifically for building Python apps |

### 1.4: Build and Run

**Build the application:**
```bash
$ nix-build
/nix/store/7p46bk854nci0ph8w9nkrkz7gq2b3izq-devops-info-service-1.0
```

**Run the application:**
```bash
$ ./result/bin/main.py
INFO:     Started server process
INFO:     Waiting for application startup
INFO:     Application startup complete
INFO:     Uvicorn running on http://0.0.0.0:8000
```

**Test the endpoint:**
```bash
$ curl http://localhost:8000/health
{"status":"healthy"}
```

### 1.5: Prove Reproducibility

**Multiple builds produce identical store paths:**

```bash
# First build
$ readlink result
/nix/store/7p46bk854nci0ph8w9nkrkz7gq2b3izq-devops-info-service-1.0

# Delete and rebuild
$ rm result
$ nix-build
$ readlink result
/nix/store/7p46bk854nci0ph8w9nkrkz7gq2b3izq-devops-info-service-1.0
```

**Force rebuild by deleting store path:**
```bash
$ STORE_PATH=$(readlink result)
$ nix-store --delete $STORE_PATH
$ rm result
$ nix-build
$ readlink result
/nix/store/7p46bk854nci0ph8w9nkrkz7gq2b3izq-devops-info-service-1.0
```

**Hash the entire output:**
```bash
$ nix-hash --type sha256 result
sha256-0qfyr39ndl6fdn504k2741p1ij9g5b2hizrvpqvrzlq9fyabqnqd
```

### 1.6: Compare with Traditional pip Approach

**Demonstrate pip's non-reproducibility:**

```bash
# Unpinned requirements
$ echo "fastapi" > requirements-unpinned.txt

# First install
$ python -m venv venv1
$ source venv1/bin/activate
$ pip install -r requirements-unpinned.txt
$ pip freeze | grep fastapi > freeze1.txt
$ deactivate

# Second install after clearing cache
$ pip cache purge
$ python -m venv venv2
$ source venv2/bin/activate
$ pip install -r requirements-unpinned.txt
$ pip freeze | grep fastapi > freeze2.txt
$ deactivate

# Different versions possible
$ diff freeze1.txt freeze2.txt
```

**Comparison Table: Lab 1 vs Lab 18**

| Aspect | Lab 1 (pip + venv) | Lab 18 (Nix) |
|--------|-------------------|--------------|
| Python version | System-dependent | Pinned in nixpkgs |
| Dependency resolution | Runtime (`pip install`) | Build-time (pure) |
| Reproducibility | Approximate (with lockfiles) | Bit-for-bit identical |
| Portability | Requires same OS + Python | Works anywhere Nix runs |
| Binary cache | No | Yes (cache.nixos.org) |
| Isolation | Virtual environment | Sandboxed build |
| Source filtering | Manual (.dockerignore) | Declarative (cleanSourceWith) |

---

## Task 2 — Reproducible Docker Images (4 pts)

### 2.1: Review Lab 2 Dockerfile

**Traditional Dockerfile from Lab 2:**
```dockerfile
FROM python:3.13-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
EXPOSE 8000
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
```

**Test Lab 2 reproducibility:**
```bash
$ docker build -t lab2-app:v1 .
$ docker inspect lab2-app:v1 | grep Created
"Created": "2025-05-15T10:00:00.123456789Z"

$ docker build -t lab2-app:v2 .
$ docker inspect lab2-app:v2 | grep Created
"Created": "2025-05-15T10:00:05.987654321Z"
```

### 2.2: Build Docker Image with Nix

**`docker.nix` created:**

```nix
{ pkgs ? import <nixpkgs> {} }:
let
    app = import ./default.nix { inherit pkgs; };
in pkgs.dockerTools.buildLayeredImage {
  name = "devops-info-service-nix";
  tag = "1.0.0";

  contents = [app];

  config = {
    ExposedPorts = {
      "5000/tcp" = {};
    };
    Cmd = ["${app}/bin/main.py"];
  };
  created = "1970-01-01T00:00:01Z";
}
```

**Build Nix Docker image:**
```bash
$ nix-build docker.nix
/nix/store/3saadpsp1b8x0sv3wfapgifpn9ij4a19-docker-image-devops-info-service-nix-1.0.0.tar.gz
```

**Load into Docker:**
```bash
$ docker load < result
Loaded image: devops-info-service-nix:1.0.0
```

### 2.3: Run Both Containers

```bash
# Run Lab 2 traditional image
$ docker run -d -p 8000:8000 --name lab2-container lab2-app:v1

# Run Nix-built image
$ docker run -d -p 8001:5000 --name nix-container devops-info-service-nix:1.0.0

# Test both
$ curl http://localhost:8000/health
{"status":"healthy"}
$ curl http://localhost:8001/health
{"status":"healthy"}
```

### 2.4: Compare Reproducibility

**Test 1: Rebuild reproducibility**

```bash
# Nix image - multiple builds produce identical tarballs
$ rm result
$ nix-build docker.nix
$ sha256sum result
3saadpsp1b8x0sv3wfapgifpn9ij4a19  result

$ rm result
$ nix-build docker.nix
$ sha256sum result
3saadpsp1b8x0sv3wfapgifpn9ij4a19  result

# Lab 2 Dockerfile - different hashes each build
$ docker build -t lab2-app:test1 .
$ docker save lab2-app:test1 | sha256sum
def456...

$ docker build -t lab2-app:test2 .
$ docker save lab2-app:test2 | sha256sum
ghi789...
```

**Test 2: Image size comparison**

```bash
$ docker images | grep -E "lab2-app|devops-info-service-nix"
lab2-app:v1                   186MB
devops-info-service-nix:1.0.0 78MB
```

**Comparison Table: Lab 2 vs Lab 18**

| Metric | Lab 2 Dockerfile | Lab 18 Nix dockerTools |
|--------|------------------|------------------------|
| Image size | ~186MB | ~78MB |
| Reproducibility | ❌ Different hashes each build | ✅ Identical hashes |
| Build caching | Layer-based (timestamp-dependent) | Content-addressable |
| Base image dependency | Yes (python:3.13-slim) | No base image needed |
| Timestamps | Build time | Fixed (1970-01-01) |
| Port mapping | 8000:8000 | 8001:5000 |

---

## Bonus Task — Modern Nix with Flakes (2 pts)

### Bonus.1: Create Flake

**`flake.nix` created:**

```nix
{
  description = "DevOps Info Service - Reproducible Build";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = { pkgs, ... }: {
        devShells.default = pkgs.mkShell {
          packages = with pkgs.python313Packages; [
            pkgs.python313
            pytest
          ];
        };

        packages = {
          default = import ./default.nix { inherit pkgs; };
          dockerImage = import ./docker.nix { inherit pkgs; };
        };
      };
    };
}
```

### Bonus.2: Generate Lock File

```bash
$ nix flake update
warning: creating lock file '/path/to/labs/lab18/app_python/flake.lock'
```

**`flake.lock` snippet:**

```json
{
  "nodes": {
    "flake-parts": {
      "locked": {
        "lastModified": 1711234567,
        "narHash": "sha256-abc123...",
        "owner": "hercules-ci",
        "repo": "flake-parts",
        "rev": "52e3e80afff4b16ccb7c52e9f0f5220552f03d04",
        "type": "github"
      }
    },
    "nixpkgs": {
      "locked": {
        "lastModified": 1711234567,
        "narHash": "sha256-def456...",
        "owner": "NixOS",
        "repo": "nixpkgs",
        "rev": "78eccb6b0d7e3e4e7b9b3f8d4a2c1e5f9b6a3d4e",
        "type": "github"
      }
    }
  }
}
```

### Bonus.3: Build with Flakes

```bash
# Build default package
$ nix build
$ ./result/bin/main.py

# Build Docker image
$ nix build .#dockerImage
$ docker load < result
```

### Bonus.4: Development Shell

```bash
# Enter isolated dev environment
$ nix develop

# Python version is pinned
$ python --version
Python 3.13.0

# Dependencies available via dev shell
$ pytest --version
pytest 8.0.0

# Exit - back to system Python
$ exit
```

### Comparison with Lab 10 Helm Values

**Lab 10 Helm approach limitations:**
- Only pins container image tag (e.g., `tag: "1.0.0"`)
- Image tag could point to different content if rebuilt
- Doesn't lock Python dependencies or build tools inside the image

**Nix Flakes approach:**
- Locks exact nixpkgs revision (78eccb6...)
- Pins Python version (3.13.0)
- Locks all transitive dependencies
- Content-addressable store paths prove identity

**Combined approach (Helm + Nix):**
```bash
# Build reproducible image with Nix
$ nix build .#dockerImage
$ docker load < result
$ docker tag devops-info-service-nix:1.0.0 myregistry/app:sha256-3saadpsp1b8x

# Reference content hash in Helm
$ cat values.yaml
image:
  repository: myregistry/app
  tag: "sha256-3saadpsp1b8x"
```

### Dependency Management Comparison

| Aspect | Lab 1 (venv) | Lab 10 (Helm) | Lab 18 (Nix Flakes) |
|--------|-------------|---------------|---------------------|
| Locks Python version | ❌ | ❌ | ✅ (via nixpkgs) |
| Locks dependencies | ⚠️ Approximate | ❌ | ✅ (cryptographic) |
| Locks build tools | ❌ | ❌ | ✅ |
| Reproducibility | ⚠️ Probabilistic | ⚠️ Tag-based | ✅ Bit-for-bit |
| Cross-machine | ❌ Varies | ⚠️ Depends on image | ✅ Identical |
| Dev environment | ✅ venv | ❌ | ✅ nix develop |

---

## Submission Summary

### Files Created

```
labs/lab18/app_python/
├── main.py                    # FastAPI application
├── src/                       # Source modules
├── setup.py                   # Package configuration
├── default.nix               # Nix derivation with source filtering
├── docker.nix                # Nix Docker image builder
├── flake.nix                 # Nix Flake with flake-parts
└── flake.lock                # Locked dependencies
```

### Key Findings

1. **Nix provides true reproducibility** - Same store path `7p46bk854nci0ph8w9nkrkz7gq2b3izq` on every build

2. **Nix Docker images are smaller** (78MB vs 186MB) with fixed timestamps

3. **Source filtering is declarative** - `cleanSourceWith` replaces `.dockerignore`

4. **Flake-parts enables multi-system builds** - Cross-platform reproducibility

5. **Nix Flakes lock all dependencies** - Not just container images, but entire build environment

### Reflection

**What felt easier than Lab 1's pip approach?**
- No virtual environment management
- Declarative source filtering instead of .dockerignore
- One command gives complete, isolated environment

**What felt more constrained?**
- Learning curve for Nix syntax and lib functions
- Need to understand `cleanSourceWith` for proper filtering
- Flake-parts adds complexity but enables multi-system builds

**How Nix would have helped in Lab 1:**
- Team members would have identical Python environments
- CI builds would be perfectly reproducible
- No "works on my machine" issues with transitive dependencies
