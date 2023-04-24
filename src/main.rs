use git_starter_rust::git;
use std::env;
use std::fs;
//
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
        let sha_commit = git::do_commit(
            args[2].to_string(),
            args[4].to_string(),
            args[6].to_string(),
        );

        println!("{}", sha_commit);
    }
}


pub fn do_commit(tree_sha: String, commit_sha: String, message: String) -> String {
    let hardcode_data = "author ScotChacon <schacon@gmail.com> 1243040974 -0700\ncommitter ScotChacon <schacon@gmail.com> 1243040974 -0700".to_string();

    let content_commit = format!(
        "tree {}\nparent {}\n{}\n\n",
        tree_sha, commit_sha, hardcode_data
    );

    let content_commit = content_commit + &message + "\n";

    let sha_commit = write_obj(content_commit.into_bytes(), "commit");
    sha_commit
}