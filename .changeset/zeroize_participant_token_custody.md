---
livekit: patch
livekit-api: patch
livekit-ffi: patch
---

Harden participant token custody by moving decoded refresh and room-move tokens into zeroizing owners, redacting diagnostic surfaces, and preventing plaintext token fan-out through public room events.
