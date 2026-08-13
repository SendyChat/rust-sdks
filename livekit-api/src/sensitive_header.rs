// Copyright 2026 SendyChat
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

use bytes::Bytes;
use http::header::{HeaderValue, InvalidHeaderValue};
use zeroize::Zeroizing;

const BEARER_PREFIX: &[u8] = b"Bearer ";

struct SensitiveHeaderBytes(Zeroizing<Vec<u8>>);

impl AsRef<[u8]> for SensitiveHeaderBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

/// Builds the unavoidable transport-owned header from an immediately-zeroized staging buffer.
///
/// `HeaderValue` and lower network layers do not offer a zeroizing drop path. The returned value
/// is therefore marked sensitive, kept request-scoped by callers, and documented as residual
/// transport custody. Workspace log features compile out tungstenite's raw-request trace.
pub(crate) fn bearer(token: &str) -> Result<HeaderValue, InvalidHeaderValue> {
    let mut staging = Zeroizing::new(Vec::with_capacity(BEARER_PREFIX.len() + token.len()));
    staging.extend_from_slice(BEARER_PREFIX);
    staging.extend_from_slice(token.as_bytes());

    // HeaderValue keeps the Bytes owner instead of allocating another ordinary copy. Cloned
    // HeaderValues share the same owner and the last drop erases its allocation.
    let mut header =
        HeaderValue::from_maybe_shared(Bytes::from_owner(SensitiveHeaderBytes(staging)))?;
    header.set_sensitive(true);
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "sendy-livekit-header-sentinel";

    #[test]
    fn bearer_header_is_sensitive_and_debug_redacted() {
        let header = bearer(SENTINEL).unwrap();

        assert!(header.is_sensitive());
        assert_eq!(header.as_bytes(), format!("Bearer {SENTINEL}").as_bytes());
        assert!(!format!("{header:?}").contains(SENTINEL));
    }

    #[test]
    fn dependency_trace_logging_is_compiled_out() {
        assert!(log::STATIC_MAX_LEVEL < log::LevelFilter::Trace);
    }
}
