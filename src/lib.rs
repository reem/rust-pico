#![feature(core, libc)]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! # pico
//!
//! Nonblocking parsing support using picohttpparser.
//!

extern crate pico_sys as sys;
extern crate libc;

/// A recursive reader which can read a single chunk into a buffer.
pub trait ChunkReader<C> {
    /// Retrieve the chunk and potentially the rest of the data.
    fn read(self, &mut [u8]) -> (Option<usize>, C);
}

/// A recursive list of chunks of bytes, available one chunk at a time.
pub trait Chunks {
    /// The "proof" that a read is ready and won't block.
    ///
    /// This is actually used to do the reading through the `read` method
    /// on ChunkReader.
    type Reader: ChunkReader<Self>;

    /// Request a single chunk, as a reader.
    fn chunk<F>(self, F) where F: FnOnce(Self::Reader);
}

/// The HTTP headers
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Headers<'s: 'h, 'h>(pub &'h [Header<'s>]);

/// A single HTTP header field and value pair.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Header<'s>(pub &'s [u8], pub &'s [u8]);

/// Static initializer for Headers storage.
pub const HEADER_EMPTY: Header<'static> = Header(&[], &[]);

/// The HTTP method.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Method<'s>(pub &'s [u8]);

/// The HTTP request path.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Path<'s>(pub &'s [u8]);

/// The HTTP status code.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Status(pub u16);

/// The HTTP version.
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Version(pub u8, pub u8);

/// HTTP Request Parsing Utilities.
pub mod request;

/// HTTP Response Parsing Utilities.
pub mod response;

mod common;

#[cfg(test)]
mod tests;

