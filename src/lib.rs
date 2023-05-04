pub mod git {
    use reqwest::header::CONTENT_TYPE;
    use std::path::PathBuf;
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;
    mod zlib;
    extern crate hex;
    //use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::env;
    use std::io;

    pub fn do_git_init() {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();       
    }

    pub fn read_blob(blob_file: String) {
        let mut file_content = Vec::new();
        let path_to_objects = ".git/objects/";
        let (hash_path, hash_file) = parse_args(&blob_file);

        let hash_path = path_to_objects.to_owned() + &hash_path + "/" + &hash_file.to_string();
        let mut hash_path = File::open(&hash_path).unwrap();

        hash_path.read_to_end(&mut file_content).unwrap();

        let compressed_data = &file_content[..];
        let (buffer, _bytes) = zlib::decode_data(compressed_data);
        print!("{}", &buffer[8..]);
    }

    pub fn write_obj(content_file: Vec<u8>, file_type: &str) -> String {
        #[allow(unsafe_code)]
        let content_file_ = unsafe { String::from_utf8_unchecked(content_file.clone()) };

        let header_blob = format!("{} {}\x00", file_type, content_file.len());

        let data_to_compress = header_blob + &format!("{}", content_file_);

        let (hash_blob_file, compressed_data) = zlib::encode_data(data_to_compress);

        let hash_dir = &hash_blob_file[..2];
        let hash_file = &hash_blob_file[2..];
        let hash_path_dir = format!(".git/objects/{}/", hash_dir);
        let full_hash_path = format!("{}{}", hash_path_dir, hash_file);

        fs::create_dir(hash_path_dir).unwrap();
        fs::write(full_hash_path, compressed_data).unwrap();

        hash_blob_file
    }

    pub fn parse_args(args: &String) -> (&str, &str) {
        let (hash_path, hash_file) = (&args[..2], &args[2..]);
        (hash_path, hash_file)
    }

    
    pub fn write_tree(file_path: &str) -> String {
        let mut sha_out: String = String::new();
        let mut entries: Vec<_> = fs::read_dir(file_path)
            .expect("Failed to read directory")
            .map(|res| res.expect("Failed to read entry").path())
            .collect();

        entries.sort();

        for dir in entries {
            let mode;
            let path_name = dir
                .as_path()
                .to_str()
                .expect("Failed to convert path to string");
            //println!("--- path_name : {} --- ", path_name);
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
                let content_file = fs::read(&path_name).unwrap();
                let sha_file1 = write_obj(content_file, "blob");
                sha_file = hex::decode(&sha_file1).expect("Failed to decode hex");
            }
            #[allow(unsafe_code)]
            let sha = unsafe { String::from_utf8_unchecked(sha_file) };

            sha_out += &format!(
                "{} {}\x00{}",
                mode,
                dir.file_name()
                    .expect("Failed to get file name")
                    .to_str()
                    .expect("Failed to convert file name to string"),
                sha
            );
        }

        let res_sha = write_obj(sha_out.into_bytes(), "tree");
        res_sha
    }

    pub fn read_tree_sha(sha_tree: String) {
        let mut file_content = Vec::new();

        let hash_dir = &sha_tree[..2];

        let hash_tree_object = &sha_tree[2..];

        let full_path = ".git/objects/".to_string() + &hash_dir + "/" + &hash_tree_object;
        let mut full_path = File::open(&full_path).unwrap();
        full_path.read_to_end(&mut file_content).unwrap();

        let _formatted_buff = String::new();
        let compressed_data = &file_content[..];
        let (formatted_buff, _bytes) = zlib::decode_data(compressed_data);

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

    pub fn do_commit(tree_sha: String, commit_sha: String, message: String) -> String {
        let content_commit = format!(
            "tree {}\nparent {}\nauthor ScotChacon <schacon@gmail.com> 1243040974 -0700\ncommitter ScotChacon <schacon@gmail.com> 1243040974 -0700\n\n",
            tree_sha, commit_sha
        );

        let content_commit = content_commit + &message + "\n";

        let sha_commit = write_obj(content_commit.into_bytes(), "commit");
        sha_commit
    }





/*
    async fn get_data(url: &str) -> &str {
        let response = reqwest::get(url).await.unwrap();
        let body = response.text().await.unwrap();
        body
    }
   */  
    pub async fn clone_repo(_dir_name: String, link: String) { 
       // let mut sha_refs = String::new();
       // let mut sha_head = String::new();

       fs::create_dir(_dir_name.clone()).unwrap();
       let dir_root = PathBuf::from(_dir_name.clone());
       env::set_current_dir(dir_root).unwrap();

       do_git_init();


        let link = format!("{}/info/refs?service=git-upload-pack",link);
        println!("link to search : {}", link);
        //let body = get_data(&link);
        let body = reqwest::blocking::get(link.clone()).unwrap().text().unwrap();
        let (sha_refs, sha_head) = extract_commit_hash(&body);
    

        print!("sha_refs : {}",&sha_refs);
        print!("sha_head : {}",&sha_head);


        let body = format!("0032want {}\n",sha_refs.clone());
        let data = get_data_form_git(link.clone(),body);
        
        let data_from_git = match data {
            Ok(data) => data,
            _ => panic!("Something go wrong with post request "),
        };

        println!("{:?}",data_from_git);
    
    }

    fn get_data_form_git(link: String, body : String) -> Result<bytes::Bytes, io::Error>{

    let mut headers = HeaderMap::new();
        headers.insert(
             CONTENT_TYPE,
                HeaderValue::from_static("application/x-git-upload-pack-request"),
            );

            let client = reqwest::blocking::Client::new();
    let client_req = client.post(link).headers(headers).body(body);
    let response_data = client_req.send().unwrap();
     
    let response_data = response_data.bytes().unwrap();
    Ok(response_data)
    }

    fn extract_commit_hash(response: &str) -> (&str,&str)  {

        println!("response : \n {}",response);
        let index = match response.find("refs/heads/master\n0000") {
            Some(index) => {index},
            None => panic!("panic occurs !")

        };
        let sha_refs = &response[index-45..index];

        println!("before head : \n {}",&response);

        let index = match response.find("HEAD") {
            Some(index) => {index},
            None => panic!("panic occurs !")

        };
        

        
        let sha_head = &response[index-45..index];

        
        (sha_refs, sha_head)
    }
}

