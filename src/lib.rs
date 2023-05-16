pub mod git {
    use reqwest::header::HeaderMap;
    use reqwest::header::HeaderValue;
    use reqwest::header::CONTENT_TYPE;

    mod zlib;
    extern crate hex;

    use std::collections::HashMap;

    use std::fs;
    use std::fs::File;
    use std::io;
    use std::io::Read;
    pub fn do_git_init() {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    }

    fn create_dir(dir_name: &str) {
        fs::create_dir(dir_name).unwrap();
        fs::create_dir(dir_name.to_owned() + "/.git").unwrap();
        fs::create_dir(dir_name.to_owned() + "/.git/objects/").unwrap();
        fs::create_dir(dir_name.to_owned() + "/.git/refs").unwrap();
        fs::write(
            dir_name.to_owned() + "/.git/HEAD",
            "ref: refs/heads/master\n",
        )
        .unwrap();
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

    pub fn write_obj(content_file: Vec<u8>, file_type: &str, target_dir: &str) -> String {
        #[allow(unsafe_code)]
        let content_file_ = unsafe { String::from_utf8_unchecked(content_file.clone()) };

        let header_blob = format!("{} {}\x00", file_type, content_file.len());

        let data_to_compress = header_blob + &format!("{}", content_file_);

        let (hash_blob_file, compressed_data) = zlib::encode_data(data_to_compress);

        let hash_dir = &hash_blob_file[..2];
        let hash_file = &hash_blob_file[2..];

        let mut _sub_hash_path_dir = String::new();
        let mut _full_hash_path_dir = String::new();

        if target_dir != "./" {
            _sub_hash_path_dir = format!("{}.git/objects/{}/", target_dir, hash_dir);
            _full_hash_path_dir = _sub_hash_path_dir.clone() + &hash_file;
            println!("sub_hash_path_dir : {:?}", _sub_hash_path_dir);
            println!("full_hash_path_dir : {:?}", _full_hash_path_dir);
        } else {
            _sub_hash_path_dir = format!(".git/objects/{}/", hash_dir);
            _full_hash_path_dir = format!("{}{}", _sub_hash_path_dir, hash_file);
        }

        fs::create_dir_all(_sub_hash_path_dir).unwrap();
        fs::write(_full_hash_path_dir, compressed_data).unwrap();

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
                let sha_file1 = write_obj(content_file, "blob", "./");
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

        let res_sha = write_obj(sha_out.into_bytes(), "tree", "./");
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

        let sha_commit = write_obj(content_commit.into_bytes(), "commit", "./");
        sha_commit
    }

    /*
     async fn get_data(url: &str) -> &str {
         let response = reqwest::get(url).await.unwrap();
         let body = response.text().await.unwrap();
         body
     }
    */
    pub fn clone_repo(dir_name: String, link: String) {
        // let mut sha_refs = String::new();
        // let mut sha_head = String::new();

        create_dir(&dir_name);

        let dir_obj = dir_name.to_owned() + "/.git/objects/";

        //let link_post =
        let post_url = link.clone() + &"/git-upload-pack".to_string();
        let link = format!("{}/info/refs?service=git-upload-pack", link);
        println!("link to search : {}", link);
        let body = reqwest::blocking::get(link.clone())
            .unwrap()
            .text()
            .unwrap();
        let (sha_refs, sha_head) = extract_commit_hash(&body);

        println!("sha_refs : {:?}", &sha_refs);

        println!("sha_head : {}", &sha_head);

        let sha_refs = &sha_refs[..40];

        let body = format!("0032want {}\n00000009done\n", &sha_refs);
        println!("post_url : {}, body : {} ", post_url, body);
        let data = get_data_form_git(post_url.clone(), body);

        let data_from_git = match data {
            Ok(data) => {
                println!("is ok");
                //println!("{:?} : data in match ", data);
                data
            }
            _ => panic!("Something go wrong with post request "),
        };

        //    print!("buff : {:?}", data_from_git);

        // --- parssing data --- //

        let git_data_size = data_from_git.len() - 20;
        println!("git_data_size : {}", git_data_size);
        let entries_bytes = &data_from_git[16..20];
        println!("entries_bytes : {:?}", entries_bytes);
        let num = u32::from_be_bytes(entries_bytes.try_into().unwrap());
        println!("num: {:?}", num);
        let data_bytes: Vec<u8> = data_from_git[20..git_data_size].try_into().unwrap();

        let mut objects = HashMap::new();
        let mut seek = 0;
        let mut obj_counter = 0;

        while obj_counter != num {
            obj_counter += 1;
            let first = data_bytes[seek];
            let mut obj_type: usize = ((first & 112) >> 4).into();
            println!("obj_type: {:?}", obj_type);
            while data_bytes[seek] > 128 {
                seek += 1;
            }
            seek += 1;
            // println!("seek : {:?}", seek);
            let data_type = [
                "",
                "commit",
                "tree",
                "blob",
                "",
                "tag",
                "ofs_delta",
                "refs_delta",
            ];
            if obj_type < 7 {
                println!("{}", obj_counter);

                let (git_data, bytes) = zlib::decode_data(&data_bytes[seek..]);

                let hash_obj =
                    write_obj(git_data.clone().into_bytes(), data_type[obj_type], &dir_obj);

                objects.insert(hash_obj, (git_data, obj_type));

                //    println!("{:?}", objects);

                seek += bytes;
            } else {
                let obj_data_bytes = &data_bytes[seek..seek + 20];

                // println!("obj_data_bytes : {:#?}", obj_data_bytes);

                let obj_data_bytes = hex::encode(obj_data_bytes);
                let (base, elem_num) = objects[&obj_data_bytes].to_owned();
                seek += 20;

                let (git_data, bytes) = zlib::decode_data(&data_bytes[seek..]);

                let content = apply_delta(&git_data.as_bytes(), &base.as_bytes());

                obj_type = elem_num;

                let hash_obj = write_obj(content.clone().into(), data_type[obj_type], &dir_obj);

                objects.insert(hash_obj, (content, obj_type));

                seek += bytes;
            }
        }

        let git_path_pack =
            dir_name.to_owned() + &format!("/.git/objects/{}/{}", &sha_refs[..2], &sha_refs[2..]);

        let git_data = fs::read(git_path_pack).unwrap();
        let (delta, _bytes) = zlib::decode_data(&git_data.to_vec());

        let data = delta.split("\n").next().unwrap().split(" ");

        let sha_obj = data.clone().nth(data.count() - 1).unwrap();
        println!("sha_obj: {:?}", &sha_obj);

        checkout(&sha_obj, &dir_name, &dir_obj);
    }
    //
    fn checkout(sha: &str, file_path: &str, dir_name: &str) {
        //do checkout
        fs::create_dir_all(&file_path).unwrap();

        let git_data =
            fs::read(dir_name.to_string() + &format!("{}/{}", &sha[..2], &sha[2..])).unwrap();

        let (s_git_data, _bytes) = zlib::decode_data(&git_data[..]);

        let mut enteries = Vec::new();

        let pos = s_git_data
            .as_bytes()
            .iter()
            .position(|&r| r == '\x00' as u8)
            .unwrap();

        let mut tree = &s_git_data[pos + 1..];

        while tree.len() > 0 {
            let pos = tree
                .as_bytes()
                .iter()
                .position(|&r| r == '\x00' as u8)
                .unwrap();

            let mode_name = &tree[..pos];

            let mut mode_name = mode_name.split(|num: char| num == ' ');

            let mode = mode_name.next().unwrap();
            let name = mode_name.next().unwrap();

            tree = &tree[pos + 1..];

            let sha = &tree[..20];

            tree = &tree[20..];

            println!("tree: {:#?}", &tree);

            let sha = hex::encode(&sha[..]);
            let mode = String::from_utf8_lossy(mode.as_bytes());
            let name = String::from_utf8_lossy(name.as_bytes());

            println!("mode: {:#?}", &mode);
            println!("name: {:#?}", &name);
            println!("sha: {:#?}", &sha);

            enteries.push((mode.clone(), name.clone(), sha.clone()));
        }
        for entry in enteries {
            if entry.0 == "40000" {
                //  println!("blob_sha 40000: {:#?}", &entry.1);
                checkout(
                    &entry.2.to_string(),
                    &(file_path.clone().to_owned() + &format!("/{}", entry.1).to_string()),
                    &dir_name.to_string(),
                );
            } else {
                let blob_sha = entry.2;

                let curr_dir = dir_name.clone().to_owned()
                    + &format!("/.git/objects/{}/{}", &blob_sha[..2], &blob_sha[2..]);

                let git_data = fs::read(curr_dir).unwrap();
                let (s_git_data, _bytes) = zlib::decode_data(&git_data[..]);

                let pos = s_git_data
                    .as_bytes()
                    .iter()
                    .position(|&r| r == '\x00' as u8)
                    .unwrap();

                let content = &s_git_data[pos + 1..];

                fs::write(
                    file_path.clone().to_owned() + &format!("/{}", entry.1),
                    content,
                )
                .unwrap();
            }
        }
    }

    fn apply_delta(delta: &[u8], base: &[u8]) -> String {
        let mut seek: usize = 0;
        // println!("delta: {:#?}", delta);
        while delta[seek] > 128 {
            seek += 1;
        }
        seek += 1;
        while delta[seek] > 128 {
            seek += 1;
        }
        seek += 1;
        let mut content = String::new();
        //content = "".to_string();
        //  println!("content: {:?}", &content);
        let delta_len = delta.len();
        // println!(" delta_len: {:?}", &delta_len);
        while seek < delta_len {
            let instr_byte = delta[seek];
            seek += 1;

            if instr_byte >= 128 {
                let offset_key = instr_byte & 0b00001111;

                let mut offset_bytes: [u8; 8] = [0; 8];

                for n in 0..8 {
                    let b = offset_key >> n & 1;

                    if b == 1 {
                        offset_bytes[n] = delta[seek];

                        seek += 1
                    }
                }
                let offset = usize::from_le_bytes(offset_bytes);

                let len_key = (instr_byte & 0b01110000) >> 4;

                let mut len_bytes: [u8; 8] = [0; 8];
                for n in 0..8 {
                    let b = len_key >> n & 1;
                    if b == 1 {
                        len_bytes[n] = delta[seek];

                        seek += 1
                    }
                }

                let len_int = usize::from_le_bytes(len_bytes);

                content += &String::from_utf8_lossy(&base[offset..(offset + len_int)]);
            } else {
                let num_bytes = instr_byte & 0b01111111;

                let num_bytes = usize::from(num_bytes);

                content += &String::from_utf8_lossy(&delta[seek..(seek + num_bytes)]);

                seek += num_bytes;
            }
        }
        content
    }

    fn get_data_form_git(link: String, body: String) -> Result<bytes::Bytes, io::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-git-upload-pack-request"),
        );

        let client = reqwest::blocking::Client::new();
        let client_req = client
            .post(link)
            .header("content-type", "application/x-git-upload-pack-request")
            .body(body);

        println!("client_req : {:#?}", client_req);
        println!("headers : {:#?}", headers.clone());
        let response_data = client_req.send().unwrap();

        println!(
            "response_data : {:#?} , status {}",
            response_data,
            response_data.status()
        );

        let response_data = response_data.bytes().unwrap();

        Ok(response_data)
    }

    fn extract_commit_hash(response: &str) -> (&str, &str) {
        println!("response : \n {}", response);
        let index = match response.find("refs/heads/master\n0000") {
            Some(index) => index,
            None => panic!("panic occurs !"),
        };
        let sha_refs = &response[index - 41..index];

        println!("before head : \n {}", &response);

        let index = match response.find("HEAD") {
            Some(index) => index,
            None => panic!("panic occurs !"),
        };

        let sha_head = &response[index - 41..index];

        (sha_refs, sha_head)
    }
}
