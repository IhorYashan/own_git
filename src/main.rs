use git_starter_rust::git;
use std::env;
use std::fs;

//mod git_tree;

fn main() {
    let args: Vec<String> = env::args().collect();

    git::do_git_init(&args);

    if args[1] == "cat-file" && args[2] == "-p" {
        let blob_file = &args[3];

        git::read_blob(blob_file.to_string());
    }

    if args[1] == "hash-object" && args[2] == "-w" {
        let content_file = fs::read(&args[3].to_string()).unwrap();

        print!("{}", git::write_obj(content_file, "blob"));
    }

    if args[1] == "ls-tree" && args[2] == "--name-only" {
        let sha_tree = &args[3];

        git::read_tree_sha(sha_tree.to_string());
    }
    if args[1] == "write-tree" {
        print!("{}", git::write_tree(&".".to_string()));
    }

    if args[1] == "commit-tree" && args[3] == "-p" && args[5] == "-m" {
        println!("args[2] {}", args[2]);
        println!("args[4] {}", args[4]);
        println!("args[6] {}", args[6]);
    }
}
