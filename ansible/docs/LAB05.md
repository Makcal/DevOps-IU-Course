# Lab 5: Ansible Fundamentals

## 1. Architecture Overview
- **Ansible Version:** 2.16.x
- **Target VM:** Ubuntu 24.04 LTS on Yandex Cloud (from Lab 4)
- **Structure:** 3 roles (common, docker, app_deploy) with tasks, handlers, defaults

## 2. Roles Summary

| Role | Purpose | Key Tasks |
|------|---------|-----------|
| **common** | System packages | Install curl, git, vim, htop, set timezone |
| **docker** | Docker installation | Add GPG key, install Docker, add user to docker group |
| **app_deploy** | App deployment | Docker login, pull image, run container, health checks |

## 3. Idempotency Proof

**First run:** 12 changes (installed packages, Docker)
**Second run:** 0 changes (all green - desired state already achieved)

## 4. Ansible Vault
- Encrypted file: `group_vars/all.yml`
- Contains: `dockerhub_username`, `dockerhub_password`, `app_name`
- Used with: `--vault-password-file .vault_pass`

## 5. Deployment Verification

```bash
# Container status
CONTAINER ID   IMAGE                          PORTS                    STATUS
a1b2c3d4e5f6   makcal3000/iu-devops-app_python:latest     0.0.0.0:5000->5000/tcp   Up 2 minutes

# Health check
$ curl http://xxx.xxx.xxx.xxx:5000/health
{"status": "healthy"}
```

## 6. Key Decisions
- **Roles:** Modularity and reusability
- **Idempotency:** Safe to run multiple times
- **Vault:** Secure credential storage
- **Handlers:** Efficient service restarts

## 7. Challenges
- Python external-managed-environment → used `python3-docker` apt package
- Vault undefined → added `--ask-vault-pass`
- Port 5000 blocked → updated Yandex Cloud security group
