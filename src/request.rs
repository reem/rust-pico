use sys::ffi;
use std::{mem, slice};
use libc::{c_char, c_int, size_t};

use {Method, Header, Headers, Version, Chunks, ChunkReader};

/// A parsed Request, borrowing a RequestParser.
#[derive(Debug)]
pub struct Request<'s: 'h, 'h> {
    pub version: Version,
    pub method: Method<'s>,
    pub path: &'s [u8],
    pub headers: Headers<'s, 'h>,
    pub raw: &'s [u8]
}

#[derive(Debug)]
pub struct RequestParser<'s: 'h, 'h> {
    read: &'s [u8],
    unread: &'s mut [u8],
    headers: &'h mut [Header<'s>],
    method: Method<'s>,
    path: &'s [u8],
    version: c_int
}

#[derive(Debug)]
pub enum RequestParserError {
    ParseError,
    TooLong,
    IncompleteRequest
}

impl<'s, 'h> RequestParser<'s, 'h> {
    pub fn new(stream: &'s mut [u8], headers: &'h mut [Header<'s>]) -> RequestParser<'s, 'h> {
        let stream_start = stream.as_ptr();
        let read: &'s [u8] =
            unsafe { mem::transmute(slice::from_raw_parts(&stream_start, 0)) };
        let this = RequestParser {
            read: read,
            unread: stream,
            headers: headers,
            method: Method(&[]),
            path: &[],
            version: 0
        };

        println!("Same {:?}", this.read.as_ptr() == this.unread.as_ptr());
        this
    }

    pub fn parse<C: Chunks, F>(mut self, chunks: C, cb: F)
    where F: FnOnce(Result<Request<'s, 'h>, RequestParserError>, C) {
        if self.unread.len() == 0 {
            return cb(Err(RequestParserError::TooLong), chunks);
        }

        chunks.chunk(move |reader| {
            let (mayberead, chunks) = reader.read(self.unread);
            let read = match mayberead {
                Some(read) => read,
                None => return cb(Err(RequestParserError::IncompleteRequest), chunks)
            };
            println!("From unread: {:?}", &self.unread[..read]);
            self.unread = &mut mem::replace(&mut self.unread, &mut [])[read..];
            unsafe { *mutlen(&mut self.read) += read; }

            println!("Read {:?}", self.read);

            let res = unsafe { ffi::phr_parse_request(
                self.read.as_ptr() as *const c_char,
                self.read.len() as size_t,
                &mut (self.method.0.as_ptr() as *const c_char),
                mutlen(&mut self.method.0) as *mut size_t,
                &mut (self.path.as_ptr() as *const c_char),
                mutlen(&mut self.path) as *mut size_t,
                &mut self.version,
                mem::transmute(&mut self.headers.as_ptr()),
                mutlen(&mut &*self.headers) as *mut size_t,
                (self.read.len() - read) as size_t
            ) };

            match res {
                // Succesfully parsed, we're done.
                x if x > 0 => {
                    let req = Request {
                        version: Version(1, self.version as u8),
                        method: self.method,
                        path: self.path,
                        headers: Headers(self.headers),
                        raw: self.read
                    };

                    cb(Ok(req), chunks)
                },

                // Parse Error
                -1 => {
                    println!("Parse error on {:?}", self.read);
                    cb(Err(RequestParserError::ParseError), chunks)
                },

                // Incomplete, continue
                -2 => { self.parse(chunks, cb) },

                x => panic!("Unexpected result from phr_parse_request: {:?}", x)
            }
        })
    }
}

unsafe fn mutlen<T>(slice: &mut &[T]) -> *mut usize {
    use std::raw;

    let rawref = mem::transmute::<_, &mut raw::Slice<T>>(slice);

    &mut rawref.len
}

