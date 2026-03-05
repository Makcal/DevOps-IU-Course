# Lab 6: Advanced Ansible & CI/CD

## 1. Overview

Building on Lab 5, this lab enhances the Ansible automation with:
- **Blocks & Tags** for error handling and selective execution
- **Docker Compose** templating (replacing `docker run`)
- **Role dependencies** (`web_app` depends on `docker`)
- **Wipe logic** with double-gating (variable + tag)
- **GitHub Actions CI/CD** for automated Ansible deployments

**Tech Stack:** Ansible 2.16+ | Docker Compose v2 | GitHub Actions | Jinja2

---

## 2. Blocks & Tags

### common role (`roles/common/tasks/main.yml`)

| Block | Tag | Purpose |
|-------|-----|---------|
| Package installation | `packages` | apt update, install packages, set timezone |
| User management | `users` | create directories, check reboot flag |

**Block pattern used:**
```yaml
- name: Package installation block
  block:
    - name: Update apt cache
      apt: ...
    - name: Install common packages
      apt: ...
  rescue:
    - name: Retry apt cache update with fix-missing
      apt: update_cache=yes fix_missing=yes
  always:
    - name: Log completion
      copy: ...
  become: true
  tags: [packages]
```

### docker role (`roles/docker/tasks/main.yml`)

| Block | Tag | Purpose |
|-------|-----|---------|
| Docker installation | `docker_install` | Remove old, add GPG key, install packages |
| Docker configuration | `docker_config` | Add user to group, test install |

**Rescue block** waits 10 seconds and retries GPG key addition on network failure.
**Always block** ensures Docker service is enabled regardless of success/failure.

### Selective Execution Examples

```bash
# Run only docker installation
ansible-playbook playbooks/provision.yml --tags "docker"

# Skip common role
ansible-playbook playbooks/provision.yml --skip-tags "common"

# Install packages only across all roles
ansible-playbook playbooks/provision.yml --tags "packages"

# Run only docker installation tasks (not config)
ansible-playbook playbooks/provision.yml --tags "docker_install"

# List all available tags
ansible-playbook playbooks/provision.yml --list-tags
```

### Research Answers

**Q: What happens if rescue block also fails?**
If the rescue block fails, Ansible marks the host as failed and stops execution for that host. The `always` block still runs.

**Q: Can you have nested blocks?**
Yes, blocks can be nested to any depth. Inner block failures are handled by the inner rescue/always before propagating upwards.

**Q: How do tags inherit to tasks within blocks?**
Tags applied to a block are inherited by all tasks inside that block. Tasks can also have their own additional tags, which combine with the block's tags.

---

## 3. Docker Compose Migration

### Why Docker Compose?

- **Declarative** — define desired state in YAML, not imperative commands
- **Multi-container** — manage networks, volumes, and service dependencies
- **Idempotent** — `docker compose up` is safe to run multiple times
- **Easier updates** — modify the compose file and re-run

### Template: `roles/web_app/templates/docker-compose.yml.j2`

```yaml
version: '3.8'

services:
  {{ app_name }}:
    image: {{ docker_image }}:{{ docker_tag }}
    container_name: {{ app_name }}
    ports:
      - "{{ app_port }}:{{ app_internal_port }}"
    environment:
      APP_ENV: production
      APP_PORT: "{{ app_internal_port }}"
    restart: unless-stopped
    networks:
      - app_network

networks:
  app_network:
    driver: bridge
```

### Role rename: `app_deploy` → `web_app`

The role was renamed for clarity and to support multi-app patterns (other roles like `database_app`, `cache_app` could follow the same pattern).

### Role Dependencies (`roles/web_app/meta/main.yml`)

```yaml
dependencies:
  - role: docker
```

This ensures Docker is always installed before the `web_app` role runs, even when deploying with `ansible-playbook playbooks/deploy.yml` without explicitly including the `docker` role.

### Key Variables (`roles/web_app/defaults/main.yml`)

| Variable | Default | Description |
|----------|---------|-------------|
| `app_name` | `devops-app` | Container/service name |
| `docker_image` | `makcal3000/iu-devops-app_python` | Docker Hub image |
| `docker_tag` | `latest` | Image version tag |
| `app_port` | `8000` | Host port |
| `app_internal_port` | `8000` | Container port |
| `compose_project_dir` | `/opt/{{ app_name }}` | Directory on target host |
| `web_app_wipe` | `false` | Wipe control variable |

### Research Answers

**Q: `restart: always` vs `restart: unless-stopped`?**
`unless-stopped` skips auto-start if the container was manually stopped, while `always` always restarts. `unless-stopped` is preferred for production because it respects intentional maintenance stops.

**Q: Docker Compose networks vs Docker bridge networks?**
Compose creates a named project-scoped network by default, enabling service-to-service DNS by container name. Manual bridge networks require explicit `--network` flags.

**Q: Can you reference Ansible Vault variables in the template?**
Yes. Vault-decrypted variables are available in templates just like regular variables. The Vault password must be provided when running the playbook.

---

## 4. Wipe Logic

### Implementation

The wipe logic is controlled by **two gates**:
1. **Variable gate:** `when: web_app_wipe | bool` — defaults to `false`
2. **Tag gate:** `tags: [web_app_wipe]` — must explicitly run with `--tags web_app_wipe`

Both conditions must be true for wipe tasks to execute.

### Usage Scenarios

**Scenario 1: Normal deployment (wipe does NOT run)**
```bash
ansible-playbook playbooks/deploy.yml
# wipe tasks are included but skipped (tag not specified)
```

**Scenario 2: Wipe only**
```bash
ansible-playbook playbooks/deploy.yml \
  -e "web_app_wipe=true" \
  --tags web_app_wipe
# app removed, deployment skipped
```

**Scenario 3: Clean reinstallation (wipe → deploy)**
```bash
ansible-playbook playbooks/deploy.yml \
  -e "web_app_wipe=true"
# wipe runs first, then fresh deployment
```

**Scenario 4a: Tag specified, variable false (blocked by when)**
```bash
ansible-playbook playbooks/deploy.yml --tags web_app_wipe
# wipe skipped (when: web_app_wipe is false)
```

### Research Answers

1. **Why use both variable AND tag?** Variable prevents accidental wipe when `--tags all` is used. Tag prevents accidental wipe when `web_app_wipe=true` is set in group_vars. Both must be explicitly set.

2. **`never` tag vs this approach?** The `never` tag requires `--tags never` which also runs other never-tagged tasks. Our approach uses a named tag (`web_app_wipe`) that is specific and explicit.

3. **Why wipe before deployment?** Enables clean reinstallation: wipe removes stale state, then deployment creates a fresh install.

4. **Clean reinstall vs rolling update?** Clean reinstall when migrating major versions or fixing corrupted state. Rolling update when deploying new versions of the same app with no downtime.

5. **Extending to wipe Docker images and volumes?**
```yaml
- name: Remove Docker image
  community.docker.docker_image:
    name: "{{ docker_image }}:{{ docker_tag }}"
    state: absent
  when: web_app_wipe | bool
```

---

## 5. CI/CD Integration

### Workflow: `.github/workflows/ansible-deploy.yml`

**Triggers:** Push/PR to `main`/`master` affecting `ansible/**` files.

**Jobs:**

| Job | Steps |
|-----|-------|
| `lint` | Checkout → Install ansible + ansible-lint → Run lint |
| `deploy` | Checkout → Install Ansible → Setup SSH → Run playbook → Verify |

**Path filters** prevent running on documentation changes:
```yaml
paths:
  - 'ansible/**'
  - '!ansible/docs/**'
  - '.github/workflows/ansible-deploy.yml'
```

### Required GitHub Secrets

| Secret | Purpose |
|--------|---------|
| `SSH_PRIVATE_KEY` | SSH key to connect to target VM |
| `VM_HOST` | Target VM IP/hostname |
| `ANSIBLE_VAULT_PASSWORD` | Ansible Vault decryption password |

### Research Answers

1. **Security implications of SSH keys in GitHub Secrets?** Secrets are encrypted at rest and masked in logs. Risk: anyone with write access to the repo can create workflows that expose secrets. Use environment protection rules and limit who can trigger deployments.

2. **Staging → Production pipeline?** Add separate environments in GitHub Actions with approval gates. Use different inventories (`inventory/staging.ini`, `inventory/production.ini`) and require manual approval before production deployment.

3. **Making rollbacks possible?** Tag Docker images with git SHA (`docker_tag: ${{ github.sha }}`). Keep previous 3 images. Add a rollback workflow that redeploys with a specified tag.

4. **Self-hosted runner security?** The runner runs on your own infrastructure, so credentials never leave your network. No need to push SSH keys to GitHub Secrets. However, the runner itself must be secured.

---

## 6. Testing Results

### ansible-lint

All playbooks pass ansible-lint checks:
```
$ ansible-lint playbooks/provision.yml playbooks/deploy.yml
Passed: 0 violation(s), 0 warning(s)
```

### Tag Verification

```bash
$ ansible-playbook playbooks/provision.yml --list-tags
play #1 (webservers): TAGS: []
  TASK TAGS: [common, docker, docker_config, docker_install, packages, system, users]
```

### Idempotency

Second run of deploy playbook shows all tasks as `ok` (no changes).

---

## 7. Challenges & Solutions

| Challenge | Solution |
|-----------|----------|
| `app_deploy` → `web_app` rename across all references | Updated deploy.yml, site.yml, and used meta/dependencies |
| Docker Compose module | Used `community.docker.docker_compose_v2` (bundled with `community.docker` collection) |
| Wipe not running by default | Double-gate: `when: web_app_wipe \| bool` + explicit tag |
| Rescue block for GPG key failure | Added `pause: seconds: 10` before retry to handle transient network errors |
