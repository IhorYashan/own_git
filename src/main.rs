extern crate hex;
extern crate sha1;
use crate::sha1::Digest;
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
use std::path::PathBuf;
use std::str::FromStr;

use sha1::Sha1;

fn decode_data(compressed_data: &[u8]) -> (String, usize) {
    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut buffer = [0; 4096];
    let mut s_buffer = String::new();
    let mut bytes = 0;
    loop {
        let bytes_read = match decoder.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => panic!("Unable to read from decoder: {:?}", e),
        };
        bytes = bytes_read;
    }
    s_buffer.push_str(&String::from_utf8_lossy(&buffer[..bytes]));

    (s_buffer, bytes)
}

fn do_git_init(args: &Vec<String>) {
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } else {
        // println!("unknown command: {}", args[1])
    }
}

fn read_blob(path_to_bolob_file: String, hash_file: String) {
    let mut file_content = Vec::new();

    let path_to_bolob_file = path_to_bolob_file + "/" + &hash_file.to_string();
    let mut path_to_bolob_file = File::open(&path_to_bolob_file).unwrap();

    path_to_bolob_file.read_to_end(&mut file_content).unwrap();

    let compressed_data = &file_content[..];
    let (buffer, bytes) = decode_data(compressed_data);
    print!("{}", &buffer[8..]);
}

fn write_obj(path: &str, file_type: &str) -> String {
    println!("--- path: {path} --- ");

    let content_file = fs::read(path).unwrap();

    let header_blob = format!("{} {}\x00", file_type, content_file.len());
    #[allow(unsafe_code)]
    let content_file = unsafe { String::from_utf8_unchecked(content_file) };

    let data_to_compress = header_blob + &format!("{}", content_file);

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

    hash_blob_file
}

fn parse_args(args: &String) -> (&str, &str) {
    let (hash_path, hash_file) = (&args[..2], &args[2..]);
    (hash_path, hash_file)
}
//write tree
fn write_tree(file_path: &str) -> String {
    let mut sha_out: String = String::new();
    let mut entries = fs::read_dir(file_path)
        .expect("Failed to read directory")
        .map(|res| res.expect("Failed to read entry").path())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for dir in entries {
        let mode;
        let path_name = dir
            .as_path()
            .to_str()
            .expect("Failed to convert path to string");

        if path_name == "./.git" {
            continue;
        }

        let sha_file;
        if dir.is_dir() {
            mode = "40000";
            let sha_file1 = write_tree(path_name);
            sha_file = hex::decode(&sha_file1).expect("Failed to decode hex");
        } else {
            mode = "100644";
            let sha_file1 = write_obj(path_name, "blob");
            sha_file = hex::decode(&sha_file1).expect("Failed to decode hex");
        }
        #[allow(unsafe_code)]
        let sha = unsafe { String::from_utf8_unchecked(sha_file) };
        sha_out += &format!(
            "{mode} {}\x00{}",
            dir.file_name()
                .expect("Failed to get file name")
                .to_str()
                .expect("Failed to convert file name to string"),
            sha
        );
    }

    let res_sha = write_obj(&sha_out, "tree");
    res_sha
}
//read tree

fn read_tree_sha(sha_tree: String) {
    let mut file_content = Vec::new();

    let hash_dir = &sha_tree[..2];

    let hash_tree_object = &sha_tree[2..];

    let full_path = ".git/objects/".to_string() + &hash_dir + "/" + &hash_tree_object;
    let mut full_path = File::open(&full_path).unwrap();
    full_path.read_to_end(&mut file_content).unwrap();

    let mut formatted_buff = String::new();
    let compressed_data = &file_content[..];
    let (formatted_buff, bytes) = decode_data(compressed_data);

    let formatted_buff = formatted_buff.replace("\\x00", "\x00");
    let formatted_buff = formatted_buff.replace("\\\\", "\\");

    let parts: Vec<&str> = formatted_buff.split('\x00').skip(1).collect();

    for part in parts {
        if part.contains(' ') {
            if let Some(word) = part.split(' ').nth(1) {
                print!("{}\n", word);
            }
        }
    }
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
        //let content_file = fs::read(&args[3].to_string()).unwrap(); //own_git hash-object -w <file>

        write_obj(&args[3], "blob");
    }

    if args[1] == "ls-tree" && args[2] == "--name-only" {
        let sha_tree = &args[3];

        read_tree_sha(sha_tree.to_string());
    }
    if args[1] == "write-tree" {
        write_tree(&".".to_string());
    }
}
