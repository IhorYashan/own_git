use git_starter_rust::git;
use std::env;
use std::fs;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();

    git::do_git_init(&args);

    let path_to_objects = ".git/objects/".to_string();

    if args[1] == "cat-file" && args[2] == "-p" {
        let blob_file = &args[3]; //own_git cat-file -p <blob_file> // hash : [hash_dir + hash_file]
        let (hash_path, hash_file) = git::parse_args(blob_file);
        let path_to_bolob_file = path_to_objects.clone() + hash_path;

        git::read_blob(path_to_bolob_file, hash_file.to_string());
    }

    if args[1] == "hash-object" && args[2] == "-w" {
        let content_file = fs::read(&args[3].to_string()).unwrap(); //own_git hash-object -w <file>

        print!("{}", git::write_obj(content_file, "blob"));
    }

    if args[1] == "ls-tree" && args[2] == "--name-only" {
        let sha_tree = &args[3];

        git::read_tree_sha(sha_tree.to_string());
    }
    if args[1] == "write-tree" {
        print!("{}", git::write_tree(&".".to_string()));
    }
}
