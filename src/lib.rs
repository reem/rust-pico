#![feature(core, libc)]
// #![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

//! # pico
//!
//! Nonblocking parsing support for hyper using picohttpparser.
//!

extern crate "pico-sys" as sys;
extern crate hyper;
extern crate libc;

trait ChunkReader<C> {
    fn read(self, &mut [u8]) -> (Option<usize>, C);
}

pub trait Chunks {
    type Reader: ChunkReader<Self>;

    fn chunk<F>(self, F) where F: FnOnce(Self::Reader);
}

#[derive(Copy, Debug)]
#[repr(C)]
pub struct Headers<'s: 'h, 'h>(pub &'h [Header<'s>]);

#[derive(Copy, Debug)]
#[repr(C)]
pub struct Header<'s>(pub &'s [u8], pub &'s [u8]);

/// Static initializer for Headers storage.
pub const HEADER_EMPTY: Header<'static> = Header(&[], &[]);

/// The HTTP method.
#[derive(Copy, Debug)]
#[repr(C)]
pub struct Method<'s>(pub &'s [u8]);

/// The HTTP request path.
#[derive(Copy, Debug)]
#[repr(C)]
pub struct Path<'s>(pub &'s [u8]);

/// The HTTP status code.
#[derive(Copy, Debug)]
#[repr(C)]
pub struct Status(pub u16);

/// The HTTP version.
#[derive(Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Version(pub u8, pub u8);

pub mod request;
pub mod response;
mod common;

#[cfg(test)]
mod tests;

