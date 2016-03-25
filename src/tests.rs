use {Chunks, ChunkReader, HEADER_EMPTY, Version};
use request::RequestParser;
use response::ResponseParser;

use std::slice::bytes;
use std::str;

#[derive(Clone, Copy)]
struct DataReader {
    data: &'static [u8],
    num_reads: usize,
}

impl Chunks for DataReader {
    type Reader = DataReader;

    fn chunk<F>(self, cb: F) where F: FnOnce(DataReader) { cb(self) }
}

impl ChunkReader<DataReader> for DataReader {
    fn read(self, into: &mut [u8]) -> (Option<usize>, DataReader) {
        if self.num_reads == 0 {
            (None, self)
        } else {
            bytes::copy_memory(self.data, into);
            (Some(self.data.len()), DataReader{data: self.data, num_reads: self.num_reads - 1})
        }
    }
}

#[inline(always)]
fn s(x: &[u8]) -> &str { str::from_utf8(x).unwrap() }

const REQUEST: &'static [u8] =
    b"GET /hoge HTTP/1.1\r\nHost: example.com\r\nCookie: \r\n\r\n";

const RESPONSE: &'static [u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 14\r\n\r\nHello World\r\n\r\n";

const PARTIAL_RESPONSE: &'static [u8] =
    b"HTTP/1.1 200";

#[test]
fn test_request_parse() {
    let mut stream = [0u8; 4096];
    let mut headers = [HEADER_EMPTY; 8];

    let parser = RequestParser::new(&mut stream, &mut headers);
    parser.parse(
        DataReader { data: REQUEST, num_reads: 1},
        |request, _, _| {
            let r = request.unwrap();
            assert_eq!(r.version, Version(1, 1));
            assert_eq!(s(r.path.0), "/hoge");
            assert_eq!(s(r.method.0), "GET");
            assert_eq!(s(r.headers.0[0].0), "Host");
            assert_eq!(s(r.headers.0[0].1), "example.com");
            assert_eq!(s(r.raw), "GET /hoge HTTP/1.1\r\nHost: example.com\r\nCookie: \r\n\r\n");
        }
    );
}

#[test]
fn test_response_parse() {
    let mut stream = [0u8; 4096];
    let mut headers = [HEADER_EMPTY; 8];

    let parser = ResponseParser::new(&mut stream, &mut headers);
    parser.parse(
        DataReader { data: RESPONSE, num_reads: 1 },
        |response, _, remaining| {
            let r = response.unwrap();
            assert_eq!(r.version, Version(1, 1));
            assert_eq!(r.status.0, 200);
            assert_eq!(s(r.reason), "OK");
            assert_eq!(s(r.headers.0[0].0), "Content-Length");
            assert_eq!(s(r.headers.0[0].1), "14");
            assert_eq!(s(r.raw), "HTTP/1.1 200 OK\r\nContent-Length: 14\r\n\r\n");
            assert_eq!(s(remaining), "Hello World\r\n\r\n");
        }
    );
}

#[test]
fn test_incomplete_parse() {
    let mut stream = [0u8; 4096];
    let mut headers = [HEADER_EMPTY; 8];

    let parser = ResponseParser::new(&mut stream, &mut headers);
    parser.parse(
        DataReader { data: PARTIAL_RESPONSE, num_reads: 1},
        |response, _, remaining| {
            assert!(response.is_err());
            assert_eq!(remaining, PARTIAL_RESPONSE);
        }
    );
}
