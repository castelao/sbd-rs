//! Parse and write Iridium Short Burst Data (SBD) messages.
//!
//! Iridium is both a [satellite constellation](https://en.wikipedia.org/wiki/Iridium_satellite_constellation)
//! and a [company](https://en.wikipedia.org/wiki/Iridium_Communications) that provides satellite
//! communications. The Iridium network is used by phones, modems, and other communication devices.
//!
//! One mode of transmitting data over the Iridium network is via Short Burst Data (SBD) messages.
//! These messages carry a payload of some small number of bytes, usually less than one thousand.
//! Messages can be Mobile-Originated (MO), meaning that they are sent *from* an Iridium modem, or
//! Mobile-Terminated (MT), meaning that the are sent *to* an Iridium modem. Mobile-originated
//! messages are delivered either to an email address via MIME attachment, or directly to a given
//! IP address and port via TCP; this second method is called DirectIP.
//!
//! This is a simple library for reading mobile originated SBD messages from a stream, decoding
//! their headers and data payloads, and writing them back to a stream. This library does not
//! handle mobile terminated messages.
//!
//! Most of the functionality of this library is exposed by a single executable, named `sbd` that
//! is part of this package. Use the `sbd` executable to inspect raw sbd files stores on a
//! filesystem, discover sbd files on a filesystem, and start a forever-running server to receive
//! Iridium SBD DirectIP messages.

#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_extern_crates,
        unused_import_braces, unused_qualifications)]

pub mod directip;
mod information_element;
pub mod logger;
pub mod filesystem;
pub mod message;

pub use message::Message;

extern crate byteorder;
extern crate chrono;
extern crate glob;
#[macro_use] extern crate log;

use std::error::Error;
use std::fmt;
use std::result;

/// Crate-specific errors
#[derive(Debug)]
pub enum SbdError {
    /// An error while reading bytes from a stream with the byteorder crate.
    Byteorder(byteorder::Error),
    /// A wrapper around a std::io::Error.
    Io(std::io::Error),
    /// Invalid IMEI number.
    // TODO Include the IMEI number as an argument.
    InvalidImei,
    /// Invalid protocol revision number.
    InvalidProtocolRevisionNumber(u8),
    /// Wrapper around a glob error.
    Glob(glob::GlobError),
    /// Missing mobile originated header.
    MissingMobileOriginatedHeader,
    /// Missing mobile originated payload.
    MissingMobileOriginatedPayload,
    /// An oversized message.
    ///
    /// Oversized doesn't demand a size since we don't want to find out how much there really is.
    Oversized,
    /// Wrapper around a glob::PatternError.
    Pattern(glob::PatternError),
    /// An undersized message.
    Undersized(usize),
}

impl fmt::Display for SbdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SbdError::Byteorder(ref err) => write!(f, "Byteorder error: {}", err),
            SbdError::Io(ref err) => write!(f, "IO error: {}", err),
            SbdError::InvalidImei => write!(f, "Invalid IMEI number"),
            SbdError::InvalidProtocolRevisionNumber(number) => write!(f, "Invalid protocl revision number: {}", number),
            SbdError::Glob(ref err) => write!(f, "Glob error: {}", err),
            SbdError::MissingMobileOriginatedHeader => write!(f, "Missing mobile origianted header"),
            SbdError::MissingMobileOriginatedPayload => write!(f, "Missing mobile orignated payload"),
            SbdError::Oversized => write!(f, "Oversized message"),
            SbdError::Pattern(ref err) => write!(f, "Glob pattern error: {}", err),
            SbdError::Undersized(size) => write!(f, "Undersized message: {}", size),
        }
    }
}

impl Error for SbdError {
    fn description(&self) -> &str {
        match *self {
            SbdError::Byteorder(ref err) => err.description(),
            SbdError::Io(ref err) => err.description(),
            SbdError::InvalidImei => "invalid IMEI number",
            SbdError::InvalidProtocolRevisionNumber(_) => "invalid protocol revision number",
            SbdError::Glob(_) => "glob error",
            SbdError::MissingMobileOriginatedHeader => "missing mobile originated header",
            SbdError::MissingMobileOriginatedPayload => "missing mobile originated payload",
            SbdError::Oversized => "oversized message",
            SbdError::Pattern(_) => "glob pattern error",
            SbdError::Undersized(_) => "undersized message",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SbdError::Byteorder(ref err) => Some(err),
            SbdError::Io(ref err) => Some(err),
            // TODO Once glob supports the `Error` trait, include it as a cause.
            _ => None,
        }
    }
}

impl From<byteorder::Error> for SbdError {
    fn from(err: byteorder::Error) -> SbdError {
        SbdError::Byteorder(err)
    }
}

impl From<glob::PatternError> for SbdError {
    fn from(err: glob::PatternError) -> SbdError {
        SbdError::Pattern(err)
    }
}

impl From<glob::GlobError> for SbdError {
    fn from(err: glob::GlobError) -> SbdError {
        SbdError::Glob(err)
    }
}

impl From<std::io::Error> for SbdError {
    fn from(err: std::io::Error) -> SbdError {
        SbdError::Io(err)
    }
}

/// Create-specific `Result`.
pub type Result<T> = result::Result<T, SbdError>;
