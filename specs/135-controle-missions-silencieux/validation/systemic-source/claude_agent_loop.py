#!/usr/bin/env python3
"""Compatibilité Claude : une seule implémentation Agent Loop, côté Codex."""
from pathlib import Path

_canonical = Path('/Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py')
exec(compile(_canonical.read_bytes(), str(_canonical), 'exec'))
