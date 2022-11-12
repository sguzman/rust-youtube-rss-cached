/*
 * This file is part of youtube rss cached project of mine.
 * This program will fetch youtube rss feed and cache it in a file by entry
 *
*/

extern crate quick_xml;
extern crate serde;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Result;

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

fn main() {
    let xml: String = std::fs::read_to_string("./data/src/template.xml").unwrap();

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
                        println!("{}", serde_json::to_string(&entry).unwrap());
                    }
                }
                _ => (),
            },
            _ => (),
        }
        buf.clear();
    }
}
