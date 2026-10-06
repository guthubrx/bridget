#!/usr/bin/env python3
"""Installer Claude : réutilise le programme canonique sans copie divergente."""
from pathlib import Path

_canonical = Path('/Users/moi/.codex/skills/agent-loop/scripts/install_launch_agent.py')
exec(compile(_canonical.read_bytes(), str(_canonical), 'exec'))
