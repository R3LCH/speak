#!/usr/bin/env bash
set -u
ok=0
for cmd in python3; do command -v "$cmd" >/dev/null || { echo "missing: $cmd"; ok=1; }; done
if [ -n "${WAYLAND_DISPLAY:-}" ]; then command -v wl-copy >/dev/null || echo "missing optional: wl-copy"; [ -e /dev/uinput ] || echo "missing: /dev/uinput access"; fi
if [ -n "${DISPLAY:-}" ]; then command -v xclip >/dev/null || { echo "missing required for X11 paste: xclip"; ok=1; }; command -v xdotool >/dev/null || { echo "missing required for X11 paste: xdotool"; ok=1; }; fi
exit "$ok"
