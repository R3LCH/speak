import json, os, sys
from faster_whisper import WhisperModel

_models = {}
def model_for(req):
    model_name = req.get("model", "small")
    device = req.get("device", "auto")
    compute = req.get("compute_type", "auto")
    candidates = [("cuda", "int8_float16"), ("cpu", "int8")] if device == "auto" else [(device, compute if compute != "auto" else ("int8_float16" if device == "cuda" else "int8"))]
    key = (model_name, candidates[0])
    if key in _models: return _models[key], candidates[0][0]
    last = None
    for dev, typ in candidates:
        try:
            m = WhisperModel(model_name, device=dev, compute_type=typ, download_root=os.environ.get("SPEAK_MODEL_CACHE"))
            _models[key] = m
            return m, dev
        except Exception as exc: last = exc
    raise RuntimeError(f"model initialization failed: {last}")

def handle(req):
    m, dev = model_for(req)
    language = req.get("language") or None
    segments, _ = m.transcribe(req["audio_path"], beam_size=int(req.get("beam_size", 1)), language=language, vad_filter=True, condition_on_previous_text=False)
    text = " ".join(s.text.strip() for s in segments if s.text.strip()).strip()
    return {"id": req.get("id", ""), "ok": True, "text": text, "error": None, "device_used": dev}

for line in sys.stdin:
    try: print(json.dumps(handle(json.loads(line))), flush=True)
    except Exception as exc: print(json.dumps({"id":"", "ok":False, "text":None, "error":str(exc), "device_used":"none"}), flush=True)
