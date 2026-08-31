# Speech-to-CLI Production Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task with verification checkpoints.

**Goal:** Deliver a usable Linux background speech-to-text tool: Alt double-tap starts/stops recording, `faster-whisper` transcribes the complete clip, and the result is pasted into the focused terminal/application on X11 and Wayland.

**Architecture:** Rust owns configuration, continuous daemon lifecycle, audio capture, state machine, hotkey/paste backends, control socket, diagnostics, and systemd integration. A persistent Python `faster-whisper` worker communicates over a versioned JSON-lines protocol and loads one model per daemon. X11 uses XInput2/XTest; Wayland uses GlobalShortcuts via zbus for configured chords and evdev/uinput for Alt double-tap and paste when permissions allow.

**Tech Stack:** Rust stable, `cpal`, `hound`, `evdev`, `uinput`, `x11rb`, `zbus`, `serde`, `toml`, `clap`, `tracing`; Python 3.9+, `faster-whisper`, pytest; user systemd.

**Spec:** `docs/superpowers/specs/2026-08-31-speech-to-cli-design.md`

## Acceptance Criteria

- `speak run` stays alive and handles repeated recordings.
- Alt double-tap toggles recording on X11 and supported Wayland setups.
- Microphone input is converted to 16 kHz mono PCM and released after each clip.
- Worker loads one configurable model; `auto` selects CUDA `int8_float16`, then CPU `int8`.
- Language auto-detects by default; final text is pasted once; empty output is ignored.
- `doctor` reports missing session, portal, device permissions, clipboard, Python, model, or CUDA capability.
- `start|stop|status` use a `0600` Unix socket and user systemd unit.
- Unit, protocol, backend-mock, and CPU-only e2e tests pass.

## Global Constraints

- Linux-only v1 with X11 and Wayland support.
- No streaming partial paste.
- Defaults: `small`, device `auto`, CPU `int8`, CUDA `int8_float16`, beam 1, language `auto`, 300 ms double-tap.
- Unprivileged daemon; evdev/uinput access is diagnosed and documented.
- One persistent worker and bounded queues.

### Task 1: Configuration and CLI contract

**Files:** `Cargo.toml`, `src/config.rs`, `src/cli.rs`, `src/main.rs`, `src/lib.rs`, `config.example.toml`, `tests/config.rs`, `README.md`

- [ ] Define typed config/defaults, XDG path loading, validation, and CLI overrides.
- [ ] Add commands `run`, `start`, `stop`, `status`, `config`, `doctor`.
- [ ] Test defaults, TOML round-trip, invalid values, and override precedence.
- [ ] Run format and `cargo test config`; commit.

### Task 2: Real audio capture

**Files:** `src/audio.rs`, `src/audio/resampler.rs`, `tests/audio.rs`

- [ ] Implement `cpal` device enumeration/selection and callback capture into a bounded ring buffer.
- [ ] Convert f32/i16, arbitrary channels, and sample rates to mono 16 kHz i16 without blocking callbacks.
- [ ] Write unique private WAV files under `$XDG_RUNTIME_DIR/speak`; delete on success, error, and shutdown.
- [ ] Test conversion, overflow policy, WAV headers, and cleanup; run clippy; commit.

### Task 3: X11 backend

**Files:** `src/backend/mod.rs`, `src/backend/x11.rs`, `src/hotkey.rs`, `src/paste.rs`, `tests/x11_mock.rs`

- [ ] Subscribe to XInput2 raw key events and implement left/right Alt double-tap with release filtering.
- [ ] Capture focused window, write clipboard selection, and inject Ctrl+V through XTest.
- [ ] Recover from connection loss and report missing `DISPLAY`, XInput2, or XTest capabilities.
- [ ] Add mock tests and opt-in Xvfb integration; commit.

### Task 4: Wayland backend

**Files:** `src/backend/wayland.rs`, `src/backend/portal.rs`, `src/backend/uinput.rs`, `tests/wayland_mock.rs`

- [ ] Register configured chord shortcuts through `org.freedesktop.portal.GlobalShortcuts` over zbus.
- [ ] Enumerate keyboard-capable `/dev/input/event*` devices and feed Alt events to the shared detector.
- [ ] Create a named uinput virtual keyboard and synthesize Ctrl+V; cleanly close it.
- [ ] Select portal or evdev paths explicitly; never silently claim unsupported compositors work.
- [ ] Test D-Bus/evdev/uinput mocks and gated live-session tests; commit.

### Task 5: Persistent faster-whisper worker

**Files:** `worker/speak_worker.py`, `worker/requirements.txt`, `worker/test_worker.py`, `src/worker.rs`, `tests/worker_protocol.rs`

- [ ] Implement versioned JSON-lines requests/responses, ready handshake, request IDs, timeout, and structured errors.
- [ ] Load one model; map `auto` to CUDA `int8_float16` then CPU `int8`; report `device_used`.
- [ ] Use `language=None` for detection, configured ISO language otherwise, VAD, beam size, and `condition_on_previous_text=False`.
- [ ] Supervise subprocess, drain stderr, validate IDs, restart once, and shut down cleanly.
- [ ] Test fake model/child for malformed JSON, timeout, fallback, restart, and segment joining; commit.

### Task 6: Continuous daemon workflow

**Files:** `src/state.rs`, `src/daemon.rs`, `src/notify.rs`, `tests/daemon.rs`

- [ ] Implement `Idle -> Recording -> Transcribing -> Idle` plus recoverable `Error` and shutdown cancellation.
- [ ] Capture focused target, record, transcribe exactly once, normalize text, paste once, and clean resources.
- [ ] Ignore activations while busy, cap recording duration, and notify/log failures.
- [ ] Test fake backends for repeated clips, empty output, worker restart, max duration, and cleanup; commit.

### Task 7: Control, service, diagnostics

**Files:** `src/control.rs`, `src/doctor.rs`, `systemd/speak.service`, `tests/control.rs`, `tests/doctor.rs`

- [ ] Implement single-instance locking and `0600` runtime socket with `status|stop|reload`.
- [ ] Wire CLI commands to the user service and graceful signals.
- [ ] Check session, X11/Wayland, portal, evdev/uinput, audio, Python/package, model cache, clipboard, and CUDA with remediation.
- [ ] Test authorization, stale sockets, malformed commands, and service syntax; commit.

### Task 8: Packaging and end-to-end verification

**Files:** `install.sh`, `scripts/check-deps.sh`, `tests/e2e.rs`, `README.md`

- [ ] Package binary, worker, config, and user unit without root; document CUDA 12/cuDNN 9 as optional.
- [ ] Check required X11 (`xclip`, `xdotool`) and Wayland (`wl-clipboard`, `ydotool`, portal) dependencies accurately.
- [ ] Add CPU-only fake-worker e2e and opt-in real-model smoke test via `SPEAK_REAL_MODEL=1`.
- [ ] Document model download, language behavior, permissions, systemd, troubleshooting, and compositor limitations.
- [ ] Run `cargo fmt --check`, clippy, `cargo test`, `pytest -q`, dependency checks, and `speak doctor`; commit.

## Plan Self-Review

Tasks 1-8 cover every spec requirement and replace the previous scaffold-only work with concrete implementation and verification steps. No placeholders or unspecified “appropriate” behavior remain; unsupported Wayland capabilities must be surfaced by `doctor`.
