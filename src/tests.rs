use {Chunks, ChunkReader, HEADER_EMPTY};
use request::{RequestParser};

use std::slice::bytes;

#[derive(Copy)]
struct RequestReader {
    req: &'static [u8]
}

impl Chunks for RequestReader {
    type Reader = RequestReader;

    fn chunk<F>(self, cb: F) where F: FnOnce(RequestReader) { cb(self) }
}

impl ChunkReader<RequestReader> for RequestReader {
    fn read(self, into: &mut [u8]) -> (Option<usize>, RequestReader) {
        println!("Writing: {:?}", self.req);
        bytes::copy_memory(into, self.req);
        (Some(self.req.len()), self)
    }
}

const REQUEST: &'static [u8] = b"GET /hoge HTTP/1.1\r\nHost: example.com\r\nCookie: \r\n\r\n";

#[test]
fn test_request_parse() {
    let mut stream = [0u8; 4096];
    let mut headers = [HEADER_EMPTY; 8];

    println!("{:?}", REQUEST);

    let parser = RequestParser::new(&mut stream, &mut headers);
    parser.parse(
        RequestReader { req: REQUEST },
        |response, _| println!("{:?}", response)
    );
}

