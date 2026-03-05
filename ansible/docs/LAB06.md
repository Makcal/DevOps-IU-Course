# Lab 6: Advanced Ansible & CI/CD

## 1. Overview

This lab extends the Ansible automation from Lab 5 with production-ready patterns:

- **Blocks & Tags** – logical task grouping, error handling, and selective execution
- **Docker Compose** – declarative multi-container deployments via Jinja2 templates
- **Role Dependencies** – `web_app` automatically pulls in `docker`
- **Wipe Logic** – double-gated safe cleanup (variable + tag)
- **CI/CD** – GitHub Actions lint and deploy workflow

**Tech Stack:** Ansible 2.16+ | Docker Compose v2 | GitHub Actions | Jinja2

---

## 2. Blocks & Tags

### `common` role (`roles/common/tasks/main.yml`)

| Block | Tag(s) | Purpose |
|-------|--------|---------|
| Install system packages | `packages`, `common` | apt cache + package install, rescue on failure |
| Timezone | `system`, `common` | Set system timezone |
| Directories & reboot check | `directories`, `common` | Create `/opt/app`, `/var/log/app`; reboot notice |

**Rescue block** retries `apt update_cache: yes` (with `cache_valid_time: 0` to force refresh) and then retries the package install if the initial installation fails.  
**Always block** writes a completion log to `/tmp/common_packages_done.log`.

### `docker` role (`roles/docker/tasks/main.yml`)

| Block | Tag(s) | Purpose |
|-------|--------|---------|
| Install Docker | `docker`, `docker_install` | Remove old packages, add GPG key, install Docker CE |
| Configure Docker | `docker`, `docker_config` | Enable service, add user to group, verify version |

**Rescue block** waits 10 s and retries GPG key + package install on network failure.  
**Always block** ensures Docker service is enabled regardless of block success/failure.

### Tag Strategy

```bash
# Run only docker installation
ansible-playbook playbooks/provision.yml --tags "docker_install"

# Skip common role entirely
ansible-playbook playbooks/provision.yml --skip-tags "common"

# Packages across all roles
ansible-playbook playbooks/provision.yml --tags "packages"

# Dry-run docker tasks
ansible-playbook playbooks/provision.yml --tags "docker" --check

# List all available tags
ansible-playbook playbooks/provision.yml --list-tags
```

### Research Answers

**Q: What happens if the rescue block also fails?**  
Ansible marks the task as failed and the play stops (unless `ignore_errors` or `any_errors_fatal: false` is set).

**Q: Can you have nested blocks?**  
Yes, blocks can be nested to arbitrary depth.

**Q: How do tags inherit to tasks within blocks?**  
Tags applied to a block are inherited by all tasks inside it. Tasks can also have their own additional tags.

---

## 3. Docker Compose Migration

### Role Renamed

`app_deploy` → `web_app` for clarity and alignment with multi-app wipe variable naming.

### Template (`roles/web_app/templates/docker-compose.yml.j2`)

```yaml
version: '{{ docker_compose_version | default("3.8") }}'
services:
  {{ app_name }}:
    image: {{ docker_image }}:{{ docker_tag | default("latest") }}
    container_name: {{ app_name }}
    ports:
      - "{{ app_port }}:{{ app_internal_port }}"
    restart: unless-stopped
    networks:
      - app_network
networks:
  app_network:
    driver: bridge
```

Variables are resolved at deploy time from role defaults or playbook `vars_files`.

### Role Dependencies (`roles/web_app/meta/main.yml`)

```yaml
dependencies:
  - role: docker
```

Running `ansible-playbook playbooks/deploy.yml` automatically provisions Docker first.

### Before vs After

| Aspect | Lab 5 (`app_deploy`) | Lab 6 (`web_app`) |
|--------|----------------------|-------------------|
| Deployment unit | `docker_container` module | `docker compose up` |
| Config | Inline Ansible variables | Templated `docker-compose.yml` |
| Dependencies | Manual ordering | Declared in `meta/main.yml` |
| Multi-app | No | Yes (role reuse with different vars) |

### Research Answers

**Q: `restart: always` vs `restart: unless-stopped`?**  
`always` restarts even after explicit `docker stop`; `unless-stopped` respects intentional stops.

**Q: Docker Compose networks vs Docker bridge?**  
Compose automatically creates an isolated project-scoped network; containers on it resolve each other by service name.

**Q: Can Ansible Vault variables be used in templates?**  
Yes – Vault-decrypted variables are available as regular Ansible variables inside Jinja2 templates.

---

## 4. Wipe Logic

### Implementation

| File | Change |
|------|--------|
| `roles/web_app/defaults/main.yml` | `web_app_wipe: false` default |
| `roles/web_app/tasks/wipe.yml` | Docker Compose down + directory removal |
| `roles/web_app/tasks/main.yml` | `include_tasks: wipe.yml` at the top |

**Double gate:** a task only runs when **both** conditions are true:
1. `when: web_app_wipe | bool` – variable must be `true`
2. `tags: [web_app_wipe]` – tag must be selected (or no tag filter given)

### Test Scenarios

| Scenario | Command | Expected Result |
|----------|---------|----------------|
| Normal deploy | `ansible-playbook deploy.yml` | App deployed, wipe skipped |
| Wipe only | `ansible-playbook deploy.yml -e "web_app_wipe=true" --tags web_app_wipe` | App removed, no deploy |
| Clean reinstall | `ansible-playbook deploy.yml -e "web_app_wipe=true"` | Wipe then redeploy |
| Tag only (no var) | `ansible-playbook deploy.yml --tags web_app_wipe` | `when` blocks wipe |

### Research Answers

1. **Why both variable AND tag?** Tag alone could run accidentally; variable alone has no run-time toggle. Together they require explicit intent.
2. **`never` tag vs this approach?** `never` prevents a task from running unless the tag is specified, but cannot be controlled at runtime with `-e`. The variable approach enables scripted clean-reinstalls.
3. **Why wipe BEFORE deploy?** Wipe removes the old state; deploy creates new state. Reversed order would deploy over stale data.
4. **Clean reinstall vs rolling update?** Rolling update minimises downtime; clean reinstall ensures no stale config/data.
5. **Extending to images/volumes?** Add `docker image rm` and `docker volume rm` steps inside the wipe block, gated by additional boolean variables.

---

## 5. CI/CD Integration

### Workflow (`.github/workflows/ansible-deploy.yml`)

```
Push to ansible/ → Lint job → Deploy job → Verify
```

**Lint job** – runs `ansible-lint` on all playbooks.  
**Deploy job** – sets up SSH, writes Vault password from secret, runs `ansible-playbook`, then `curl`s the health endpoint.

### Required GitHub Secrets

| Secret | Purpose |
|--------|---------|
| `ANSIBLE_VAULT_PASSWORD` | Decrypt Vault-encrypted variables |
| `SSH_PRIVATE_KEY` | SSH access to target VM |
| `VM_HOST` | Target VM IP/hostname |

### Path Filters

The workflow only runs when files under `ansible/` change (docs excluded) or the workflow file itself changes, avoiding unnecessary runs on unrelated commits.

### Status Badge

[![Ansible Deployment](https://github.com/Makcal/DevOps-IU-Course/actions/workflows/ansible-deploy.yml/badge.svg)](https://github.com/Makcal/DevOps-IU-Course/actions/workflows/ansible-deploy.yml)

### Research Answers

1. **Security of SSH keys in Secrets?** GitHub Secrets are encrypted at rest and masked in logs, but the key is still exposed to any workflow run. Use short-lived keys or deploy tokens and rotate regularly.
2. **Staging → Production pipeline?** Add a second deploy job that depends on a staging deploy job passing a smoke test.
3. **Rollbacks?** Store the previous image tag as an artifact/output and add a manual workflow dispatch step that runs the playbook with `docker_tag: <previous>`.
4. **Self-hosted runner security?** The runner runs inside your network, never exposing VM credentials outside; the GitHub-hosted runner must SSH in, risking key exfiltration if GitHub were compromised.

---

## 6. Bonus: Multi-App Deployment

The `web_app` role is reused for both the Python and Rust apps by passing different variables.

| App | Playbook | Host Port | Image |
|-----|----------|-----------|-------|
| Python | `deploy_python.yml` | 8000 | `makcal3000/iu-devops-app_python` |
| Rust | `deploy_rust.yml` | 8001 | `makcal3000/iu-devops-app_rust` |

```bash
# Deploy both
ansible-playbook playbooks/deploy_all.yml

# Deploy independently
ansible-playbook playbooks/deploy_python.yml
ansible-playbook playbooks/deploy_rust.yml

# Wipe only Python app
ansible-playbook playbooks/deploy_python.yml -e "web_app_wipe=true" --tags web_app_wipe
```

---

## 7. Challenges & Solutions

| Challenge | Solution |
|-----------|---------|
| `external-managed-environment` pip error | Used `python3-docker` apt package (carried from Lab 5) |
| Docker Compose v2 module vs CLI | Used `docker compose` CLI command inside `command` module for compatibility |
| Wipe running during normal deploy | Double-gate pattern: variable + tag |
| Health check before app is ready | `wait_for` + retry loop on `uri` module |

---

## 8. Key Decisions

- **`include_tasks` for wipe** – keeps `main.yml` clean; tags on `include_tasks` pass through to the included file
- **`tags: [always]` on include_tasks** – ensures wipe tasks are evaluated (but `when` condition blocks execution) even with `--tags app_deploy`
- **`compose_project_dir`** – parameterised so each app lives in its own directory (`/opt/devops-python`, `/opt/devops-rust`)
- **Role meta dependency** – guarantees correct execution order without manual playbook orchestration
