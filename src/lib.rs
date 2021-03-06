// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Interface to the random number generator of the operating system.
//!
//! # Platform sources
//!
//! | OS               | interface
//! |------------------|---------------------------------------------------------
//! | Linux, Android   | [`getrandom`][1] system call if available, otherwise [`/dev/urandom`][2] after reading from `/dev/random` once
//! | Windows          | [`RtlGenRandom`][3]
//! | macOS, iOS       | [`SecRandomCopyBytes`][4]
//! | FreeBSD          | [`kern.arandom`][5]
//! | OpenBSD, Bitrig  | [`getentropy`][6]
//! | NetBSD           | [`/dev/urandom`][7] after reading from `/dev/random` once
//! | Dragonfly BSD    | [`/dev/random`][8]
//! | Solaris, illumos | [`getrandom`][9] system call if available, otherwise [`/dev/random`][10]
//! | Fuchsia OS       | [`cprng_draw`][11]
//! | Redox            | [`rand:`][12]
//! | CloudABI         | [`random_get`][13]
//! | Haiku            | `/dev/random` (identical to `/dev/urandom`)
//! | SGX              | RDRAND
//! | Web browsers     | [`Crypto.getRandomValues`][14] (see [Support for WebAssembly and ams.js][14])
//! | Node.js          | [`crypto.randomBytes`][15] (see [Support for WebAssembly and ams.js][16])
//!
//! Getrandom doesn't have a blanket implementation for all Unix-like operating
//! systems that reads from `/dev/urandom`. This ensures all supported operating
//! systems are using the recommended interface and respect maximum buffer
//! sizes.
//!
//! ## Support for WebAssembly and ams.js
//!
//! The three Emscripten targets `asmjs-unknown-emscripten`,
//! `wasm32-unknown-emscripten` and `wasm32-experimental-emscripten` use
//! Emscripten's emulation of `/dev/random` on web browsers and Node.js.
//!
//! The bare WASM target `wasm32-unknown-unknown` tries to call the javascript
//! methods directly, using either `stdweb` or `wasm-bindgen` depending on what
//! features are activated for this crate. Note that if both features are
//! enabled `wasm-bindgen` will be used. If neither feature is enabled,
//! `getrandom` will always fail.
//!
//! ## Early boot
//!
//! It is possible that early in the boot process the OS hasn't had enough time
//! yet to collect entropy to securely seed its RNG, especially on virtual
//! machines.
//!
//! Some operating systems always block the thread until the RNG is securely
//! seeded. This can take anywhere from a few seconds to more than a minute.
//! Others make a best effort to use a seed from before the shutdown and don't
//! document much.
//!
//! A few, Linux, NetBSD and Solaris, offer a choice between blocking and
//! getting an error; in these cases we always choose to block.
//!
//! On Linux (when the `genrandom` system call is not available) and on NetBSD
//! reading from `/dev/urandom` never blocks, even when the OS hasn't collected
//! enough entropy yet. To avoid returning low-entropy bytes, we first read from
//! `/dev/random` and only switch to `/dev/urandom` once this has succeeded.
//!
//! # Error handling
//!
//! We always choose failure over returning insecure "random" bytes. In general,
//! on supported platforms, failure is unlikely, though not impossible. If an
//! error does occur, then it is likely that it will occur on every call to
//! `getrandom`, hence after the first successful call one can be reasonably
//! confident that no errors will occur.
//! 
//! On unsupported platforms, `getrandom` always fails.
//!
//! [1]: http://man7.org/linux/man-pages/man2/getrandom.2.html
//! [2]: http://man7.org/linux/man-pages/man4/urandom.4.html
//! [3]: https://msdn.microsoft.com/en-us/library/windows/desktop/aa387694.aspx
//! [4]: https://developer.apple.com/documentation/security/1399291-secrandomcopybytes?language=objc
//! [5]: https://www.freebsd.org/cgi/man.cgi?query=random&sektion=4
//! [6]: https://man.openbsd.org/getentropy.2
//! [7]: http://netbsd.gw.com/cgi-bin/man-cgi?random+4+NetBSD-current
//! [8]: https://leaf.dragonflybsd.org/cgi/web-man?command=random&section=4
//! [9]: https://docs.oracle.com/cd/E88353_01/html/E37841/getrandom-2.html
//! [10]: https://docs.oracle.com/cd/E86824_01/html/E54777/random-7d.html
//! [11]: https://fuchsia.googlesource.com/zircon/+/HEAD/docs/syscalls/cprng_draw.md
//! [12]: https://github.com/redox-os/randd/blob/master/src/main.rs
//! [13]: https://github.com/NuxiNL/cloudabi/blob/v0.20/cloudabi.txt#L1826
//! [14]: https://www.w3.org/TR/WebCryptoAPI/#Crypto-method-getRandomValues
//! [15]: https://nodejs.org/api/crypto.html#crypto_crypto_randombytes_size_callback
//! [16]: #support-for-webassembly-and-amsjs

#![no_std]

#[cfg(not(target_env = "sgx"))]
#[macro_use] extern crate std;

#[cfg(any(
    target_os = "android",
    target_os = "netbsd",
    target_os = "solaris",
    target_os = "redox",
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "emscripten",
    target_os = "linux",
))]
mod utils;
mod error;
pub use error::{Error, UNKNOWN_ERROR, UNAVAILABLE_ERROR};


// System-specific implementations.
// 
// These should all provide getrandom_inner with the same signature as getrandom.

macro_rules! mod_use {
    ($cond:meta, $module:ident) => {
        #[$cond]
        mod $module;
        #[$cond]
        use $module::getrandom_inner;
    }
}

mod_use!(cfg(target_os = "android"), linux_android);
mod_use!(cfg(target_os = "bitrig"), openbsd_bitrig);
mod_use!(cfg(target_os = "cloudabi"), cloudabi);
mod_use!(cfg(target_os = "dragonfly"), dragonfly_haiku);
mod_use!(cfg(target_os = "emscripten"), emscripten);
mod_use!(cfg(target_os = "freebsd"), freebsd);
mod_use!(cfg(target_os = "fuchsia"), fuchsia);
mod_use!(cfg(target_os = "haiku"), dragonfly_haiku);
mod_use!(cfg(target_os = "ios"), macos);
mod_use!(cfg(target_os = "linux"), linux_android);
mod_use!(cfg(target_os = "macos"), macos);
mod_use!(cfg(target_os = "netbsd"), netbsd);
mod_use!(cfg(target_os = "openbsd"), openbsd_bitrig);
mod_use!(cfg(target_os = "redox"), redox);
mod_use!(cfg(target_os = "solaris"), solaris);
mod_use!(cfg(windows), windows);
mod_use!(cfg(target_env = "sgx"), sgx);

mod_use!(
    cfg(all(
        target_arch = "wasm32",
        not(target_os = "emscripten"),
        feature = "wasm-bindgen"
    )),
    wasm32_bindgen
);

mod_use!(
    cfg(all(
        target_arch = "wasm32",
        not(target_os = "emscripten"),
        not(feature = "wasm-bindgen"),
        feature = "stdweb",
    )),
    wasm32_stdweb
);

mod_use!(
    cfg(not(any(
        target_os = "android",
        target_os = "bitrig",
        target_os = "cloudabi",
        target_os = "dragonfly",
        target_os = "emscripten",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "redox",
        target_os = "solaris",
        target_env = "sgx",
        windows,
        all(
            target_arch = "wasm32",
            any(
                target_os = "emscripten",
                feature = "wasm-bindgen",
                feature = "stdweb",
            ),
        ),
    ))),
    dummy
);


/// Fill `dest` with random bytes from the system's preferred random number
/// source.
/// 
/// This function returns an error on any failure, including partial reads. We
/// make no guarantees regarding the contents of `dest` on error.
/// 
/// Blocking is possible, at least during early boot; see module documentation.
/// 
/// In general, `getrandom` will be fast enough for interactive usage, though
/// significantly slower than a user-space CSPRNG; for the latter consider
/// [`rand::thread_rng`](https://docs.rs/rand/*/rand/fn.thread_rng.html).
pub fn getrandom(dest: &mut [u8]) -> Result<(), Error> {
    getrandom_inner(dest)
}
