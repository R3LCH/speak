# speak

Linux speech-to-text daemon for pasting dictation into the focused application.

Install dependencies with `python3 -m pip install --user -r worker/requirements.txt`, then run `./install.sh`. The default model is multilingual `small`; language detection is automatic, CPU inference uses int8, and CUDA is selected automatically when available. Set `language` to an ISO code to skip detection.

X11 requires `xclip` and `xdotool`. Wayland requires `wl-clipboard`, `ydotool`, the GlobalShortcuts portal where supported, and permission to read `/dev/input/event*` and write `/dev/uinput` (usually via udev rules or group membership). Run `speak doctor` to inspect the session and dependencies.
