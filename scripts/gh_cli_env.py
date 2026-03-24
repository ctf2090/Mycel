#!/usr/bin/env python3

from __future__ import annotations

import os
from typing import Mapping


def preferred_gh_env(base_env: Mapping[str, str] | None = None) -> dict[str, str]:
    env = dict(base_env or os.environ)
    agent_token = env.get("GH_TOKEN_AGENT", "").strip()
    if agent_token:
        env["GH_TOKEN"] = agent_token
    return env
