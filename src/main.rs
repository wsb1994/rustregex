#![allow(dead_code)]

use regex::*;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Instant;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use scoped_threadpool::Pool;
use std::collections::HashMap;

//thread_pool size
pub const THREAD_POOL: u32 = 4;

fn strip(filename: &str) -> Result<Vec<String>, Error> {
    let file = slurp::read_all_to_string(filename).unwrap();
    let regex = Regex::new(r"[^a-zA-Z\d\s'\)]").unwrap();
    let stripped_file = regex.replace_all(&file, " ");

    let words: Vec<String> = stripped_file.split_whitespace().map(String::from).collect();
    Ok(words)
}

fn native_filelist(filename: &str) -> Result<Vec<String>, Error> {
    let file = File::open(filename).unwrap();
    let buf = BufReader::new(file);
    Ok(buf
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect())
}

fn slurp_filelist(filename: &str) -> Vec<String> {
    let file = slurp::read_all_lines(filename).unwrap();
    file
}

fn par_regex(filenames: Vec<String>) -> Vec<Vec<String>> {
    let mut sanitized_files = vec![];
    let mut pool = Pool::new(THREAD_POOL);
    let (send, recv) = channel();
    pool.scoped(|scope| {
        for file in filenames {
            let path = Path::new(&file).exists();
            if path {
                let sender = send.clone();
                scope.execute(move || {
                    sender.send(strip(&file).unwrap()).unwrap();
                })
            }
        }
    });

    for file in recv.try_recv() {
        sanitized_files.push(file);
    }

    sanitized_files
}

fn inline_regex(filenames: Vec<String>) -> Vec<Vec<String>> {
    let mut sanitized_files = vec![];
    for file in filenames {
        sanitized_files.push(strip(&file.to_owned()).unwrap());
    }
    sanitized_files
}

fn crossbeam_regex(filenames: Vec<String>) -> Vec<Vec<String>> {
    let mut sanitized_files = vec![];
    let length = filenames.len();
    let (send, recv) = channel();

    crossbeam::scope(|scope| {
        for file in filenames {
            let sender = send.clone();
            scope.spawn(move |_| {
                sender.send(strip(&file).unwrap()).unwrap();
            });
        }
    })
    .unwrap();

    for _ in 0..length {
        let value = recv.recv().unwrap();
        sanitized_files.push(value);
    }

    sanitized_files
}

fn generate_tuple(sanitized_file: Vec<String>) -> Vec<(String, u32)> {
    let mut current_hashmap = HashMap::new();
    let mut words: Vec<(String, u32)> = Vec::new();
    for word in sanitized_file {
        let count = current_hashmap.entry(word).or_insert(0);
        *count += 1;
    }

    for (word, count) in current_hashmap {
        words.push((word, count));
    }

    words
}

fn generate_list(
    file_names: Vec<String>,
    sanitized_files: Vec<Vec<String>>,
) -> Vec<Vec<(String, u32)>> {
    let mut results = vec![];
    for i in 0..file_names.len() {
        results.push(generate_tuple(sanitized_files[i].to_owned()));
    }
    results
}
/*
fn generate_report(
    filename: Vec<String>,
    sanitized_files: Vec<Vec<String>>,
) -> (String, Vec<Vec<(String, u32)>>) {
}*/
fn clean_filelist(files: Vec<String>) -> Vec<String> {
    let mut files = files;
    files.retain(|x| Path::new(&x).exists());
    return files;
}

fn generate_report(filenames: Vec<String>, results: Vec<Vec<(String, u32)>>) {
    for tuple in filenames.iter().zip(results) {
        {
            let filename = tuple.0;
            let file_name = format!("{}{}", filename.to_owned(), "_report.txt".to_owned());
            let words_list = tuple.1;

            let mut f = File::create(file_name.to_owned()).expect("Unable to create file");
            f.write_all(file_name.as_bytes())
                .expect("unable to write to file");
            f.write_all("\n".as_bytes())
                .expect("Unable to write to file");

            for word_tuples in words_list {
                let line = format!("{} {} \n", word_tuples.0, word_tuples.1);
                f.write_all(line.as_bytes())
                    .expect("Unable to write to file");
            }
        }
    }
}

fn sort_tuples(results: &mut Vec<Vec<(String, u32)>>) {
    for values in results {
        values.sort_by_key(|k| k.1);
    }
}
fn main() {
    let now = Instant::now();
    let filenames = slurp_filelist("filenames.txt");
    let filenames = clean_filelist(filenames);
    let word_lists = crossbeam_regex(filenames.to_owned());
    let mut results = generate_list(filenames.to_owned(), word_lists);
    sort_tuples(&mut results);
    generate_report(filenames, results);
    println!("results in : {} \n", now.elapsed().as_secs_f32());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_native_inline() {
        let now = Instant::now();
        let native = native_filelist("filenames.txt").unwrap();
        let _ = inline_regex(native);
        println!("test_native_par took {} \n", now.elapsed().as_secs_f32());
    }

    #[test]
    fn test_native_crossbeam() {
        let now = Instant::now();
        let native = native_filelist("filenames.txt").unwrap();
        let _ = crossbeam_regex(native);
        println!(
            "test_native_crossbeam took {} \n",
            now.elapsed().as_secs_f32()
        );
    }

    #[test]
    fn test_native_pool() {
        let now = Instant::now();
        let native = native_filelist("filenames.txt").unwrap();
        let _ = par_regex(native);
        println!("test_native_pool took {} \n", now.elapsed().as_secs_f32());
    }

    #[test]
    fn test_slurp_inline() {
        let now = Instant::now();
        let native = slurp_filelist("filenames.txt");
        let _ = inline_regex(native);
        println!("test_slurp_par took {} \n", now.elapsed().as_secs_f32());
    }

    #[test]
    fn test_slurp_crossbeam() {
        let now = Instant::now();
        let native = slurp_filelist("filenames.txt");
        let _ = crossbeam_regex(native);
        println!(
            "test_slurp_crossbeam took {} \n",
            now.elapsed().as_secs_f32()
        );
    }

    #[test]
    fn test_slurp_pool() {
        let now = Instant::now();
        let native = slurp_filelist("filenames.txt");
        let _ = par_regex(native);
        println!("test_slurp_pool took {} \n", now.elapsed().as_secs_f32());
    }
}
