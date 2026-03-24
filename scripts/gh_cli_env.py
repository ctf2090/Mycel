#!/usr/bin/env python3

from __future__ import annotations

import os
from typing import Mapping


def preferred_gh_env(base_env: Mapping[str, str] | None = None) -> dict[str, str]:
    env = dict(base_env or os.environ)
    # Repo-local default: GH_TOKEN is the agent identity. Keep a legacy
    # fallback for older shells that may still export GH_TOKEN_AGENT.
    agent_token = env.get("GH_TOKEN", "").strip() or env.get("GH_TOKEN_AGENT", "").strip()
    if agent_token:
        env["GH_TOKEN"] = agent_token
    return env


def preferred_user_gh_env(base_env: Mapping[str, str] | None = None) -> dict[str, str]:
    env = dict(base_env or os.environ)
    user_token = env.get("GH_TOKEN_USER", "").strip()
    if user_token:
        env["GH_TOKEN"] = user_token
    return env


def preferred_git_https_env(base_env: Mapping[str, str] | None = None) -> dict[str, str]:
    env = preferred_gh_env(base_env)
    agent_token = env.get("GH_TOKEN", "").strip()
    if agent_token:
        # Codespaces' default HTTPS git credential helper consumes GITHUB_TOKEN
        # rather than GH_TOKEN, so mirror the agent token here for git push/pull.
        env["GITHUB_TOKEN"] = agent_token
    env.setdefault("GITHUB_SERVER_URL", "https://github.com")
    env["GIT_TERMINAL_PROMPT"] = "0"
    return env
