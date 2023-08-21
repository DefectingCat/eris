use anyhow::Result;
use scraper::{Html, Selector};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use ziper::Ziper;

mod gegenees;
mod ziper;

fn main() -> Result<()> {
    let path = PathBuf::from("./test/test.zip");
    let mut ziper = Ziper::new(&path)?;
    ziper.unzip(Some("./test"))?;

    let prefix = PathBuf::from("./test");
    let index = {
        let mut p = prefix.clone();
        p.push("index.html");
        p
    };
    let mut index_file = File::options()
        .read(true)
        .write(true)
        .append(true)
        .open(&index)?;
    let mut index = String::new();
    index_file.read_to_string(&mut index)?;
    let doc = Html::parse_document(&index);
    let body_selector = Selector::parse("body").unwrap();
    // body tag should has one
    let body = doc.select(&body_selector).next().unwrap();
    index_file.set_len(0)?;
    // rewrite body tag to html file
    index_file.write_all(body.html().as_bytes())?;

    let style = {
        let mut p = prefix;
        p.push("style.css");
        p
    };
    let _style_file = fs::read_to_string(style)?;
    let _style = Html::new_fragment();

    Ok(())
}
