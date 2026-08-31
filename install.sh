#!/usr/bin/env bash
set -euo pipefail
cargo install --path . --root "${HOME}/.local"
python3 -m pip install --user -r worker/requirements.txt
install -Dm755 worker/speak_worker.py "${HOME}/.local/share/speak/speak_worker.py"
mkdir -p "${HOME}/.config/speak" "${HOME}/.config/systemd/user"
cp -n config.example.toml "${HOME}/.config/speak/config.toml" || true
sed -e "s#%h/.local/bin/speak#${HOME}/.local/bin/speak#" -e "s#python3 worker/speak_worker.py#python3 ${HOME}/.local/share/speak/speak_worker.py#" config.example.toml > "${HOME}/.config/speak/config.toml"
sed "s#%h/.local/bin/speak#${HOME}/.local/bin/speak#" systemd/speak.service > "${HOME}/.config/systemd/user/speak.service"
systemctl --user daemon-reload
systemctl --user enable --now speak.service
