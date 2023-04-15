extern crate hex;
extern crate sha1;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;

use crate::sha1::Digest;

use sha1::Sha1;

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

fn read_blob(path_to_bolob_file: String, hash_file: String) {
    let mut file_content = Vec::new();

    let path_to_bolob_file = path_to_bolob_file + "/" + &hash_file.to_string();
    let mut path_to_bolob_file = File::open(&path_to_bolob_file).unwrap();

    path_to_bolob_file.read_to_end(&mut file_content).unwrap();

    let compressed_data = &file_content[..];

    let mut decoder = ZlibDecoder::new(compressed_data);

    let mut buffer = [0; 4096];

    loop {
        let bytes_read = match decoder.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => panic!("Unable to read from decoder: {:?}", e),
        };

        std::io::stdout().write_all(&buffer[..bytes_read]).unwrap();
    }
}

fn write_blob(content_blob_file: Vec<u8>) {
    //let data_to_compress = content_blob_file.as_bytes();

    let header_blob = format!("blob {}\x00", content_blob_file.len());

    let data_to_compress =
        header_blob + &format!("{}", String::from_utf8(content_blob_file.into()).unwrap());

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data_to_compress.as_bytes()).unwrap();
    let compressed_data = encoder.finish().unwrap();

    let mut hasher = Sha1::new();
    hasher.update(data_to_compress);
    let hash = hasher.finalize();
    let hash_blob_file = hex::encode(&hash);

    print!("{}", hash_blob_file);

    let hash_dir = &hash_blob_file[..2];
    let hash_file = &hash_blob_file[2..];
    let hash_path_dir = format!(".git/objects/{}/", hash_dir);
    let full_hash_path = format!("{}{}", hash_path_dir, hash_file);

    fs::create_dir(hash_path_dir).unwrap();
    fs::write(full_hash_path, compressed_data).unwrap();
}

fn parse_args(args: &String) -> (&str, &str) {
    let (hash_path, hash_file) = (&args[..2], &args[2..]);
    (hash_path, hash_file)
}

fn read_tree_sha(sha_tree: String) {
    let mut file_content = Vec::new();

    let hash_dir = &sha_tree[..2];
    //println!("hash_dir : {}", hash_dir);
    let hash_tree_object = &sha_tree[2..];
    println!("hash_dir : {}", hash_tree_object);

    let full_path = ".git/objects/".to_string() + &hash_dir + &hash_tree_object;
    let mut full_path = File::open(&full_path).unwrap();
    full_path.read_to_end(&mut file_content).unwrap();

    let compressed_data = &file_content[..];

    let mut buffer = [0; 4096];

    loop {
        let bytes_read = match decoder.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => panic!("Unable to read from decoder: {:?}", e),
        };

        std::io::stdout().write_all(&buffer[..bytes_read]).unwrap();
    }

    //println!("decoded_data : {}", decoded_data);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    do_git_init(&args);

    let path_to_objects = ".git/objects/".to_string();

    if args[1] == "cat-file" && args[2] == "-p" {
        let blob_file = &args[3]; //own_git cat-file -p <blob_file> // hash : [hash_dir + hash_file]
        let (hash_path, hash_file) = parse_args(blob_file);
        let path_to_bolob_file = path_to_objects.clone() + hash_path;

        read_blob(path_to_bolob_file, hash_file.to_string());
    }

    if args[1] == "hash-object" && args[2] == "-w" {
        let content_file = fs::read(&args[3].to_string()).unwrap(); //own_git hash-object -w <file>

        write_blob(content_file);
    }

    if args[1] == "ls-tree" && args[2] == "--name-only" {
        let sha_tree = &args[3];

        //let sha_tree = "acada1c1122b334b98a15430aa2fae91d024c7ca";

        read_tree_sha(sha_tree.to_string());
    }
}
