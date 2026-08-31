#!/usr/bin/env bash
set -euo pipefail
cargo install --path . --root "${HOME}/.local"
if [ -x .venv/bin/python ]; then .venv/bin/python -m pip install -r worker/requirements.txt; else python3 -m pip install --user -r worker/requirements.txt; fi
install -Dm755 worker/speak_worker.py "${HOME}/.local/share/speak/speak_worker.py"
mkdir -p "${HOME}/.config/speak" "${HOME}/.config/systemd/user"
cp -n config.example.toml "${HOME}/.config/speak/config.toml" || true
worker_python="python3"; [ -x .venv/bin/python ] && worker_python="${PWD}/.venv/bin/python"
sed -e "s#%h/.local/bin/speak#${HOME}/.local/bin/speak#" -e "s#python3 worker/speak_worker.py#${worker_python} ${HOME}/.local/share/speak/speak_worker.py#" config.example.toml > "${HOME}/.config/speak/config.toml"
sed "s#%h/.local/bin/speak#${HOME}/.local/bin/speak#" systemd/speak.service > "${HOME}/.config/systemd/user/speak.service"
systemctl --user daemon-reload
systemctl --user enable --now speak.service
