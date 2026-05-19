---
# TRACKER CONFIGURATION
# Using GitHub with gh CLI auto-detection

tracker:
  kind: github
  # api_key is optional - Symphony will auto-detect from gh CLI
  project_slug: tmorin/plantuml-generator
  active_states:
    - Open
  terminal_states:
    - Closed

polling:
  interval_ms: 30000

workspace:
  root: ~/.symphony/workspaces

hooks:
  after_create: |
    echo "Workspace created: $(pwd)"
    git init || true
  before_run: |
    echo "Starting agent run in $(pwd)"
  after_run: |
    echo "Agent run completed"
  timeout_ms: 60000

agent:
  max_concurrent_agents: 2
  max_turns: 3
  max_retry_backoff_ms: 300000
  max_concurrent_agents_by_state:
    "In Progress": 1

codex:
  command: "copilot-cli"
  approval_policy: auto
  thread_sandbox: none
  turn_sandbox_policy: allow
  turn_timeout_ms: 1800000
  read_timeout_ms: 5000
  stall_timeout_ms: 300000

server:
  port: 3000
---

# Issue: {{ issue.identifier }}

You are a coding agent working on the following issue:

**Title:** {{ issue.title }}

**State:** {{ issue.state }}

**Priority:** {{ issue.priority }}

**Description:**
{{ issue.description }}

## Your Task

Analyze the issue and determine the best course of action. You may:
1. Make code changes in the workspace
2. Create or update files
3. Run tests and builds
4. Ask for clarification if needed

Work incrementally, test your changes, and provide clear commit messages describing your work.
