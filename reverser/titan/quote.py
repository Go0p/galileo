#!/usr/bin/env python3
"""
Minimal Mobile Wallet Adapter websocket helper and gzip+msgpack decoder.

This mirrors the cryptographic handshake that appears in `js/1.js`:
  * P-256 ECDH to agree a shared secret.
  * HKDF-SHA256 using the association public key as salt.
  * AES-GCM (128 bit) frames with 4-byte big-endian sequence numbers as AAD.

The script exposes building blocks so you can experiment from Python,
e.g. generate the association payload or encrypt/decrypt JSON-RPC frames.

Dependencies:
    pip install cryptography websockets msgpack

Example (offline round-trip):
    >>> ctx = AssociationContext.generate()
    >>> peer = AssociationContext.generate()
    >>> session = SessionCrypto.from_handshake(
    ...     association=ctx,
    ...     peer_public_key=peer.public_key_bytes
    ... )
    >>> message = session.encrypt_json({"jsonrpc": "2.0", "id": 1, "method": "ping"})
    >>> session.decrypt_json(message)
    {'jsonrpc': '2.0', 'id': 1, 'method': 'ping'}

For a real websocket flow you still need to reproduce the dual-socket state
machine from `js/1.js`. This module focuses on the crypto pieces.

Other commands:
    $ python3 js/ws_adapter_client.py decode 'H4sIA...'
    $ python3 js/ws_adapter_client.py stream --wallet <PUBKEY> --input-mint So11111111111111111111111111111111111111112 \\
          --output-mint Es9vMFrzaCERhDWV... --amount 1000000
"""
from __future__ import annotations

DEFAULT_CURL_COMMAND = """curl 'wss://api.titan.exchange/api/v1/ws?auth=eyJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3NjA1Mzc4NTUsImV4cCI6MTc2MDUzODc1NSwic3ViIjoiZ2VuZXJpY19mcm9udGVuZF91c2VyIiwiYXVkIjoiYXBpLnRpdGFuLmFnIiwiaXNzIjoiaHR0cHM6Ly9qd3QtYXV0aC13b3JrZXItcHJvZC5kZWxpY2F0ZS1zaWxlbmNlLTE2Nzcud29ya2Vycy5kZXYvIiwiaHR0cHM6Ly9hcGkudGl0YW4uYWcvdXBrX2I1OCI6IlRpdGFuMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTEifQ.MTjdbNaVsnxaltPoKHyfPapzfGk1i2kMYu3sTx99x_g' \\
  -H 'Upgrade: websocket' \\
  -H 'Origin: https://app.titan.exchange' \\
  -H 'Cache-Control: no-cache' \\
  -H 'Accept-Language: zh-CN,zh;q=0.9' \\
  -H 'Pragma: no-cache' \\
  -H 'Connection: Upgrade' \\
  -H 'Sec-WebSocket-Key: ratHBQLbVU+9dSo+WDfwXQ==' \\
  -H 'User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36' \\
  -H 'Sec-WebSocket-Version: 13' \\
  -H 'Sec-WebSocket-Protocol: v1.api.titan.ag+gzip, v1.api.titan.ag' \\
  -H 'Sec-WebSocket-Extensions: permessage-deflate; client_max_window_bits'"""

import argparse
import asyncio
import base64
import contextlib
import gzip
import json
import os
import signal
import struct
import time
import urllib.parse
import urllib.request
from dataclasses import dataclass
from typing import Any, Callable, Dict, List, Optional, Tuple, Union

try:
    import msgpack  # type: ignore
except ImportError:  # pragma: no cover - optional dependency
    msgpack = None  # type: ignore[assignment]

try:
    import websockets
except ImportError:  # pragma: no cover - optional dependency
    websockets = None  # type: ignore[assignment]

from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.hkdf import HKDF


class SequenceOverflow(ValueError):
    """Outbound sequence exceeded 32-bit unsigned range."""


class SequenceMismatch(ValueError):
    """Inbound sequence number does not match expected monotonic counter."""


@dataclass
class KeyPair:
    private_key: ec.EllipticCurvePrivateKey
    public_key_bytes: bytes  # Uncompressed 65-byte form.

    @classmethod
    def generate(cls) -> "KeyPair":
        private_key = ec.generate_private_key(ec.SECP256R1())
        public_key_bytes = private_key.public_key().public_bytes(
            serialization.Encoding.X962,
            serialization.PublicFormat.UncompressedPoint,
        )
        return cls(private_key=private_key, public_key_bytes=public_key_bytes)


def sign_public_key(message: bytes, signer: ec.EllipticCurvePrivateKey) -> bytes:
    """ECDSA-SHA256 signature over bytes, matches `function u` in js/1.js."""
    signature = signer.sign(message, ec.ECDSA(hashes.SHA256()))
    return signature


def build_association_payload(
    association_keypair: KeyPair,
    signing_keypair: KeyPair,
) -> bytes:
    """
    Construct the association payload that the JS client sends first:
    raw public key concatenated with its ECDSA signature.
    """
    signature = sign_public_key(
        association_keypair.public_key_bytes,
        signing_keypair.private_key,
    )
    return association_keypair.public_key_bytes + signature


def hkdf_aes_key(shared_secret: bytes, salt: bytes) -> AESGCM:
    """
    Mirror of `function A` in js/1.js: HKDF with SHA-256, salt = association public key.
    The JS code requests a 128-bit AES-GCM key.
    """
    hkdf = HKDF(
        algorithm=hashes.SHA256(),
        length=16,
        salt=salt,
        info=b"",
    )
    aes_key = hkdf.derive(shared_secret)
    return AESGCM(aes_key)


def b64_url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).decode("ascii").rstrip("=")


_BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
_BASE58_LOOKUP = {ch: idx for idx, ch in enumerate(_BASE58_ALPHABET)}

RESET_COLOR = "\033[0m"
CLIENT_COLOR = "\033[92m"
SERVER_COLOR = "\033[91m"


def _print_direction(prefix: str, color: str, message: str) -> None:
    print(f"{color}{prefix}{RESET_COLOR} {message}")


def _print_client(message: str) -> None:
    _print_direction("client ↑", CLIENT_COLOR, message)


def _print_server(message: str) -> None:
    _print_direction("server ↓", SERVER_COLOR, message)


def base58_decode(value: str) -> bytes:
    """Decode a base58 string into bytes (no checksum handling)."""
    value = value.strip()
    if not value:
        return b""

    num = 0
    for char in value:
        try:
            num = num * 58 + _BASE58_LOOKUP[char]
        except KeyError as exc:
            raise ValueError(f"Invalid base58 character: {char!r}") from exc

    # Convert number to bytes
    byte_length = (num.bit_length() + 7) // 8
    data = num.to_bytes(byte_length, "big") if byte_length else b""

    # Handle leading zeroes encoded as '1'
    pad = len(value) - len(value.lstrip("1"))
    return b"\x00" * pad + data.lstrip(b"\x00")


def base58_encode(data: bytes) -> str:
    """Encode bytes into base58 (no checksum)."""
    if not data:
        return ""
    num = int.from_bytes(data, "big")
    encoded = ""
    while num:
        num, rem = divmod(num, 58)
        encoded = _BASE58_ALPHABET[rem] + encoded
    # Preserve leading zeroes
    pad = len(data) - len(data.lstrip(b"\x00"))
    return "1" * pad + encoded or "1"


def _fallback_unpackb(data: bytes, *, raw: bool) -> Any:
    """Minimal msgpack decoder covering Jupiter websocket payloads."""

    def read(offset: int, length: int) -> Tuple[bytes, int]:
        end = offset + length
        if end > len(data):
            raise ValueError("Unexpected end of data while decoding msgpack")
        return data[offset:end], end

    def unpack(offset: int) -> Tuple[Any, int]:
        if offset >= len(data):
            raise ValueError("Unexpected end of data while decoding msgpack")
        first = data[offset]
        offset += 1

        # positive fixint
        if first <= 0x7F:
            return first, offset
        # fixmap
        if 0x80 <= first <= 0x8F:
            size = first & 0x0F
            result = {}
            for _ in range(size):
                key, offset = unpack(offset)
                value, offset = unpack(offset)
                result[key] = value
            return result, offset
        # fixarray
        if 0x90 <= first <= 0x9F:
            size = first & 0x0F
            result = []
            for _ in range(size):
                item, offset = unpack(offset)
                result.append(item)
            return result, offset
        # fixstr
        if 0xA0 <= first <= 0xBF:
            length = first & 0x1F
            raw_bytes, offset = read(offset, length)
            if raw:
                return raw_bytes, offset
            return raw_bytes.decode("utf-8"), offset
        # nil
        if first == 0xC0:
            return None, offset
        # false / true
        if first == 0xC2:
            return False, offset
        if first == 0xC3:
            return True, offset
        # bin 8/16/32
        if first == 0xC4:
            length = data[offset]
            offset += 1
            raw_bytes, offset = read(offset, length)
            return raw_bytes, offset
        if first == 0xC5:
            length = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            raw_bytes, offset = read(offset, length)
            return raw_bytes, offset
        if first == 0xC6:
            length = struct.unpack(">I", data[offset:offset + 4])[0]
            offset += 4
            raw_bytes, offset = read(offset, length)
            return raw_bytes, offset
        # float32 / float64
        if first == 0xCA:
            value = struct.unpack(">f", data[offset:offset + 4])[0]
            offset += 4
            return value, offset
        if first == 0xCB:
            value = struct.unpack(">d", data[offset:offset + 8])[0]
            offset += 8
            return value, offset
        # uint 8/16/32/64
        if first == 0xCC:
            value = data[offset]
            offset += 1
            return value, offset
        if first == 0xCD:
            value = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            return value, offset
        if first == 0xCE:
            value = struct.unpack(">I", data[offset:offset + 4])[0]
            offset += 4
            return value, offset
        if first == 0xCF:
            value = struct.unpack(">Q", data[offset:offset + 8])[0]
            offset += 8
            return value, offset
        # int 8/16/32/64
        if first == 0xD0:
            value = struct.unpack("b", data[offset:offset + 1])[0]
            offset += 1
            return value, offset
        if first == 0xD1:
            value = struct.unpack(">h", data[offset:offset + 2])[0]
            offset += 2
            return value, offset
        if first == 0xD2:
            value = struct.unpack(">i", data[offset:offset + 4])[0]
            offset += 4
            return value, offset
        if first == 0xD3:
            value = struct.unpack(">q", data[offset:offset + 8])[0]
            offset += 8
            return value, offset
        # str8/16/32
        if first == 0xD9:
            length = data[offset]
            offset += 1
            raw_bytes, offset = read(offset, length)
            return raw_bytes if raw else raw_bytes.decode("utf-8"), offset
        if first == 0xDA:
            length = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            raw_bytes, offset = read(offset, length)
            return raw_bytes if raw else raw_bytes.decode("utf-8"), offset
        if first == 0xDB:
            length = struct.unpack(">I", data[offset:offset + 4])[0]
            offset += 4
            raw_bytes, offset = read(offset, length)
            return raw_bytes if raw else raw_bytes.decode("utf-8"), offset
        # array16/32
        if first == 0xDC:
            count = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            result = []
            for _ in range(count):
                item, offset = unpack(offset)
                result.append(item)
            return result, offset
        if first == 0xDD:
            count = struct.unpack(">I", data[offset:offset + 4])[0]
            offset += 4
            result = []
            for _ in range(count):
                item, offset = unpack(offset)
                result.append(item)
            return result, offset
        # map16/32
        if first == 0xDE:
            count = struct.unpack(">H", data[offset:offset + 2])[0]
            offset += 2
            result = {}
            for _ in range(count):
                key, offset = unpack(offset)
                value, offset = unpack(offset)
                result[key] = value
            return result, offset
        if first == 0xDF:
            count = struct.unpack(">I", data[offset:offset + 4])[0]
            offset += 4
            result = {}
            for _ in range(count):
                key, offset = unpack(offset)
                value, offset = unpack(offset)
                result[key] = value
            return result, offset
        # negative fixint
        if first >= 0xE0:
            return struct.unpack("b", bytes([first]))[0], offset

        raise ValueError(f"Unsupported msgpack byte: 0x{first:02x}")

    value, cursor = unpack(0)
    if cursor != len(data):
        # There may be multiple objects; we only decode the first entirely.
        pass
    return value


def decode_gzip_msgpack(data: Union[str, bytes], *, raw: bool = False) -> Any:
    """
    Decode Jupiter websocket frames that are sent as gzip(msgpack).

    Args:
        data: Base64 string or raw bytes.
        raw: When True, keep msgpack raw bytes (raw=True). Defaults to False.
    """
    if isinstance(data, str):
        payload = base64.b64decode(data)
    else:
        payload = data

    decompressed = gzip.decompress(payload)
    return decode_msgpack_bytes(decompressed, raw=raw)


def decode_msgpack_bytes(data: bytes, *, raw: bool = False) -> Any:
    if msgpack is not None:
        return msgpack.unpackb(data, raw=raw)
    return _fallback_unpackb(data, raw=raw)


def encode_msgpack(obj: Any) -> bytes:
    """Serialize Python primitives to msgpack (small subset used by Titan WS)."""
    buffer = bytearray()
    _encode_msgpack_value(obj, buffer)
    return bytes(buffer)


def _encode_msgpack_value(obj: Any, buffer: bytearray) -> None:
    if obj is None:
        buffer.append(0xC0)
    elif obj is False:
        buffer.append(0xC2)
    elif obj is True:
        buffer.append(0xC3)
    elif isinstance(obj, int):
        _encode_msgpack_int(obj, buffer)
    elif isinstance(obj, float):
        buffer.append(0xCB)
        buffer.extend(struct.pack(">d", obj))
    elif isinstance(obj, str):
        encoded = obj.encode("utf-8")
        _encode_msgpack_str_header(len(encoded), buffer)
        buffer.extend(encoded)
    elif isinstance(obj, (bytes, bytearray, memoryview)):
        data = bytes(obj)
        _encode_msgpack_bin_header(len(data), buffer)
        buffer.extend(data)
    elif isinstance(obj, dict):
        items = list(obj.items())
        _encode_msgpack_map_header(len(items), buffer)
        for key, value in items:
            if not isinstance(key, (str, int)):
                raise TypeError(f"msgpack map keys must be str or int, got {type(key)!r}")
            _encode_msgpack_value(key, buffer)
            _encode_msgpack_value(value, buffer)
    elif isinstance(obj, (list, tuple)):
        _encode_msgpack_array_header(len(obj), buffer)
        for item in obj:
            _encode_msgpack_value(item, buffer)
    else:
        raise TypeError(f"Unsupported type for msgpack encoding: {type(obj)!r}")


def _encode_msgpack_int(value: int, buffer: bytearray) -> None:
    if 0 <= value <= 0x7F:
        buffer.append(value)
    elif -32 <= value < 0:
        buffer.append(0xE0 | (value + 32))
    elif 0 <= value <= 0xFF:
        buffer.extend((0xCC, value))
    elif 0 <= value <= 0xFFFF:
        buffer.append(0xCD)
        buffer.extend(struct.pack(">H", value))
    elif 0 <= value <= 0xFFFFFFFF:
        buffer.append(0xCE)
        buffer.extend(struct.pack(">I", value))
    elif 0 <= value <= 0xFFFFFFFFFFFFFFFF:
        buffer.append(0xCF)
        buffer.extend(struct.pack(">Q", value))
    elif -0x80 <= value < 0:
        buffer.append(0xD0)
        buffer.extend(struct.pack("b", value))
    elif -0x8000 <= value < -0x80:
        buffer.append(0xD1)
        buffer.extend(struct.pack(">h", value))
    elif -0x80000000 <= value < -0x8000:
        buffer.append(0xD2)
        buffer.extend(struct.pack(">i", value))
    elif -0x8000000000000000 <= value < -0x80000000:
        buffer.append(0xD3)
        buffer.extend(struct.pack(">q", value))
    else:
        raise OverflowError(f"Integer out of supported range for msgpack: {value}")


def _encode_msgpack_str_header(length: int, buffer: bytearray) -> None:
    if length < 32:
        buffer.append(0xA0 | length)
    elif length < 0x100:
        buffer.extend((0xD9, length))
    elif length < 0x10000:
        buffer.append(0xDA)
        buffer.extend(struct.pack(">H", length))
    elif length < 0x100000000:
        buffer.append(0xDB)
        buffer.extend(struct.pack(">I", length))
    else:
        raise ValueError(f"String too long for msgpack: {length} bytes")


def _encode_msgpack_bin_header(length: int, buffer: bytearray) -> None:
    if length < 0x100:
        buffer.extend((0xC4, length))
    elif length < 0x10000:
        buffer.append(0xC5)
        buffer.extend(struct.pack(">H", length))
    elif length < 0x100000000:
        buffer.append(0xC6)
        buffer.extend(struct.pack(">I", length))
    else:
        raise ValueError(f"Binary too long for msgpack: {length} bytes")


def _encode_msgpack_array_header(length: int, buffer: bytearray) -> None:
    if length < 16:
        buffer.append(0x90 | length)
    elif length < 0x10000:
        buffer.append(0xDC)
        buffer.extend(struct.pack(">H", length))
    elif length < 0x100000000:
        buffer.append(0xDD)
        buffer.extend(struct.pack(">I", length))
    else:
        raise ValueError(f"Array too large for msgpack: {length}")


def _encode_msgpack_map_header(length: int, buffer: bytearray) -> None:
    if length < 16:
        buffer.append(0x80 | length)
    elif length < 0x10000:
        buffer.append(0xDE)
        buffer.extend(struct.pack(">H", length))
    elif length < 0x100000000:
        buffer.append(0xDF)
        buffer.extend(struct.pack(">I", length))
    else:
        raise ValueError(f"Map too large for msgpack: {length}")




@dataclass
class AssociationContext:
    """
    Holds both keypairs used during the handshake.

    association_keypair: ECDH P-256 key (raw bytes are sent to peer).
    signing_keypair: ECDSA P-256 key for signing the association public key.
    """

    association_keypair: KeyPair
    signing_keypair: KeyPair

    @classmethod
    def generate(cls) -> "AssociationContext":
        return cls(KeyPair.generate(), KeyPair.generate())

    @property
    def public_key_bytes(self) -> bytes:
        return self.association_keypair.public_key_bytes

    def association_payload(self) -> bytes:
        return build_association_payload(self.association_keypair, self.signing_keypair)

    def association_payload_b64url(self) -> str:
        return b64_url(self.association_payload())


class SessionCrypto:
    """
    AES-GCM helper mirroring the adapter framing format.

    * Outbound frames: [4 byte seq][12 byte IV][ciphertext]
    * AAD = sequence number, stored big-endian
    * Plaintext = JSON payload (UTF-8 bytes)
    """

    def __init__(self, aesgcm: AESGCM, initial_inbound: int = 0, initial_outbound: int = 0):
        self._aesgcm = aesgcm
        self._inbound_seq = initial_inbound
        self._outbound_seq = initial_outbound

    @classmethod
    def from_handshake(
        cls,
        association: AssociationContext,
        peer_public_key: bytes,
        *,
        salt: Optional[bytes] = None,
        initial_inbound: int = 0,
        initial_outbound: int = 0,
    ) -> "SessionCrypto":
        peer_key = ec.EllipticCurvePublicKey.from_encoded_point(ec.SECP256R1(), peer_public_key)
        shared_secret = association.association_keypair.private_key.exchange(ec.ECDH(), peer_key)
        salt_bytes = salt if salt is not None else association.public_key_bytes
        aesgcm = hkdf_aes_key(shared_secret, salt_bytes)
        return cls(aesgcm, initial_inbound=initial_inbound, initial_outbound=initial_outbound)

    @staticmethod
    def _seq_to_bytes(seq: int) -> bytes:
        if seq >= 0x100000000:
            raise SequenceOverflow(f"sequence {seq} exceeds 32-bit unsigned range")
        return struct.pack(">I", seq)

    def encrypt_json(self, payload: Dict[str, Any]) -> bytes:
        self._outbound_seq += 1
        seq_bytes = self._seq_to_bytes(self._outbound_seq)
        iv = os.urandom(12)
        plaintext = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        ciphertext = self._aesgcm.encrypt(iv, plaintext, seq_bytes)
        return seq_bytes + iv + ciphertext

    def decrypt_json(self, frame: bytes) -> Dict[str, Any]:
        if len(frame) < 4 + 12 + 16:
            raise ValueError("frame too short to contain AES-GCM payload")
        seq_bytes = frame[:4]
        iv = frame[4:16]
        ciphertext = frame[16:]
        seq = struct.unpack(">I", seq_bytes)[0]
        expected = self._inbound_seq + 1
        if seq != expected:
            raise SequenceMismatch(f"expected seq {expected}, got {seq}")
        plaintext = self._aesgcm.decrypt(iv, ciphertext, seq_bytes)
        self._inbound_seq = seq
        return json.loads(plaintext.decode("utf-8"))


# Optional async demo utilities.
DEFAULT_TITAN_ENDPOINT = "wss://api.titan.exchange/api/v1/ws"
DEFAULT_JWT_ENDPOINT = "https://titan.exchange/api/apollo-jwt"
DEFAULT_REFERER_TEMPLATE = "https://titan.exchange/swap?{input}-{output}"
DEFAULT_USER_AGENT = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36"
)
DEFAULT_WALLET = "P1ayvj9UdRxiVWa6rxXbqmosF1tZf62VZSJnrUnvhak"
DEFAULT_INPUT_MINT = "So11111111111111111111111111111111111111112"
DEFAULT_OUTPUT_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
DEFAULT_AMOUNT = 1 * 1_000_000_000


class TitanWsClient:
    """Minimal Titan quote WebSocket client (gzip + msgpack)."""

    def __init__(
        self,
        endpoint: str = DEFAULT_TITAN_ENDPOINT,
        *,
        subprotocols: Optional[Tuple[str, ...]] = None,
        request_timeout: float = 10.0,
    ) -> None:
        self.endpoint = endpoint
        self.subprotocols = subprotocols or ("v1.api.titan.ag+gzip", "v1.api.titan.ag")
        self.request_timeout = request_timeout
        self._ws: Optional["websockets.WebSocketClientProtocol"] = None
        self._use_gzip = False
        self._recv_task: Optional[asyncio.Task[None]] = None
        self._pending: Dict[int, asyncio.Future[Any]] = {}
        self._stream_queues: Dict[int, asyncio.Queue[Any]] = {}
        self._next_request_id = 1

    async def __aenter__(self) -> "TitanWsClient":
        await self.connect()
        return self

    async def __aexit__(self, exc_type, exc, tb) -> None:
        await self.close()

    async def connect(self) -> None:
        if self._ws:
            return
        if websockets is None:
            raise RuntimeError("websockets is required: pip install websockets")
        self._ws = await websockets.connect(self.endpoint, subprotocols=list(self.subprotocols))
        protocol = self._ws.subprotocol or ""
        self._use_gzip = "gzip" in protocol.lower()
        self._recv_task = asyncio.create_task(self._recv_loop())

    async def close(self) -> None:
        if self._recv_task:
            self._recv_task.cancel()
            with contextlib.suppress(asyncio.CancelledError):
                await self._recv_task
        self._recv_task = None
        if self._ws:
            await self._ws.close()
        self._ws = None
        for future in list(self._pending.values()):
            if not future.done():
                future.set_exception(RuntimeError("connection closed"))
        self._pending.clear()
        for queue in self._stream_queues.values():
            with contextlib.suppress(asyncio.QueueFull):
                queue.put_nowait({"error": "connection_closed"})
        self._stream_queues.clear()

    async def _recv_loop(self) -> None:
        assert self._ws is not None
        try:
            async for raw_message in self._ws:
                if isinstance(raw_message, str):
                    # Protocol only uses binary frames.
                    continue
                payload = gzip.decompress(raw_message) if self._use_gzip else raw_message
                try:
                    message = decode_msgpack_bytes(payload)
                except Exception as exc:
                    # Continue but surface to pending callers.
                    for future in list(self._pending.values()):
                        if not future.done():
                            future.set_exception(exc)
                    continue
                self._handle_message(message)
        except asyncio.CancelledError:
            pass
        except Exception as exc:
            for future in list(self._pending.values()):
                if not future.done():
                    future.set_exception(exc)
        finally:
            # Notify listeners that the stream ended.
            for future in list(self._pending.values()):
                if not future.done():
                    future.set_exception(RuntimeError("connection closed"))
            self._pending.clear()
            for stream_id, queue in list(self._stream_queues.items()):
                with contextlib.suppress(asyncio.QueueFull):
                    queue.put_nowait({"StreamEnd": {"id": stream_id, "reason": "connection_closed"}})
            self._stream_queues.clear()

    def _handle_message(self, message: Dict[str, Any]) -> None:
        if "Response" in message:
            response = message["Response"]
            request_id = response.get("requestId")
            if isinstance(request_id, int):
                future = self._pending.pop(request_id, None)
                if future and not future.done():
                    future.set_result(response)
        elif "Error" in message:
            error = message["Error"]
            request_id = error.get("requestId")
            err = RuntimeError(f"{error.get('code')}: {error.get('message')}")
            if isinstance(request_id, int):
                future = self._pending.pop(request_id, None)
                if future and not future.done():
                    future.set_exception(err)
            else:
                for future in list(self._pending.values()):
                    if not future.done():
                        future.set_exception(err)
        elif "StreamData" in message:
            data = message["StreamData"]
            stream_id = data.get("id")
            if isinstance(stream_id, int):
                queue = self._stream_queues.get(stream_id)
                if queue:
                    with contextlib.suppress(asyncio.QueueFull):
                        queue.put_nowait(data)
        elif "StreamEnd" in message:
            end = message["StreamEnd"]
            stream_id = end.get("id")
            if isinstance(stream_id, int):
                queue = self._stream_queues.pop(stream_id, None)
                if queue:
                    with contextlib.suppress(asyncio.QueueFull):
                        queue.put_nowait({"StreamEnd": end})

    def _allocate_request_id(self) -> int:
        request_id = self._next_request_id
        self._next_request_id = request_id + 1 if request_id < 999999 else 1
        return request_id

    async def _send(self, message: Dict[str, Any]) -> None:
        if self._ws is None:
            raise RuntimeError("WebSocket not connected")
        payload = encode_msgpack(message)
        if self._use_gzip:
            payload = gzip.compress(payload)
        await self._ws.send(payload)

    async def send_request(self, data: Dict[str, Any], *, timeout: Optional[float] = None) -> Dict[str, Any]:
        await self.connect()
        request_id = self._allocate_request_id()
        loop = asyncio.get_running_loop()
        future: asyncio.Future[Any] = loop.create_future()
        self._pending[request_id] = future
        await self._send({"id": request_id, "data": data})
        try:
            effective_timeout = timeout if timeout is not None else self.request_timeout
            response = await asyncio.wait_for(future, timeout=effective_timeout)
        finally:
            self._pending.pop(request_id, None)
        return response

    async def get_info(self, *, timeout: Optional[float] = None) -> Dict[str, Any]:
        response = await self.send_request({"GetInfo": {}}, timeout=timeout)
        return response.get("data", {})

    async def get_venues(
        self,
        *,
        include_program_ids: bool = False,
        timeout: Optional[float] = None,
    ) -> Dict[str, Any]:
        request: Dict[str, Any] = {}
        if include_program_ids:
            request["includeProgramIds"] = True
        response = await self.send_request({"GetVenues": request}, timeout=timeout)
        return response.get("data", {})

    async def list_providers(
        self,
        *,
        include_icons: bool = False,
        timeout: Optional[float] = None,
    ) -> List[Dict[str, Any]]:
        request: Dict[str, Any] = {}
        if include_icons:
            request["includeIcons"] = True
        response = await self.send_request({"ListProviders": request}, timeout=timeout)
        return response.get("data", [])

    async def new_swap_quote_stream(
        self,
        *,
        input_mint: str,
        output_mint: str,
        amount: int,
        user_public_key: str,
        swap_mode: str = "ExactIn",
        slippage_bps: Optional[int] = 50,
        include_dexes: Optional[List[str]] = None,
        only_direct_routes: Optional[bool] = None,
        add_size_constraint: Optional[bool] = None,
        update_interval_ms: Optional[int] = 800,
        timeout: Optional[float] = None,
    ) -> Tuple[Dict[str, Any], asyncio.Queue[Any]]:
        swap: Dict[str, Any] = {
            "inputMint": base58_decode(input_mint),
            "outputMint": base58_decode(output_mint),
            "amount": int(amount),
        }
        if swap_mode:
            swap["swapMode"] = swap_mode
        if slippage_bps is not None:
            swap["slippageBps"] = int(slippage_bps)
        if include_dexes:
            swap["dexes"] = list(include_dexes)
        if only_direct_routes is not None:
            swap["onlyDirectRoutes"] = bool(only_direct_routes)
        if add_size_constraint is not None:
            swap["addSizeConstraint"] = bool(add_size_constraint)

        transaction: Dict[str, Any] = {
            "userPublicKey": base58_decode(user_public_key),
        }

        request_payload: Dict[str, Any] = {
            "swap": swap,
            "transaction": transaction,
            "update": {"intervalMs": int(update_interval_ms)} if update_interval_ms is not None else None,
        }

        response = await self.send_request({"NewSwapQuoteStream": request_payload}, timeout=timeout)
        stream_info = response.get("stream", {})
        stream_id = stream_info.get("id")
        queue: asyncio.Queue[Any] = asyncio.Queue()
        if isinstance(stream_id, int):
            self._stream_queues[stream_id] = queue
        return response, queue

    async def stop_stream(self, stream_id: int, *, timeout: Optional[float] = None) -> Dict[str, Any]:
        response = await self.send_request({"StopStream": {"id": int(stream_id)}}, timeout=timeout)
        self._stream_queues.pop(stream_id, None)
        return response


async def demo_round_trip() -> None:
    """
    Run an in-memory demo that encrypts/decrypts a JSON-RPC payload.
    Useful to quickly verify the crypto pipeline works end-to-end.
    """
    ctx_a = AssociationContext.generate()
    ctx_b = AssociationContext.generate()

    # In the adapter protocol the salt is the association public key supplied
    # during the initial pairing. For this self-contained demo we reuse one side.
    salt = ctx_a.public_key_bytes
    session_a = SessionCrypto.from_handshake(ctx_a, ctx_b.public_key_bytes, salt=salt)
    session_b = SessionCrypto.from_handshake(ctx_b, ctx_a.public_key_bytes, salt=salt)

    payload = {"jsonrpc": "2.0", "id": 1, "method": "ping"}
    frame = session_a.encrypt_json(payload)
    result = session_b.decrypt_json(frame)
    assert result == payload
    print("Encrypted frame:", frame.hex())
    print("Decrypted payload:", result)


def _json_default(value: Any) -> Any:
    if isinstance(value, (bytes, bytearray)):
        return _bytes_repr(value)
    raise TypeError(f"Object of type {type(value).__name__} is not JSON serializable")


def _bytes_repr(value: bytes) -> Dict[str, Any]:
    """Provide base58 + base64 encodings to keep dumps readable."""
    return {
        "type": "bytes",
        "base58": base58_encode(value),
        "base64": base64.b64encode(value).decode("ascii"),
        "length": len(value),
    }


def _decode_stream_payload(message: Any) -> Any:
    """Best-effort conversion of Titan stream payloads into human-readable form."""
    if isinstance(message, dict):
        if "SwapQuotes" in message:
            return {"SwapQuotes": _decode_swap_quotes(message["SwapQuotes"])}
        if "StreamData" in message:
            data = dict(message)
            data["StreamData"] = _decode_stream_payload(message["StreamData"])
            return data
        return {k: _decode_stream_payload(v) for k, v in message.items()}
    if isinstance(message, list):
        return [_decode_stream_payload(item) for item in message]
    if isinstance(message, (bytes, bytearray)):
        return base58_encode(message)
    return message


def _decode_swap_quotes(quotes: Dict[str, Any]) -> Dict[str, Any]:
    decoded = {}
    for key, value in quotes.items():
        if isinstance(value, (bytes, bytearray)):
            decoded[key] = base58_encode(value)
        elif isinstance(value, dict):
            decoded[key] = _decode_stream_payload(value)
        else:
            decoded[key] = value
    return decoded


def _build_referer_url(template: str, input_mint: str, output_mint: str) -> str:
    try:
        return template.format(input=input_mint, output=output_mint)
    except Exception:
        return template


def _append_address_param(endpoint: str, address: str) -> str:
    if not address:
        return endpoint
    parsed = urllib.parse.urlsplit(endpoint)
    query = urllib.parse.parse_qsl(parsed.query, keep_blank_values=True)
    query = [(k, v) for (k, v) in query if k != "address"]
    query.append(("address", address))
    new_query = urllib.parse.urlencode(query)
    return urllib.parse.urlunsplit(
        (parsed.scheme, parsed.netloc, parsed.path, new_query, parsed.fragment)
    )


def _fetch_jwt_sync(
    address: str,
    endpoint: str,
    referer: str,
    user_agent: str,
) -> Tuple[str, Dict[str, Any]]:
    url = _append_address_param(endpoint, address)
    headers = {
        "User-Agent": user_agent,
        "Referer": referer,
        "Accept": "application/json",
    }
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=10) as resp:
        body = resp.read()
    data = json.loads(body.decode("utf-8"))
    token = data.get("token")
    if not token:
        raise RuntimeError("JWT response missing token field")
    return token, data


def _install_sigint_handler(
    loop: asyncio.AbstractEventLoop,
    handler: Callable[[], None],
) -> Callable[[], None]:
    """
    Install a SIGINT handler that routes Ctrl+C into the asyncio loop and
    returns a callback that restores the previous handler.
    """
    try:
        loop.add_signal_handler(signal.SIGINT, handler)
        return lambda: loop.remove_signal_handler(signal.SIGINT)
    except (NotImplementedError, RuntimeError):
        previous = signal.getsignal(signal.SIGINT)

        def _sync_handler(_: int, __: Any) -> None:
            handler()

        signal.signal(signal.SIGINT, _sync_handler)

        def _restore() -> None:
            signal.signal(signal.SIGINT, previous)

        return _restore


def main(argv: Optional[Any] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command")

    parser_decode = sub.add_parser(
        "decode", help="Decode a base64 encoded gzip(msgpack) payload"
    )
    parser_decode.add_argument("payload", help="Base64 string (e.g. starts with H4sI...)")
    parser_decode.add_argument(
        "--keep-raw",
        action="store_true",
        help="Preserve msgpack raw bytes (raw=True); default decodes as UTF-8 strings",
    )

    parser_stream = sub.add_parser(
        "stream", help="Connect to Titan WS and stream swap quotes"
    )
    parser_stream.add_argument("--endpoint", default=DEFAULT_TITAN_ENDPOINT)
    parser_stream.add_argument(
        "--wallet",
        default=DEFAULT_WALLET,
        help=f"Base58 user public key (default: {DEFAULT_WALLET})",
    )
    parser_stream.add_argument(
        "--input-mint",
        default=DEFAULT_INPUT_MINT,
        help=f"Base58 mint to swap from (default: {DEFAULT_INPUT_MINT})",
    )
    parser_stream.add_argument(
        "--output-mint",
        default=DEFAULT_OUTPUT_MINT,
        help=f"Base58 mint to swap into (default: {DEFAULT_OUTPUT_MINT})",
    )
    parser_stream.add_argument(
        "--amount",
        type=int,
        default=DEFAULT_AMOUNT,
        help=f"Input amount in base units (default: {DEFAULT_AMOUNT})",
    )
    parser_stream.add_argument(
        "--swap-mode",
        choices=("ExactIn", "ExactOut"),
        default="ExactIn",
        help="Swap mode (default: ExactIn)",
    )
    parser_stream.add_argument("--slippage-bps", type=int, default=50)
    parser_stream.add_argument("--interval", type=int, default=800, help="Update interval in ms")
    parser_stream.add_argument(
        "--dex",
        dest="dexes",
        action="append",
        help="Restrict to specific DEX identifiers (can repeat)",
    )
    parser_stream.add_argument(
        "--only-direct-routes",
        action="store_true",
        help="Request only direct routes (no routed aggregations)",
    )
    parser_stream.add_argument(
        "--no-size-constraint",
        action="store_true",
        help="Disable size constraint flag (enabled by default in UI)",
    )
    parser_stream.add_argument(
        "--duration",
        type=float,
        default=0.0,
        help="Optional cap in seconds for how long to keep the stream open (0 = unlimited)",
    )
    parser_stream.add_argument(
        "--max-messages",
        type=int,
        default=0,
        help="Stop after this many StreamData payloads (0 = unlimited)",
    )
    parser_stream.add_argument(
        "--timeout",
        type=float,
        default=10.0,
        help="Per-request timeout in seconds (default: 10)",
    )
    parser_stream.add_argument(
        "--idle-timeout",
        type=float,
        default=0.0,
        help="Exit if no StreamData arrives within this many seconds (0 = wait indefinitely)",
    )
    parser_stream.add_argument(
        "--print-request",
        action="store_true",
        help="Dump the SwapQuoteRequest payload before sending",
    )
    parser_stream.add_argument(
        "--raw-stream",
        action="store_true",
        help="Print StreamData payloads without decoding bytes to base58",
    )
    parser_stream.add_argument(
        "--jwt",
        help="Explicit JWT token for wss auth (skip auto-fetch when provided)",
    )

    sub.add_parser("demo", help="Run AES-GCM round-trip demo (default)")

    args = parser.parse_args(argv)
    if args.command == "decode":
        result = decode_gzip_msgpack(args.payload, raw=args.keep_raw)
        print(json.dumps(result, indent=2, default=_json_default))
        return 0
    if args.command == "stream":
        async def run_stream() -> None:
            ws_endpoint = args.endpoint
            jwt_token = args.jwt
            if "auth=" not in ws_endpoint and not jwt_token:
                referer = _build_referer_url(
                    DEFAULT_REFERER_TEMPLATE, args.input_mint, args.output_mint
                )
                try:
                    jwt_token, metadata = await asyncio.to_thread(
                        _fetch_jwt_sync,
                        args.wallet,
                        DEFAULT_JWT_ENDPOINT,
                        referer,
                        DEFAULT_USER_AGENT,
                    )
                    expires = metadata.get("expires_at") or metadata.get("expires_in")
                    _print_server(
                        f"Fetched JWT for {args.wallet} "
                        f"(expires: {expires})"
                    )
                except Exception as exc:  # pragma: no cover - network error
                    raise RuntimeError(f"Failed to fetch JWT: {exc}") from exc
            if jwt_token and "auth=" not in ws_endpoint:
                separator = "&" if "?" in ws_endpoint else "?"
                ws_endpoint = f"{ws_endpoint}{separator}auth={jwt_token}"

            client = TitanWsClient(endpoint=ws_endpoint, request_timeout=args.timeout)
            loop = asyncio.get_running_loop()
            main_task = asyncio.current_task()
            cancelled_by_signal = False
            remove_sigint_handler: Callable[[], None] = lambda: None
            if main_task is not None:
                def _request_shutdown() -> None:
                    nonlocal cancelled_by_signal
                    cancelled_by_signal = True
                    loop.call_soon_threadsafe(main_task.cancel)

                remove_sigint_handler = _install_sigint_handler(loop, _request_shutdown)
            try:
                async with client:
                    info = await client.get_info(timeout=args.timeout)
                    _print_server("Server info:")
                    print(json.dumps(info, indent=2, default=_json_default))
                    try:
                        venues = await client.get_venues(
                            include_program_ids=True, timeout=args.timeout
                        )
                        _print_server("GetVenues - 支持的 dex 标签：")
                        print(json.dumps(venues, indent=2, default=_json_default))
                    except Exception as exc:  # pragma: no cover - network error
                        _print_server(f"获取 GetVenues 失败: {exc}")
                    try:
                        providers = await client.list_providers(timeout=args.timeout)
                        _print_server("ListProviders - 已配置的 providers：")
                        print(json.dumps(providers, indent=2, default=_json_default))
                    except Exception as exc:  # pragma: no cover - network error
                        _print_server(f"获取 ListProviders 失败: {exc}")
                    request_payload = {
                        "swap": {
                            "inputMint": args.input_mint,
                            "outputMint": args.output_mint,
                            "amount": args.amount,
                            "swapMode": args.swap_mode,
                            "slippageBps": args.slippage_bps,
                            "dexes": args.dexes,
                            "onlyDirectRoutes": args.only_direct_routes,
                            "addSizeConstraint": not args.no_size_constraint,
                        },
                        "transaction": {"userPublicKey": args.wallet},
                    }
                    update_params = {}
                    if args.interval is not None:
                        update_params["intervalMs"] = args.interval
                    if update_params:
                        request_payload["update"] = update_params

                    if args.print_request:
                        _print_client("SwapQuoteRequest 参数 (调试显示)")
                        print(json.dumps(request_payload, indent=2))

                    _print_client("发送订阅请求 (SwapQuoteRequest)")
                    print(json.dumps(request_payload, indent=2))
                    response, queue = await client.new_swap_quote_stream(
                        input_mint=args.input_mint,
                        output_mint=args.output_mint,
                        amount=args.amount,
                        user_public_key=args.wallet,
                        swap_mode=args.swap_mode,
                        slippage_bps=args.slippage_bps,
                        include_dexes=args.dexes,
                        only_direct_routes=args.only_direct_routes,
                        add_size_constraint=not args.no_size_constraint,
                        update_interval_ms=args.interval,
                        timeout=args.timeout,
                    )
                    stream_id = response.get("stream", {}).get("id")
                    _print_server("Stream opened:")
                    print(json.dumps(response, indent=2, default=_json_default))

                    received = 0
                    start = time.monotonic()
                    deadline = start + args.duration if args.duration and args.duration > 0 else None
                    try:
                        while True:
                            now = time.monotonic()
                            remaining = (
                                max(0.0, deadline - now) if deadline is not None else None
                            )
                            timeouts = []
                            if remaining is not None:
                                timeouts.append(remaining)
                            if args.idle_timeout and args.idle_timeout > 0:
                                timeouts.append(args.idle_timeout)
                            wait_timeout: Optional[float] = min(timeouts) if timeouts else None
                            if wait_timeout is not None and wait_timeout <= 0:
                                print("Stream duration reached, exiting.")
                                break
                            try:
                                if wait_timeout is None:
                                    message = await queue.get()
                                else:
                                    message = await asyncio.wait_for(queue.get(), timeout=wait_timeout)
                            except asyncio.TimeoutError:
                                print("Quote stream timed out waiting for data.")
                                break
                            printable = message if args.raw_stream else _decode_stream_payload(message)
                            _print_server("Stream payload:")
                            print(json.dumps(printable, indent=2, default=_json_default))
                            if isinstance(message, dict) and "StreamEnd" in message:
                                break
                            received += 1
                            if args.max_messages and args.max_messages > 0 and received >= args.max_messages:
                                break
                    except asyncio.CancelledError:
                        if cancelled_by_signal:
                            print("\nInterrupted by user, closing stream...")
                        raise
                    finally:
                        if stream_id is not None:
                            try:
                                await client.stop_stream(stream_id, timeout=args.timeout)
                            except Exception as exc:  # pragma: no cover - best effort
                                print(f"Failed to stop stream cleanly: {exc}")
            except asyncio.CancelledError:
                if not cancelled_by_signal:
                    raise
            finally:
                remove_sigint_handler()

        asyncio.run(run_stream())
        return 0

    asyncio.run(demo_round_trip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
def _decode_stream_payload(message: Any) -> Any:
    """Best-effort conversion of Titan stream payloads into human-readable form."""
    if isinstance(message, dict):
        if "SwapQuotes" in message:
            return {"SwapQuotes": _decode_swap_quotes(message["SwapQuotes"])}
        if "StreamData" in message:
            data = dict(message)
            data["StreamData"] = _decode_stream_payload(message["StreamData"])
            return data
        return {k: _decode_stream_payload(v) for k, v in message.items()}
    if isinstance(message, list):
        return [_decode_stream_payload(item) for item in message]
    if isinstance(message, (bytes, bytearray)):
        return base58_encode(message)
    return message


def _decode_swap_quotes(quotes: Dict[str, Any]) -> Dict[str, Any]:
    decoded = {}
    for key, value in quotes.items():
        if isinstance(value, (bytes, bytearray)):
            decoded[key] = base58_encode(value)
        elif isinstance(value, dict):
            decoded[key] = _decode_stream_payload(value)
        else:
            decoded[key] = value
    return decoded
