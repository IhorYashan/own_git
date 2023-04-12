use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::stdout;
use std::io::Read;

fn do_git_init(args: &Vec<String>) {
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } else {
        //   println!("unknown command: {}", args[1])
    }
}

fn read_blob(path_to_objects: String, hash_file: String) {
    let mut file_content = Vec::new();

    let path_to_objects = path_to_objects + "/";
    let path_to_objects = path_to_objects + &hash_file.to_string();

    let mut path_to_objects = File::open(&path_to_objects).expect("Unable to open file");
    path_to_objects
        .read_to_end(&mut file_content)
        .expect("Unable to read");

    let compressed_data = &file_content[..];

    //-------------------------decocde---------------------------------------//

    let mut decoder = ZlibDecoder::new(compressed_data);

    let mut buffer = [0; 4096];

    loop {
        let bytes_read = match decoder.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => panic!("Unable to read from decoder: {:?}", e),
        };

        std::io::stdout().write_all(&buffer[8..bytes_read]).unwrap();
    }
}

fn write_blob(content_blob_file: String) {
    let data_to_compress = content_blob_file.as_bytes();
    let mut compressed_data = Vec::new();

    let mut encoder = ZlibEncoder::new(&mut compressed_data, Compression::default());
    encoder.write_all(data_to_compress).unwrap();
    encoder.finish().unwrap();
    stdout().write_all(&compressed_data).unwrap();
}

fn parse_args(args: &String) -> (&str, &str) {
    let (hash_path, hash_file) = (&args[..2], &args[2..]);
    (hash_path, hash_file)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    do_git_init(&args);

    let path_to_objects = ".git/objects/".to_string();

    if args[1] == "cat-file" && args[2] == "-p" {
        let blob_file = &args[3]; //own_git cat-file -p <blob_file>
        let (hash_path, hash_file) = parse_args(blob_file);
        let path = path_to_objects + hash_path;

        read_blob(path, hash_file.to_string());
    }

    if args[1] == "hash-object" && args[2] == "-w" {
        let content_blob_file = &args[3]; //own_git hash-object -w <file>

        //let path = path_to_objects + content_hash_path;
        write_blob(content_blob_file.to_string());
    }
}
