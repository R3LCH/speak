# speak

Local speech-to-text for Linux. Press and release Alt twice to start recording, speak, press Alt twice again, and the complete transcription is pasted into the focused terminal or application.

The Rust daemon runs in the background. A persistent Python worker runs [faster-whisper](https://github.com/SYSTRAN/faster-whisper), using NVIDIA CUDA when available and CPU fallback otherwise.

## Requirements

- Linux with X11 or Wayland
- Rust/Cargo
- Python 3.9+ and `python3-venv`
- Working microphone

X11:

```bash
sudo apt install xclip xdotool
```

Wayland:

```bash
sudo apt install wl-clipboard ydotool
sudo usermod -aG input "$USER"
```

Log out and back in after adding the `input` group. Wayland also needs read access to `/dev/input/event*` and write access to `/dev/uinput`.

## Installation

```bash
git clone https://github.com/R3LCH/speak.git
cd speak
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -r worker/requirements.txt
./install.sh
```

The installer builds the Rust binary, installs it to `~/.local/bin/speak`, copies the worker to `~/.local/share/speak/`, creates the user config, and enables the user systemd service.

If needed, add the binary directory to `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Verify

```bash
speak doctor
systemctl --user status speak.service
```

The first transcription downloads the Whisper model. Later runs use the local cache.

## Use

1. Focus a terminal prompt or text field.
2. Press and release `Alt` twice within 300 ms.
3. Speak.
4. Press and release `Alt` twice again.
5. Wait for the final text to be pasted.

Watch logs while testing:

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

Edit `~/.config/speak/config.toml`:

```toml
model = "small"
device = "auto"          # auto, cpu, or cuda
compute_type = "auto"   # auto, int8, int8_float16, or float16
language = "auto"       # automatic detection, or an ISO code such as "en"
beam_size = 1
hotkey = "alt-double-tap"
double_tap_ms = 300
audio_device = "default"
```

Whisper checkpoints are multilingual by default. `language = "auto"` enables detection; setting an ISO code skips detection and can reduce latency. Larger models improve accuracy but use more RAM/VRAM.

After editing:

```bash
systemctl --user restart speak.service
```

## Performance

The default `small` model with CPU `int8` prioritizes lower RAM usage. With compatible CUDA 12 and cuDNN 9 libraries, `device = "auto"` uses GPU `int8_float16` and falls back to CPU `int8` if GPU initialization fails.

## Troubleshooting

Inspect recent errors:

```bash
journalctl --user -u speak.service -n 100 --no-pager
speak doctor
```

For Wayland hotkeys, verify the running service inherited the `input` group:

```bash
pid=$(systemctl --user show -p MainPID --value speak.service)
grep Groups /proc/$pid/status
```

For Wayland paste, `wl-copy` and `ydotool` are required. For X11 paste, `xclip` and `xdotool` are required. If `faster-whisper` is missing, reinstall it in the project environment and run `./install.sh` again:

```bash
.venv/bin/python -m pip install -r worker/requirements.txt
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
