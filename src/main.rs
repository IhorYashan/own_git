use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;

fn parse_args() {}

fn do_git_init(args: &Vec<String>) {
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn find_blob_file_path(dir_path: &str, file_name: &str) -> Option<String> {
    let mut file_path: Option<String> = None;

    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(dir_entry) => {
                        let path = dir_entry.path();
                        if path.is_file() && path.file_name().unwrap() == file_name {
                            file_path = Some(path.to_string_lossy().to_string());
                            break;
                        }
                    }
                    Err(e) => println!("Err: {}", e),
                }
            }
        }
        Err(e) => println!("Err: {}", e),
    }

    file_path = file_path.map(|s| s.replace("\\", "/"));

    file_path
}

fn read_blob(path_to_objects: String) {
    let mut file_content = Vec::new();

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

        std::io::stdout().write_all(&buffer[..bytes_read]).unwrap();
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();

    do_git_init(&args);

    let blob_file = &args[2]; //cat-file -p <blob_file>

    //------------------------------get all paths---------------------------//
    let paths = fs::read_dir(".git/objects/")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    let mut blob_file_path = String::from("");

    //------------------------------find blob file path --------------------- //
    for path in paths {
        let path_blob = find_blob_file_path(path.as_str(), blob_file);

        match path_blob {
            Some(file) => {
                blob_file_path = file;
                // println!("{}", blob_file_path);
            }
            None => {
                //  println!("Could not find file");
            }
        }
    }

    if args[1] == "cat-file" {
        read_blob(blob_file_path);
    }
}
