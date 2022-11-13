/*
 * This file is part of youtube rss cached project of mine.
 * This program will fetch youtube rss feed and cache it in a file by entry
 *
*/

extern crate md5;
extern crate quick_xml;
extern crate rayon;
extern crate serde;
extern crate serde_json;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

// Struct for final json object to be serialized
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EntryOptional {
    pub video_id: Option<String>,
    pub channel_id: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub published: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
struct Entry {
    video_id: String,
    channel_id: String,
    title: String,
    author: String,
    published: String,
}

const NULL_ENTRY: EntryOptional = EntryOptional {
    video_id: None,
    channel_id: None,
    title: None,
    author: None,
    published: None,
};

// Return all files in a directory
fn get_files(path: &str) -> Vec<String> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            files.push(path.to_str().unwrap().to_string());
        }
    }

    files
}

fn get_file_string(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}

// Compute md5 hash of a string
fn get_md5_hash(s: &str) -> String {
    format!("{:x}", md5::compute(s))
}

// Write JSON object to a file
fn write_file_string(path: &str, json: &str) {
    let digest = get_md5_hash(json);
    let file_path = format!("{}/{}.json", path, digest);

    std::fs::write(file_path, json).unwrap();
}

// Function that retrieves first cmd line argument and returns it
fn get_src_dir() -> String {
    let args: Vec<String> = std::env::args().collect();
    let arg = &args[1];
    arg.to_string()
}

fn get_dst_dir() -> String {
    let args: Vec<String> = std::env::args().collect();
    let arg = &args[2];
    arg.to_string()
}

// Function to handl parsing author from xml
fn parse_author(reader: &mut Reader<&[u8]>) -> Option<String> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"name" => {
                    let name = reader.read_text(e.name()).unwrap();
                    return Some(name.into_owned().clone());
                }
                _ => (),
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"author" => return None,
                _ => (),
            },
            Ok(Event::Eof) => {
                println!("Error not find {} end element", "author");
                return None;
            }
            Err(e) => {
                println!("Error at position {}: {:?}", reader.buffer_position(), e);
                return None;
            }
            _ => (),
        }
        buf.clear();
    }
}

// Function to handle parsing entry from xml
fn parse_entry(
    reader: &mut Reader<&[u8]>,
    entry_op: Option<EntryOptional>,
) -> Option<EntryOptional> {
    let mut buf = Vec::new();
    if entry_op.is_none() {
        return None;
    }

    let entry = entry_op.unwrap().clone();

    match reader.read_event_into(&mut buf) {
        Ok(Event::Start(e)) => {
            match e.name().as_ref() {
                b"yt:videoId" => {
                    let video_id = reader.read_text(e.name()).unwrap();
                    return parse_entry(
                        reader,
                        Some(EntryOptional {
                            video_id: Some(video_id.into_owned().clone()),
                            channel_id: entry.channel_id,
                            title: entry.title,
                            author: entry.author,
                            published: entry.published,
                        }),
                    );
                }
                b"yt:channelId" => {
                    let title = reader.read_text(e.name()).unwrap();
                    return parse_entry(
                        reader,
                        Some(EntryOptional {
                            video_id: entry.video_id,
                            channel_id: Some(title.into_owned().clone()),
                            title: entry.title,
                            author: entry.author,
                            published: entry.published,
                        }),
                    );
                }
                b"title" => {
                    let title = reader.read_text(e.name()).unwrap();
                    return parse_entry(
                        reader,
                        Some(EntryOptional {
                            video_id: entry.video_id,
                            channel_id: entry.channel_id,
                            title: Some(title.into_owned().clone()),
                            author: entry.author,
                            published: entry.published,
                        }),
                    );
                }
                // author
                b"author" => {
                    let author = parse_author(reader).unwrap();
                    return parse_entry(
                        reader,
                        Some(EntryOptional {
                            video_id: entry.video_id,
                            channel_id: entry.channel_id,
                            title: entry.title,
                            author: Some(author),
                            published: entry.published,
                        }),
                    );
                }
                b"published" => {
                    let published = reader.read_text(e.name()).unwrap();
                    return parse_entry(
                        reader,
                        Some(EntryOptional {
                            video_id: entry.video_id,
                            channel_id: entry.channel_id,
                            title: entry.title,
                            author: entry.author,
                            published: Some(published.into_owned().clone()),
                        }),
                    );
                }
                _ => return parse_entry(reader, Some(entry)),
            }
        }
        Ok(Event::End(e)) => match e.name().as_ref() {
            b"entry" => return Some(entry),
            _ => return parse_entry(reader, Some(entry)),
        },
        Ok(Event::Eof) => panic!("Error not find {} end element", "entry"),
        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        _ => return parse_entry(reader, Some(entry)),
    }
}

// Function to handle parsing xml
fn parse(xml: &str, dst: &str) -> () {
    // Load file
    let xml = get_file_string(xml);
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);
    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"entry" => {
                    if let Some(entry) = parse_entry(&mut reader, Some(NULL_ENTRY)) {
                        let entry = Entry {
                            video_id: entry.video_id.unwrap(),
                            channel_id: entry.channel_id.unwrap(),
                            title: entry.title.unwrap(),
                            author: entry.author.unwrap(),
                            published: entry.published.unwrap(),
                        };

                        let string: String = serde_json::to_string(&entry).unwrap();
                        let string: &str = string.as_ref();
                        write_file_string(dst, string);
                        println!("{}", string.len());
                    }
                }
                _ => (),
            },
            _ => (),
        }
        buf.clear();
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(16)
        .build_global()
        .unwrap();

    println!("{}", "bye :(");
    // Get path to directory of xml files
    let src = get_src_dir();
    let dst = get_dst_dir();

    // Get all files in directory
    let files = get_files(&src);
    println!("Processing {} files", files.len());

    // Use Rayon to parse files in parallel
    files.par_iter().for_each(|file| {
        parse(file, &dst);
    });

    println!("{}", "bye :(")
}
