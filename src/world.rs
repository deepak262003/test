use std::cell::OnceCell;
use std::collections::HashMap;

use crate::fonts::{FontSearcher, FontSlot};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Datelike, Local};
use comemo::Prehashed;
use typst::diag::{FileResult, StrResult,FileError};
use typst::eval::{Bytes, Datetime, Library};
use typst::font::{Font, FontBook};
use typst::syntax::{FileId, Source, VirtualPath};

pub struct SimpleWorld { 
    main: FileId,
    source_string_encoded: String,
    library: Prehashed<Library>,
    book: Prehashed<FontBook>,
    now: OnceCell<DateTime<Local>>,
    fonts: Vec<FontSlot>,
    image: String,
    images: Vec<Bytes>,
    templates: Vec<String>,
    image_name_hash: HashMap<String,usize>,
    templates_data_hash: HashMap<String,usize>
}

impl SimpleWorld {
    pub fn new(typst_source: String,image_source: Vec<Vec<u8>>,image_hash:HashMap<String,usize>,templates_data:Vec<String>,templates_hash:HashMap<String,usize>) -> StrResult<Self> {
        let image_d = String::from("data");
        let mut searcher = FontSearcher::new();
        searcher.search(&[]);

        let mut image_data : Vec<Bytes> = Vec::new();
        let mut image_data_hash: HashMap<String, usize> = image_hash;
        let vpa = VirtualPath::new(".");
        for img in image_source{
            image_data.push(img.clone().into());
        }

        Ok(Self {
            main: FileId::new(None, vpa),
            source_string_encoded: typst_source,
            library: Prehashed::new(typst_library::build()),
            book: Prehashed::new(searcher.book),
            fonts: searcher.fonts,
            now: OnceCell::new(),
            image:image_d.to_string(),
            images: image_data, //vector of images base64
            templates:templates_data, //vector of templates string
            image_name_hash:image_data_hash,  //maps images data position in vector to file name
            templates_data_hash:templates_hash //maps templates data position in vector to file name
        })
    }
    
    fn _get_source_sample(&self) -> Source {
        let vpa = VirtualPath::new(".");
        let fid = FileId::new(None, vpa);
        let text = "text".to_string();
        Source::new(fid, text)
    }

    fn get_source(&self) -> Source {
        let vpa = VirtualPath::new(".");
        let fid = FileId::new(None, vpa);
        let text = self.source_string_encoded.to_string();
        let text_decoded_b = general_purpose::STANDARD.decode(text).unwrap();
        let text_decoded = decode_utf8(text_decoded_b).unwrap();
        Source::new(fid, text_decoded)
    }

}

impl typst::World for SimpleWorld {
    fn main(&self) -> Source {
        self.get_source()
    }

    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn source(&self, id: FileId) -> FileResult<Source> { //gets templates file from typst code and return the string data from vector
        if let Some(template_data) = id
        .vpath()
        .as_rootless_path()
        .to_str()
        .and_then(|p| self.templates.get(match (self.templates_data_hash.get(p)){
            Some(value)=>{
              *value
            }
            None=>{ 100 }
        }))
    {
        FileResult::Ok(Source::new(id,template_data.clone()))
    } else {
        FileResult::Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }
       
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> { //gets image name from typst code and returns the bytes data from vector
    
        if let Some(image) = id
            .vpath()
            .as_rootless_path()
            .to_str()
            .and_then(|p| self.images.get(match (self.image_name_hash.get(p)){
                Some(value)=>{
                  *value
                }
                None=>{ 100 }
            }))
        {
            Ok(image.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = self.now.get_or_init(chrono::Local::now);

        let naive = match offset {
            None => now.naive_local(),
            Some(o) => now.naive_utc() + chrono::Duration::hours(o),
        };

        Datetime::from_ymd(
            naive.year(),
            naive.month().try_into().ok()?,
            naive.day().try_into().ok()?,
        )
    }
}

/// Decode UTF-8 with an optional BOM.
fn decode_utf8(buf: Vec<u8>) -> FileResult<String> {
    Ok(if buf.starts_with(b"\xef\xbb\xbf") {
        // Remove UTF-8 BOM.
        std::str::from_utf8(&buf[3..])?.into()
    } else {
        // Assume UTF-8.
        String::from_utf8(buf)?
    })
}

