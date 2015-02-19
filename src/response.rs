use sys::ffi;
use std::{mem, slice};
use libc::{c_char, c_int, size_t};
use common::slice_to_mut_pair;
use {Status, Header, Headers, Version, Chunks, ChunkReader};

/// A parsed Response, borrowing a ResponseParser.
#[derive(Debug)]
pub struct Response<'s: 'h, 'h> {
    pub version: Version,
    pub status: Status,
    pub headers: Headers<'s, 'h>,
    pub reason: &'s [u8],
    pub raw: &'s [u8]
}

#[derive(Debug)]
pub struct ResponseParser<'s: 'h, 'h> {
    read: &'s [u8],
    unread: &'s mut [u8],
    headers: &'h mut [Header<'s>],
    reason: &'s [u8],
    status: c_int,
    version: c_int
}

#[derive(Debug, PartialEq, Copy)]
pub enum ResponseParserError {
    ParseError,
    TooLong,
    IncompleteResponse
}

impl<'s, 'h> ResponseParser<'s, 'h> {
    pub fn new(stream: &'s mut [u8], headers: &'h mut [Header<'s>]) -> ResponseParser<'s, 'h> {
        let stream_start = stream.as_ptr();
        let read: &'s [u8] =
            unsafe { mem::transmute(slice::from_raw_parts(stream_start, 0)) };
        ResponseParser {
            read: read,
            unread: stream,
            headers: headers,
            reason: &[],
            status: 0,
            version: 0
        }
    }

    pub fn parse<C: Chunks, F>(mut self, chunks: C, cb: F)
    where F: FnOnce(Result<Response<'s, 'h>, ResponseParserError>, C) {
        if self.unread.len() == 0 {
            return cb(Err(ResponseParserError::TooLong), chunks);
        }

        chunks.chunk(move |reader| {
            let (mayberead, chunks) = reader.read(self.unread);
            let read = match mayberead {
                Some(read) => read,
                None => return cb(Err(ResponseParserError::IncompleteResponse), chunks)
            };
            self.unread = &mut mem::replace(&mut self.unread, &mut [])[read..];
            unsafe { *slice_to_mut_pair(&mut self.read).1 += read; }

            let res = unsafe {
                let reason_pair = slice_to_mut_pair(&mut self.reason);

                ffi::phr_parse_response(
                    self.read.as_ptr() as *const c_char,
                    self.read.len() as size_t,
                    &mut self.version,
                    &mut self.status,
                    reason_pair.0 as *mut *const u8 as *mut *const c_char,
                    reason_pair.1 as *mut usize as *mut size_t,
                    mem::transmute::<*mut Header,
                                     *mut ffi::phr_header>(self.headers.as_mut_ptr()),
                    slice_to_mut_pair(&mut &*self.headers).1 as *mut usize as *mut size_t,
                    (self.read.len() - read) as size_t
                )
            };

            match res {
                // Succesfully parsed, we're done.
                x if x > 0 => {
                    let req = Response {
                        version: Version(1, self.version as u8),
                        status: Status(self.status as u16),
                        headers: Headers(self.headers),
                        reason: self.reason,
                        raw: self.read
                    };

                    cb(Ok(req), chunks)
                },

                // Parse Error
                -1 => {
                    println!("Parse error on {:?}", self.read);
                    cb(Err(ResponseParserError::ParseError), chunks)
                },

                // Incomplete, continue
                -2 => { self.parse(chunks, cb) },

                x => panic!("Unexpected result from phr_parse_request: {:?}", x)
            }
        })
    }
}

