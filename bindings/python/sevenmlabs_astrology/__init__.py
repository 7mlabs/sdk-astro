"""Offline individual natal, couple synastry and chart geometry using the shared native engine."""
import ctypes
import json
import platform
from pathlib import Path

__version__ = "0.10.0a1"
_system = {"Darwin": "darwin", "Linux": "linux", "Windows": "win32"}[platform.system()]
_machine = {"aarch64": "arm64", "arm64": "arm64", "AMD64": "x64", "x86_64": "x64"}.get(platform.machine(), platform.machine())
_file = {"darwin": "libastro_engine.dylib", "linux": "libastro_engine.so", "win32": "astro_engine.dll"}[_system]
_path = Path(__file__).parent / "native" / f"{_system}-{_machine}" / _file
try:
    _lib = ctypes.CDLL(str(_path))
except OSError as exc:
    raise ImportError(f"No usable astrology binary for {_system}-{_machine}: {_path}") from exc
_lib.astro_abi_version.restype = ctypes.c_uint32
if _lib.astro_abi_version() != 1:
    raise ImportError("Unsupported astrology ABI version")
_lib.astro_calculate_json.argtypes = [ctypes.c_char_p]
_lib.astro_calculate_json.restype = ctypes.c_void_p
_lib.astro_compress_json.argtypes = [ctypes.c_char_p]
_lib.astro_compress_json.restype = ctypes.c_void_p
_lib.astro_expand_context_json.argtypes = [ctypes.c_char_p]
_lib.astro_expand_context_json.restype = ctypes.c_void_p
_lib.astro_free_string.argtypes = [ctypes.c_void_p]
_lib.astro_free_string.restype = None


class EngineError(ValueError):
    def __init__(self, result):
        self.result = result
        self.code = result["errors"][0]["code"]
        super().__init__(result["errors"][0]["message"])


def calculate_json(request: str) -> str:
    """Return a JSON envelope. Caller-owned input; Rust-owned output freed here."""
    return _native_json(request, _lib.astro_calculate_json, 4 * 1024 * 1024, "calculate_json")


def _native_json(request, operation, max_bytes, name):
    if not isinstance(request, str):
        raise TypeError(f"{name} expects a JSON string")
    encoded = request.encode("utf-8")
    if b"\0" in encoded:
        raise ValueError("Input JSON cannot contain a literal NUL")
    if len(encoded) > max_bytes:
        raise ValueError(f"Input exceeds {max_bytes // (1024 * 1024)} MiB")
    pointer = operation(encoded)
    if not pointer:
        raise MemoryError("Engine returned a null result")
    try:
        return ctypes.string_at(pointer).decode("utf-8")
    finally:
        _lib.astro_free_string(pointer)


def calculate(request: dict) -> dict:
    return _checked_result(calculate_json(json.dumps(request, allow_nan=False)))


def _checked_result(output):
    result = json.loads(output)
    if result["errors"]:
        raise EngineError(result)
    return result


def compress_payload_json(payload_json: str, options_json: str = "{}") -> str:
    """Prepare context without reparsing raw payload number literals; return errors in the envelope."""
    if not isinstance(payload_json, str) or not isinstance(options_json, str):
        raise TypeError("compress_payload_json expects JSON strings for payload and options")
    wrapper = '{"payload":' + payload_json + ',"options":' + options_json + '}'
    return _native_json(wrapper, _lib.astro_compress_json, 256 * 1024 * 1024, "compress_payload_json")


def compress_payload(payload: dict, options: dict | None = None) -> dict:
    """Create an astro-context/1 envelope in the shared core. The input is not mutated."""
    if options is None:
        options = {}
    return _checked_result(compress_payload_json(
        json.dumps(payload, allow_nan=False), json.dumps(options, allow_nan=False)))


def expand_context_json(context_json: str) -> str:
    """Decode the retained engine envelope. Omitted facts cannot be restored."""
    return _native_json(context_json, _lib.astro_expand_context_json, 256 * 1024 * 1024, "expand_context_json")


def expand_context(context: dict) -> dict:
    return _checked_result(expand_context_json(json.dumps(context, allow_nan=False)))


def calculate_with_context(request: dict, options: dict | None = None) -> dict:
    """Calculate synchronously; return both the original result and its prepared context."""
    result = calculate(request)
    return {"result": result, "context": compress_payload(result, options)}


def _query_method(group, action):
    def invoke(options):
        if not isinstance(options, dict):
            raise TypeError(f"{group}.{action} expects an options dictionary")
        for key in ("operation", "group", "action"):
            if key in options:
                raise TypeError(f"{key} is set by {group}.{action}")
        return calculate({**options, "operation": "query", "group": group, "action": action})
    invoke.__name__ = action
    invoke.__doc__ = f"Return the JSON envelope for query {group}.{action}; all calculation runs in Rust."
    return invoke


class _Geometry:
    normalize = staticmethod(_query_method("geometry", "normalize"))
    separation = staticmethod(_query_method("geometry", "separation"))
    midpoint = staticmethod(_query_method("geometry", "midpoint"))


class _Aspects:
    between = staticmethod(_query_method("aspects", "between"))
    inspect = staticmethod(_query_method("aspects", "inspect"))


class _Houses:
    locate = staticmethod(_query_method("houses", "locate"))
    inspect = staticmethod(_query_method("houses", "inspect"))


class _Points:
    inspect = staticmethod(_query_method("points", "inspect"))


geometry = _Geometry()
aspects = _Aspects()
houses = _Houses()
points = _Points()
