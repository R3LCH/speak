# Speech-to-CLI Design Specification

## Goal

Provide a Linux background daemon that records speech on a configurable global hotkey, transcribes it with `faster-whisper`, and pastes the final text into the currently focused terminal or application. The default workflow is optimized for low RAM and CPU use while using an available NVIDIA GPU automatically.

## Scope and Defaults

- Linux only for the initial release.
- X11 and Wayland are supported in v1.
- Default hotkey is Alt double-tap: two presses of either Alt key within a configurable interval toggle recording.
- Recording is push-toggle: first activation starts capture, second stops capture.
- Transcription runs after stop and pastes one complete result; no streaming partial paste in v1.
- Whisper language detection is automatic by default. An optional language override skips detection for users who know the input language.
- Default model is `small` with `int8` CPU quantization. In `auto` device mode, use `int8_float16` on a working CUDA device and fall back to CPU `int8` on failure.
- Model name, device policy, compute type, language, beam size, model cache, audio input, hotkey, and timing are configurable.

## Architecture

The primary executable is a Rust daemon and CLI built as one Cargo package. The daemon contains a state machine (`Idle`, `Recording`, `Transcribing`, `Error`), audio capture, backend selection, temporary audio-file lifecycle, configuration, logging, and text delivery. A long-lived Python worker process runs `faster-whisper`, loads one model, accepts JSON-lines requests on stdin, and returns JSON-lines responses on stdout. Keeping the worker separate uses the requested upstream implementation and allows its Python/CTranslate2 runtime to be upgraded independently of the Rust control plane.

The daemon selects an input and delivery backend at startup. X11 uses X11 key observation plus XTest and clipboard tooling. Wayland first attempts the `org.freedesktop.portal.GlobalShortcuts` portal for configured chord shortcuts; the default Alt double-tap is recognized from `evdev`, and paste is synthesized through `uinput`. Wayland setup checks `/dev/input` and `/dev/uinput` access and reports actionable errors. Backend traits isolate hotkey and paste behavior from transcription.

## Data Flow

1. Load TOML configuration, validate values, and detect session type.
2. Start the Python worker and request model initialization with device policy.
3. Subscribe to the selected global-hotkey backend.
4. On the first activation, open the selected audio input and transition to `Recording`.
5. On the second activation, stop capture, write a short-lived 16 kHz mono WAV, and transition to `Transcribing`.
6. Send the audio path and transcription options to the worker; consume all returned segments and join normalized text.
7. Copy text to the clipboard and synthesize paste into the previously focused target. Empty or whitespace-only output is not pasted.
8. Delete the temporary audio file and return to `Idle`.

## Configuration and CLI

Configuration is TOML at `$XDG_CONFIG_HOME/speak/config.toml` (fallback `~/.config/speak/config.toml`). The CLI exposes `run`, `start`, `stop`, `status`, `config`, and `doctor` commands. `doctor` checks session type, backend availability, input/uinput permissions, Python worker startup, model cache, and CUDA availability. `start`/`stop` use a Unix-domain control socket with a user-only filesystem permission.

Important settings include:

```toml
model = "small"
device = "auto"          # auto | cpu | cuda
compute_type = "auto"   # auto | int8 | int8_float16 | float16
language = "auto"
beam_size = 1
hotkey = "alt-double-tap"
double_tap_ms = 300
audio_device = "default"
```

The worker command, Python executable, and model cache directory are also configurable. CLI flags override file values for one invocation.

## Resource and Performance Policy

- Load the model once per daemon and reuse it.
- Use CPU `int8` by default; use beam size 1 unless configured otherwise.
- Limit audio buffering to the active recording and avoid retaining decoded samples after WAV creation.
- Use VAD filtering and `condition_on_previous_text = false` for independent dictation snippets.
- Use bounded worker queues so repeated hotkeys cannot create unbounded memory growth.
- CUDA initialization is attempted once; failures are logged and trigger CPU fallback without losing the recording.

## Error Handling

Invalid configuration fails before daemon startup with a field-specific message. Audio-device, permission, worker protocol, model-load, CUDA, clipboard, and paste failures produce structured logs and a user-visible desktop notification where available. A failed transcription never pastes partial or stale text. The daemon remains alive after a per-recording failure and returns to `Idle`; unrecoverable worker failure causes one restart attempt, then `Error` until the next health check or explicit restart.

## Security and Permissions

The daemon runs as the unprivileged user. Wayland evdev/uinput operation requires read access to the relevant `/dev/input/event*` device and write access to `/dev/uinput`, provided through group membership or narrowly scoped udev rules. The control socket is created with mode `0600`. Audio files are created in a private runtime directory and removed after each request, including shutdown cleanup.

## Testing and Verification

- Unit tests cover TOML defaults/validation, hotkey double-tap timing, state transitions, text normalization, worker JSON protocol, device-policy fallback, and temporary-file cleanup.
- Backend tests use mock event and paste implementations; X11/Wayland integration tests are opt-in and skipped with clear diagnostics when a display/session or device permissions are unavailable.
- End-to-end tests run the daemon with a fake worker and fake audio source, assert one complete paste, and verify no paste for empty transcription.
- Verification includes `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, Python worker tests, and `doctor` output on CPU-only and CUDA-capable hosts where available.

## Packaging and Operations

Ship the Rust binary, Python worker module/requirements, an example config, and a user systemd unit. The unit starts after the graphical session, restarts on failure, and logs to the user journal. Installation documents Python 3.9+, `faster-whisper`, optional NVIDIA CUDA 12/cuDNN 9 libraries, X11 dependencies, Wayland portal availability, and evdev/uinput permissions.

## Non-Goals for v1

- Streaming partial transcription and incremental paste.
- Cloud transcription or remote model execution.
- A graphical settings application.
- Automatic modification of system-wide udev/group policy.
- Guaranteed Wayland support on compositors lacking the GlobalShortcuts portal or permitting the required input devices; `doctor` must identify the missing capability.
