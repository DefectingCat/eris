use anyhow::Result;
use consts::RESET_CSS;

use scraper::{Html, Selector};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use ziper::Ziper;

mod consts;
mod gegenees;
mod ziper;

fn main() -> Result<()> {
    let path = PathBuf::from("./test/test.zip");
    let mut ziper = Ziper::new(&path)?;
    ziper.unzip(Some("./test"))?;

    let prefix = PathBuf::from("./test");
    let index_path: PathBuf = {
        let mut p = prefix.clone();
        p.push("index.html");
        p
    };
    let mut index_file = File::options()
        .read(true)
        .write(true)
        .append(true)
        .open(&index_path)?;
    let mut index = String::new();
    index_file.read_to_string(&mut index)?;
    let doc = Html::parse_document(&index);
    let body_selector = Selector::parse("body").unwrap();
    // body tag should has one
    let body = doc.select(&body_selector).next().unwrap();

    // add data attributes
    // let image_selector = Selector::parse("img").unwrap();
    // for img in body.select(&image_selector) {
    //     let src = img
    //         .value()
    //         .attrs
    //         .get(&QualName {
    //             prefix: None,
    //             ns: Namespace::from(""),
    //             local: LocalName::from("src"),
    //         })
    //         .unwrap();
    //     dbg!(&src);

    //     let data_img = QualName {
    //         prefix: None,
    //         ns: Namespace::from(""),
    //         local: LocalName::from("data-template"),
    //     };
    //     let data_v: Tendril<UTF8> = Tendril::from("test");

    //     let mut new_img = img;
    //     new_img.value().attrs.insert(data_img, data_v);
    //     dbg!(&new_img);
    // }

    let style_path = {
        let mut p = prefix;
        p.push("style.css");
        p
    };
    let style_file = fs::read_to_string(style_path)?;
    let styles = format!("<style>\n{}\n{}\n</style>", style_file, RESET_CSS);
    let html = format!("{}\n{}", styles, body.html());
    index_file.set_len(0)?;
    // rewrite body tag to html file
    index_file.write_all(html.as_bytes())?;

    // rename index file
    let mut new_name = PathBuf::from(&index_path);
    new_name.set_file_name("template.html");
    fs::rename(index_path, new_name)?;

    Ok(())
}
