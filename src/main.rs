use anyhow::Result;
use consts::RESET_CSS;

use html5ever::{
    tendril::{fmt::UTF8, Tendril},
    LocalName, Namespace, QualName,
};
use regex::Regex;
use scraper::{Element, Html, Selector};
use std::{
    env::{self},
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use ziper::Ziper;

use crate::consts::TEXT_REG;

mod consts;
mod gegenees;
mod ziper;

fn main() -> Result<()> {
    let args = env::args();
    dbg!(&args);

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

    let mut body_html = body.html();

    // add data attributes to images
    let image_selector = Selector::parse("img").unwrap();
    for img in body.select(&image_selector) {
        let mut new_img = img.value().clone();
        let data_img = QualName {
            prefix: None,
            ns: Namespace::from(""),
            local: LocalName::from("data-template"),
        };
        let data_v: Tendril<UTF8> = Tendril::from("img");
        new_img.attrs.insert(data_img, data_v);

        let data_title = QualName {
            prefix: None,
            ns: Namespace::from(""),
            local: LocalName::from("data-title"),
        };
        let title_v: Tendril<UTF8> = Tendril::from("待替换");
        new_img.attrs.insert(data_title, title_v);

        let new_h = format!("{:?}", new_img);
        let h = img.html();
        body_html = body_html.replace(&h, &new_h);
    }

    // add data attributes to texts
    let re = Regex::new(TEXT_REG).unwrap();
    let new_body_html = body_html.clone();
    re.find_iter(&new_body_html).for_each(|m| {
        // m.as_str() = "<div class=\"div\">三维沉浸式场景</div>"
        let h = m.as_str();
        let element = Html::parse_fragment(h);
        let value = element
            .root_element()
            .first_element_child()
            .unwrap()
            .value();
        let mut new_text = value.clone();

        let data_text = QualName {
            prefix: None,
            ns: Namespace::from(""),
            local: LocalName::from("data-template"),
        };
        let data_v: Tendril<UTF8> = Tendril::from("text");
        new_text.attrs.insert(data_text, data_v);

        let data_title = QualName {
            prefix: None,
            ns: Namespace::from(""),
            local: LocalName::from("data-title"),
        };
        let title_v: Tendril<UTF8> = Tendril::from("待替换");
        new_text.attrs.insert(data_title, title_v);
        let new_h = format!("{:?}", new_text);

        let mut replace_h = String::from(h);
        replace_h = replace_h.replace(&format!("{:?}", value), &new_h);

        body_html = body_html.replace(h, &replace_h);
    });

    let style_path = {
        let mut p = prefix;
        p.push("style.css");
        p
    };
    let style_file = fs::read_to_string(style_path)?;
    let styles = format!("<style>\n{}\n{}\n</style>", style_file, RESET_CSS);
    let html = format!("{}\n{}", styles, body_html);
    index_file.set_len(0)?;
    // rewrite body tag to html file
    index_file.write_all(html.as_bytes())?;

    // rename index file
    let mut new_name = PathBuf::from(&index_path);
    new_name.set_file_name("template.html");
    fs::rename(index_path, new_name)?;

    Ok(())
}
