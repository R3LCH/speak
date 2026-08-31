#!/usr/bin/env bash
set -euo pipefail
cargo install --path . --root "${HOME}/.local"
python3 -m pip install --user -r worker/requirements.txt
mkdir -p "${HOME}/.config/speak" "${HOME}/.config/systemd/user"
cp -n config.example.toml "${HOME}/.config/speak/config.toml" || true
sed "s#%h/.local/bin/speak#${HOME}/.local/bin/speak#" systemd/speak.service > "${HOME}/.config/systemd/user/speak.service"
systemctl --user daemon-reload
systemctl --user enable --now speak.service
