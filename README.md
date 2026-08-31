<p align="center"><strong>Speak</strong></p>
<p align="center">Local speech-to-text for Linux, pasted into the focused app.</p>

Press <code>Alt</code> twice, speak, press <code>Alt</code> twice again, and the final transcription is pasted into your terminal or text field. Speak runs as a user-level systemd service and keeps the Whisper model loaded.

## Highlights

- X11 and Wayland sessions
- Global Alt double-tap hotkey
- Multilingual Whisper models with automatic language detection
- CUDA acceleration with CPU fallback
- RAM-conscious <code>small</code> + CPU <code>int8</code> defaults
- Configurable model, language, device, microphone, and timing

## Install

### System packages

```bash
sudo apt install python3 python3-venv wl-clipboard ydotool
sudo apt install xclip xdotool  # X11 only
sudo usermod -aG input "$USER"
```

Log out and back in after adding the <code>input</code> group. Wayland requires access to <code>/dev/input/event*</code> and <code>/dev/uinput</code>.

### From source

```bash
git clone https://github.com/R3LCH/speak.git
cd speak
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -r worker/requirements.txt
./install.sh
```

The installer builds the Rust binary, installs it to <code>~/.local/bin/speak</code>, copies the worker, creates the user config, and enables <code>speak.service</code>.

## Verify

```bash
speak doctor
systemctl --user status speak.service
```

The first transcription downloads the selected model from Hugging Face; later runs use the local cache.

## Use

1. Focus a terminal prompt or text field.
2. Press and release <code>Alt</code> twice within 300 ms.
3. Speak.
4. Press and release <code>Alt</code> twice again.
5. Wait for the complete text to be pasted.

```bash
journalctl --user -fu speak.service -o cat
```

## Commands

```bash
speak start
speak stop
speak status
speak doctor
systemctl --user restart speak.service
systemctl --user enable --now speak.service
```

## Configuration

Edit <code>~/.config/speak/config.toml</code>:

```toml
model = "small"
device = "auto"
compute_type = "auto"
language = "auto"
beam_size = 1
hotkey = "alt-double-tap"
double_tap_ms = 300
audio_device = "default"
```

Whisper checkpoints are multilingual by default. Keep <code>language = "auto"</code> for detection, or set an ISO code such as <code>en</code> to skip detection. Restart the service after edits:

```bash
systemctl --user restart speak.service
```

## Performance

The default <code>small</code> model with CPU <code>int8</code> prioritizes lower RAM usage. With NVIDIA CUDA 12 and cuDNN 9, <code>device = "auto"</code> uses GPU <code>int8_float16</code> and falls back to CPU <code>int8</code>.

## Troubleshooting

```bash
systemctl --user status speak.service
journalctl --user -u speak.service -n 100 --no-pager
speak doctor
```

For Wayland hotkeys, verify the service has the <code>input</code> group:

```bash
pid=$(systemctl --user show -p MainPID --value speak.service)
grep Groups /proc/$pid/status
```

Wayland paste uses <code>Ctrl+Shift+V</code> and requires <code>wl-copy</code> plus <code>ydotool</code>. X11 paste uses <code>Ctrl+V</code> and requires <code>xclip</code> plus <code>xdotool</code>.

If the worker is missing:

```bash
.venv/bin/python -m pip install -r worker/requirements.txt
.venv/bin/python -c 'import faster_whisper; print("worker ready")'
./install.sh
```

## Development

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
.venv/bin/python -m py_compile worker/speak_worker.py
```

## License

MIT. See [LICENSE](LICENSE).
