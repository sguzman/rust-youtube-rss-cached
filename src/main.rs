/*
    * This file is part of youtube rss cached project of mine.
    * This program will fetch youtube rss feed and cache it in a file by entry
    *
*/ 

extern crate quick_xml;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

struct MediaCommunity {
    statistics: u64,
    star_rating: u64,
    category: Option<String>,
    keywords: Option<String>,
}

// Function to handl parsing media community element from xml
fn parse_media_community(reader: &mut Reader<&[u8]>) -> Result<(), quick_xml::Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"media:statistics" => {
                        let statistics = reader.read_text(e.name()).unwrap();
                        println!("\tStatistics: {}", statistics);
                    }
                    b"media:starRating" => {
                        let star_rating = reader.read_text(e.name()).unwrap();
                        println!("\tStar Rating: {}", star_rating);
                    }
                    b"media:category" => {
                        let category = reader.read_text(e.name()).unwrap();
                        println!("\tCategory: {}", category);
                    }
                    b"media:keywords" => {
                        let keywords = reader.read_text(e.name()).unwrap();
                        println!("\tKeywords: {}", keywords);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"media:community" => return Ok(()),
                    _ => (),
                }
            }
            Ok(Event::Eof) => panic!("Error not find {} end element", "media:community"),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        buf.clear();
    }
}

// Function to handl parsing media group element from xml
fn parse_media_group(reader: &mut Reader<&[u8]>) -> Result<(), quick_xml::Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"media:title" => {
                        let title = reader.read_text(e.name()).unwrap();
                        println!("\tTitle: {}", title);
                    }
                    b"media:content" => {
                        let content = reader.read_text(e.name()).unwrap();
                        println!("\tContent: {}", content);
                    }
                    b"media:thumbnail" => {
                        let thumbnail = reader.read_text(e.name()).unwrap();
                        println!("\tThumbnail: {}", thumbnail);
                    }
                    b"media:description" => {
                        let description = reader.read_text(e.name()).unwrap();
                        println!("\tdescription: {}", description);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"media:group" => return Ok(()),
                    _ => (),
                }
            }
            Ok(Event::Eof) => panic!("Error not find {} end element", "media:group"),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        buf.clear();
    }
}

// Function to handl parsing author from xml
fn parse_author(reader: &mut Reader<&[u8]>) -> Result<(), quick_xml::Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"name" => {
                        let name = reader.read_text(e.name()).unwrap();
                        println!("\tName: {}", name);
                    }
                    b"uri" => {
                        let uri = reader.read_text(e.name()).unwrap();
                        println!("\tUri: {}", uri);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"author" => return Ok(()),
                    _ => (),
                }
            }
            Ok(Event::Eof) => panic!("Error not find {} end element", "author"),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        buf.clear();
    }
}

// Function to handle parsing entry from xml
fn parse_entry(reader: &mut Reader<&[u8]>) -> Result<(), quick_xml::Error> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"yt:videoId" => {
                        let title = reader.read_text(e.name()).unwrap();
                        println!("yt:videoId: \"{}\"", title);
                    }
                    b"yt:channelId" => {
                        let title = reader.read_text(e.name()).unwrap();
                        println!("yt:channelId: \"{}\"", title);
                    }
                    b"title" => {
                        let title = reader.read_text(e.name()).unwrap();
                        println!("Title: {}", title);
                    }
                    // author
                    b"author" => {
                        parse_author(reader).unwrap();
                    }
                    b"published" => {
                        let published = reader.read_text(e.name()).unwrap();
                        println!("Published: {}", published);
                    }
                    b"updated" => {
                        let updated = reader.read_text(e.name()).unwrap();
                        println!("Updated: {}", updated);
                    }
                    b"media:group" => {
                        parse_media_group(reader).unwrap();
                    }
                    _ => (),
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"entry" => return Ok(()),
                    _ => (),
                }
            }
            Ok(Event::Eof) => panic!("Error not find {} end element", "entry"),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        buf.clear();
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

            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"entry" => {
                        parse_entry(&mut reader).unwrap();
                    },
                    _ => (),
                }
            }
            _ => (),
        }
        buf.clear();
    }
}