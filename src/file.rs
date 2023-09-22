pub mod file_import {
    use crate::data_structures::data_structures::BlockHeaderData;
    use regex::Regex;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    pub fn import_headers_file(filename: String) -> Vec<BlockHeaderData> {
        let file = File::open(filename).unwrap();
        let reader = BufReader::new(file);
        let mut data: Vec<BlockHeaderData> = Vec::new();

        // Read the file line by line using the lines() iterator from std::io::BufRead.
        let mut n_lines: u64 = 0;
        for line in reader.lines() {
            n_lines = n_lines + 1;
            let line = line.unwrap();
            let mut vals: Vec<&str> = line.split(" ").collect();

            let nonce = vals.pop().unwrap();
            let header = vals.pop().unwrap();

            let values: BlockHeaderData = BlockHeaderData {
                nonce: import_bin_string(nonce),
                header: import_bin_string(header),
            };
            data.push(values);
            let n_lines_print = n_lines;
            if n_lines_print % 10000 == 0 {
                println!("Processing hash file line: {}", n_lines_print);
            }
        }
        data
    }

    fn import_bin_string(string_array: &str) -> Vec<bool> {
        lazy_static! {
            static ref RE: Regex = Regex::new("([10])").unwrap();
        }
        let mut vals: Vec<bool> = Vec::new();
        for cap in RE.captures_iter(string_array) {
            match &cap[0] {
                "1" => vals.push(true),
                "0" => vals.push(false),
                _ => continue,
            }
        }
        vals
    }

    pub fn import_nonce_stats_file(filename: String) -> HashMap<usize, f32> {
        let file = File::open(filename).unwrap();
        let mut reader = BufReader::new(file);
        let mut data: HashMap<usize, f32> = HashMap::new();

        // Read the file line by line using the lines() iterator from std::io::BufRead.
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("Cannot open nonce stats file");
        let json_object = json::parse(&line);

        for (n, it) in json_object.unwrap().entries() {
            let mut prob_one = 0.0;
            let mut prob_zero = 0.0;

            for (k, vv) in it.entries() {
                match k {
                    "p0" => prob_zero = vv.as_f32().expect("Error extracting PZERO value"),
                    "p1" => prob_one = vv.as_f32().expect("Error extracting PONE value"),
                    _ => {}
                }
            }

            let mut entropy: f32 = 0.0;
            if prob_zero > 0.0 {
                entropy = entropy + prob_zero * prob_zero.log2();
            }
            if prob_one > 0.0 {
                entropy = entropy + prob_one * prob_one.log2();
            }
            entropy = -1.0 * entropy;
            let mut key: usize = n.parse().unwrap();
            key = key - 1;
            data.insert(key, entropy);
        }
        data
    }
}

pub mod file_export {
    use crate::data_structures::data_structures::Statistic;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn save_stats_to_file(stats: Vec<Statistic>) {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let file_name = format!("experiment04_{}.json", since_the_epoch.as_secs().to_string());

        let path = Path::new(&file_name);
        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        println!("Dumping data to file {}", display);
        for s in stats {
            let val = json::stringify(s);
            match file.write_all(val.as_bytes()) {
                Err(why) => panic!("couldn't write to {}: {}", display, why),
                _ => {}
            }
            file.write_all("\n".as_bytes())
                .expect("Cannot write to file!");
        }
    }
}
