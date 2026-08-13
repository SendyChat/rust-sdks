// Copyright 2026 SendyChat
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

use std::fmt;

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use zeroize::{Zeroize, Zeroizing};

#[cfg(test)]
static ZEROIZING_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

/// A participant token whose owned allocation is erased on replacement and drop.
///
/// Cloning is intentionally explicit at the type boundary: every clone is another
/// zeroizing owner, and `Debug`/`Display` never expose the plaintext.
pub struct ParticipantToken(Zeroizing<String>);

impl ParticipantToken {
    /// Moves a provider-decoded token into zeroizing custody without another plaintext clone.
    #[doc(hidden)]
    pub fn from_owned(value: String) -> Self {
        Self(Zeroizing::new(value))
    }

    pub(crate) fn copy_from(value: &str) -> Self {
        Self::from_owned(value.to_owned())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Clone for ParticipantToken {
    fn clone(&self) -> Self {
        Self::copy_from(self.as_str())
    }
}

impl Zeroize for ParticipantToken {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Drop for ParticipantToken {
    fn drop(&mut self) {
        // Keep this explicit instead of relying only on Zeroizing's nested Drop so lifecycle
        // tests can prove that every ParticipantToken owner takes the zeroizing path.
        self.0.zeroize();
        #[cfg(test)]
        {
            debug_assert!(self.0.is_empty());
            ZEROIZING_DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
pub(crate) fn observed_zeroizing_drops() -> usize {
    ZEROIZING_DROP_COUNT.load(Ordering::SeqCst)
}

impl fmt::Debug for ParticipantToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ParticipantToken([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "sendy-livekit-token-sentinel";

    #[test]
    fn debug_and_clone_never_create_plaintext_diagnostics() {
        let token = ParticipantToken::copy_from(SENTINEL);
        let clone = token.clone();

        assert_eq!(format!("{token:?}"), "ParticipantToken([REDACTED])");
        assert_eq!(clone.as_str(), SENTINEL);
    }

    #[test]
    fn explicit_zeroize_clears_each_owned_copy() {
        let mut first = ParticipantToken::copy_from(SENTINEL);
        let mut second = first.clone();

        first.zeroize();
        second.zeroize();

        assert!(first.as_str().is_empty());
        assert!(second.as_str().is_empty());
    }

    #[test]
    fn drop_runs_the_explicit_zeroizing_path() {
        let before = observed_zeroizing_drops();
        {
            let _token = ParticipantToken::copy_from(SENTINEL);
        }

        assert!(observed_zeroizing_drops() > before);
    }
}
