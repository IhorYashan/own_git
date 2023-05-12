extern crate hex;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::Read;
use std::io::Write;

pub fn decode_data(compressed_data: &[u8]) -> (String, usize) {
    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut buffer = [0; 4096];
    let mut string_buffer = String::new();
    let mut bytes = 0;
    loop {
        let bytes_read = match decoder.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => panic!("Unable to read from decoder: {:?}", e),
        };
        bytes = bytes_read;
    }

    #[allow(unsafe_code)]
    let string_buffer = unsafe { String::from_utf8_unchecked((&buffer[..bytes]).to_vec()) };
    //string_buffer.push_str(&String::from_utf8_lossy(&buffer[..bytes]));

    (string_buffer, bytes)
}

pub fn encode_data(data_to_compress: String) -> (String, Vec<u8>) {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data_to_compress.as_bytes()).unwrap();
    let compressed_data = encoder.finish().unwrap();

    let mut hasher = Sha1::new();
    hasher.update(data_to_compress);
    let hash = hasher.finalize();
    let hash_blob_file = hex::encode(&hash);
    (hash_blob_file, compressed_data)
}

//
