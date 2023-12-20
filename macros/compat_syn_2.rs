//! Definitions for compatibility with syn 2
//!
//! This, along with `compat_syn_1.rs` and
//! [`compat_syn_common`](super::compat_syn_common)
//! exists to minimise the delta in the commit which switches to syn 2.
//!
//! This approach would also allow us to support *both* syn 1 and syn 2,
//! and correspondingly reduce our MSRV back to 1.54,
//! eg via cargo features,
//! if that turns out to be desirable.

use super::prelude::*;
