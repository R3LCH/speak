# Speech-to-CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Linux Rust daemon and CLI that records speech via Alt double-tap, transcribes it with a persistent `faster-whisper` worker using automatic GPU/CPU selection, and pastes the complete transcription into the focused application on both X11 and Wayland.

**Architecture:** A Cargo workspace contains a Rust binary/library with configuration, state machine, audio capture, hotkey and paste backend traits, worker protocol, control socket, and CLI. A Python JSON-lines worker loads one `faster-whisper` model and serves transcription requests. Runtime backend selection chooses X11 or Wayland capabilities while sharing recording and delivery logic.

**Tech Stack:** Rust stable, Cargo, `cpal` + `hound`, `serde`/`toml`, `clap`, `tracing`, `evdev`, `uinput`, X11/XTest bindings, zbus for portals, Python 3.9+, `faster-whisper`, pytest, user systemd.

**Spec:** `docs/superpowers/specs/2026-08-31-speech-to-cli-design.md`

## Global Constraints

- Linux-only initial release with X11 and Wayland support.
- Default hotkey is Alt double-tap with a configurable 300 ms interval.
- Paste one complete transcription after recording stops; do not stream partial text.
- Default model is `small`; CPU uses `int8`, CUDA uses `int8_float16` when available.
- Language is automatic by default; explicit language is an optional override.
- Run unprivileged; Wayland evdev/uinput access must be diagnosed, not silently bypassed.
- Model is loaded once and reused; recording/transcription queues are bounded.

### Task 1: Bootstrap workspace and configuration

**Files:**
- Create: `Cargo.toml`, `src/lib.rs`, `src/main.rs`
- Create: `src/config.rs`, `tests/config.rs`, `config.example.toml`
- Create: `README.md`

**Interfaces:**
- Produces `Config`, `DevicePolicy`, `ComputeType`, `HotkeyConfig`, `Config::load(path) -> Result<Config>`, and CLI subcommands `run`, `start`, `stop`, `status`, `config`, `doctor`.

- [ ] Write tests asserting missing fields receive defaults (`small`, `auto`, `beam_size=1`, `alt-double-tap`, 300 ms) and invalid enum/range values fail with field names.
- [ ] Run `cargo test config`; verify tests fail because types are absent.
- [ ] Implement serde-deserializable TOML configuration, XDG path resolution, CLI parsing, and example config.
- [ ] Run `cargo fmt --check`, `cargo test config`, and verify pass.
- [ ] Commit `feat: bootstrap speak configuration and cli`.

### Task 2: Audio capture and WAV lifecycle

**Files:**
- Create: `src/audio.rs`, `tests/audio.rs`
- Modify: `Cargo.toml`

**Interfaces:**
- Produces `AudioCapture::start(device: &str) -> Result<RecordingHandle>`, `RecordingHandle::stop(self) -> Result<TempAudio>`, and `TempAudio::path() -> &Path` with automatic deletion on drop.

- [ ] Add tests with a fake capture source for 16 kHz mono conversion, stop behavior, and cleanup after drop.
- [ ] Run `cargo test audio`; verify failure before implementation.
- [ ] Implement `cpal` input selection, bounded sample buffering, conversion to mono 16-bit PCM at 16 kHz, private runtime-directory WAV creation via `hound`, and cleanup guards.
- [ ] Run `cargo test audio` and `cargo clippy -- -D warnings`.
- [ ] Commit `feat: add bounded audio capture`.

### Task 3: Hotkey and paste backend abstractions

**Files:**
- Create: `src/backend/mod.rs`, `src/backend/x11.rs`, `src/backend/wayland.rs`, `src/hotkey.rs`, `src/paste.rs`, `tests/hotkey.rs`, `tests/backend_mocks.rs`
- Modify: `Cargo.toml`

**Interfaces:**
- `HotkeyBackend::next_activation() -> Result<Activation>` and `PasteBackend::paste(text: &str) -> Result<()>`.
- `DoubleTapDetector::feed(event: KeyEvent) -> Option<Activation>`.
- `BackendFactory::detect(config: &Config) -> Result<BackendSet>`.

- [ ] Test double-tap timing, left/right Alt handling, key-release filtering, and reset after timeout using deterministic timestamps.
- [ ] Run `cargo test hotkey`; verify failure.
- [ ] Implement detector and mock traits.
- [ ] Implement X11 event hook and XTest paste path, preserving clipboard text and focused-window timing.
- [ ] Implement Wayland portal registration through zbus for configured shortcuts; implement evdev Alt-event reading for double-tap and uinput Ctrl+V synthesis, with explicit permission errors.
- [ ] Add capability tests that skip real display/device integration when unavailable.
- [ ] Run backend unit tests, `cargo fmt --check`, and clippy.
- [ ] Commit `feat: support x11 and wayland input backends`.

### Task 4: Faster-whisper worker protocol

**Files:**
- Create: `worker/speak_worker.py`, `worker/requirements.txt`, `worker/test_worker.py`
- Create: `src/worker.rs`, `tests/worker_protocol.rs`

**Interfaces:**
- Rust `WorkerClient::start(config: &Config)`, `WorkerClient::transcribe(request: TranscriptionRequest) -> Result<TranscriptionResponse>`, `WorkerClient::restart()`.
- JSON request `{"id": string, "audio_path": string, "model": string, "device": string, "compute_type": string, "language": string|null, "beam_size": u32}`; response `{ "id": string, "ok": bool, "text": string|null, "error": string|null, "device_used": string }`.

- [ ] Write Python tests for model initialization options, automatic language handling, segment joining, VAD, and malformed requests.
- [ ] Write Rust protocol tests with a fake child process for round trips, mismatched IDs, malformed JSON, timeout, and one restart.
- [ ] Run pytest and Rust protocol tests; verify failures.
- [ ] Implement persistent worker model caching, `device=auto` CUDA probe with fallback to CPU `int8`, `language=auto` mapping to `None`, `beam_size`, VAD, and `condition_on_previous_text=False`.
- [ ] Implement line-delimited Rust I/O with bounded request handling, stderr capture, timeout, and actionable errors.
- [ ] Run `pytest -q`, `cargo test worker_protocol`, and clippy.
- [ ] Commit `feat: add persistent faster whisper worker`.

### Task 5: Daemon state machine and end-to-end flow

**Files:**
- Create: `src/daemon.rs`, `src/state.rs`, `tests/daemon.rs`
- Modify: `src/lib.rs`, `src/main.rs`

**Interfaces:**
- `Daemon::run(config: Config) -> Result<()>`.
- States: `Idle`, `Recording(RecordingHandle)`, `Transcribing`, `Error`.
- `normalize_transcription(text: &str) -> Option<String>`.

- [ ] Add end-to-end tests using fake hotkey, audio, worker, and paste backends: one complete paste, empty text skipped, recording failure recovery, and worker restart.
- [ ] Run `cargo test daemon`; verify failure.
- [ ] Implement event loop, state transitions, temporary-file ownership, text normalization, single final paste, structured tracing, and bounded repeated-activation handling.
- [ ] Run daemon tests and `cargo clippy -- -D warnings`.
- [ ] Commit `feat: implement speech daemon lifecycle`.

### Task 6: Control socket, systemd service, and diagnostics

**Files:**
- Create: `src/control.rs`, `systemd/speak.service`, `tests/control.rs`
- Modify: `src/main.rs`, `README.md`

**Interfaces:**
- Unix socket commands `status`, `stop`, `reload`; socket mode `0600`.
- `doctor` checks session, backend, devices, Python worker, model cache, and CUDA.

- [ ] Test socket authorization, command parsing, graceful stop, and status serialization.
- [ ] Run `cargo test control`; verify failure.
- [ ] Implement runtime-dir socket lifecycle, CLI client commands, signal handling, and systemd user unit ordered after graphical session.
- [ ] Implement `doctor` checks with remediation text for X11/Wayland, `/dev/input`, `/dev/uinput`, portal, Python, model cache, and CUDA.
- [ ] Run control tests, `cargo test`, and inspect `systemd-analyze verify` where available.
- [ ] Commit `feat: add daemon control and systemd integration`.

### Task 7: Packaging and verification

**Files:**
- Create: `install.sh`, `scripts/check-deps.sh`, `tests/e2e.rs`
- Modify: `README.md`, `worker/requirements.txt`, `config.example.toml`

- [ ] Add CPU-only integration test using a fake worker and verify no CUDA dependency is required for startup.
- [ ] Add optional real-worker smoke test gated by `SPEAK_REAL_MODEL=1`.
- [ ] Document Python/faster-whisper installation, CUDA 12/cuDNN 9 optional libraries, X11 packages, Wayland portal support, evdev/uinput group or udev setup, systemd commands, and model cache sizing.
- [ ] Run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `pytest -q`, and `scripts/check-deps.sh`.
- [ ] Commit `docs: document installation and verification`.

## Self-Review Checklist

- Spec coverage: tasks 1-7 cover configuration, hotkeys, both display protocols, audio, worker, resource policy, errors, security, tests, and packaging.
- Placeholder scan: no `TBD`, `TODO`, or unspecified implementation steps are used.
- Interface consistency: daemon consumes `AudioCapture`, `BackendSet`, and `WorkerClient` interfaces defined in earlier tasks; `doctor` and systemd are added after daemon startup exists.
