use anyhow::{anyhow, bail, Result};
use ego_tree::NodeRef;
use html5ever::{
    tendril::{fmt::UTF8, Tendril},
    LocalName, Namespace, QualName,
};
use regex::Regex;
use scraper::{Element, Html, Node, Selector};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    consts::{RESET_CSS, TEXT_REG},
    ziper::Ziper,
};

#[derive(Debug)]
pub struct Sisyphus {
    /// Target directory
    pub directory: PathBuf,
    // Target zip files
    file_list: Vec<PathBuf>,
}

impl Sisyphus {
    /// Sisyphus builder.
    ///
    /// Create new Sisyphus struct.
    ///
    /// - `directory`: target diretory. Sisyphus will read all zip file in this directory.
    pub fn new(directory: &PathBuf) -> Result<Self> {
        let target_paths = fs::read_dir(directory)?;
        let file_list = target_paths.fold(vec![], |mut prev, path| {
            let target = match path {
                Ok(p) => {
                    let file_name = p.file_name();
                    let file_type = if let Ok(t) = p.file_type() {
                        t
                    } else {
                        eprintln!(
                            "Error: read file {} file type failed",
                            file_name.to_string_lossy()
                        );
                        return prev;
                    };
                    if file_type.is_file() && file_name.to_string_lossy().ends_with(".zip") {
                        p.path()
                    } else {
                        return prev;
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return prev;
                }
            };
            prev.push(target);
            prev
        });

        let s = Self {
            directory: PathBuf::from(directory),
            file_list,
        };
        dbg!(&s);
        Ok(s)
    }

    /// Unzip target
    ///
    /// - `path` unzip to folder
    fn unzip(&self, path: &Path) -> Result<()> {
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("conver filename failed")),
        };
        // create same name folder
        let name = self.format_name(&file_name);
        let mut dir_path = PathBuf::from(&self.directory);
        dir_path.push(name);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
        }
        let mut ziper = Ziper::new(path)?;
        let dir_path = dir_path.to_string_lossy();
        ziper.unzip(Some(&dir_path))?;
        Ok(())
    }

    pub fn process(&self) -> Result<()> {
        for file in &self.file_list {
            self.unzip(file)?;

            let file = file.to_string_lossy();
            let folder_prefix = self.format_name(&file);
            let index_path = {
                let mut p = PathBuf::from(folder_prefix);
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
            // let doc = self.parse_html(&index_path)?;
            let body_selector = Selector::parse("body").unwrap();
            let body = doc.select(&body_selector).next().unwrap();
            let mut html = body.html();

            // add data attributes to images
            let image_selector = Selector::parse("img").unwrap();
            for img in body.select(&image_selector) {
                let old_img = img.value();
                let mut new_img = old_img.clone();
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
                let h = format!("{:?}", old_img);
                html = html.replace(&h, &new_h);
            }

            // add data attributes to texts
            body.children().for_each(|child| {
                self.traverse_node(&child, &mut html);
            });

            let style_path = {
                let mut p = PathBuf::from(folder_prefix);
                p.push("style.css");
                p
            };
            let style_file = fs::read_to_string(style_path)?;
            let styles = format!("<style>\n{}\n{}\n</style>", style_file, RESET_CSS);
            let html = format!("{}\n{}", styles, html);
            index_file.set_len(0)?;
            // rewrite body tag to html file
            index_file.write_all(html.as_bytes())?;

            // rename index file
            let mut new_name = PathBuf::from(&index_path);
            new_name.set_file_name("template.html");
            fs::rename(index_path, new_name)?;
        }
        Ok(())
    }

    fn traverse_node(&self, target: &NodeRef<Node>, mut html: &mut String) -> Result<()> {
        let v = target.value();
        if v.is_text() {
            let is_vaild =
                self.vaild_text(&v.as_text().map(|t| t.to_string()).unwrap_or(String::new()));
            if is_vaild {
                let parent = target.parent().unwrap();
                let text = parent.value().as_element().unwrap();
                dbg!(&text);
                let mut new_text = text.clone();
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
                let old_h = format!("{:?}", text);
                *html = html.replace(&old_h, &new_h);
            }
        } else {
            target.children().for_each(|child| {
                self.traverse_node(&child, &mut html);
            });
        }
        Ok(())
    }

    fn vaild_text(&self, text: &str) -> bool {
        let t = text.split('\n').collect::<Vec<_>>();
        let is_vaild = t.iter().all(|text| !text.trim().is_empty());
        if is_vaild {
            dbg!(t);
        }
        is_vaild
    }

    /// Parse index html file to string
    ///
    /// - `path`: &Path document path
    fn _parse_html(&self, path: &Path) -> Result<Html> {
        let mut index_file = File::options()
            .read(true)
            .write(true)
            .append(true)
            .open(path)?;
        let mut index = String::new();
        index_file.read_to_string(&mut index)?;
        let doc = Html::parse_document(&index);
        // body tag should has one
        // let body = doc.select(&body_selector).next().unwrap();
        Ok(doc)
    }

    /// Format filename with extention
    ///
    /// - `file_name` target file name, such as `test.zip`
    fn format_name<'a>(&self, file_name: &'a str) -> &'a str {
        let name = file_name.split('.');
        let name = name.collect::<Vec<_>>();
        let name = name.first();
        if let Some(n) = name {
            n
        } else {
            ""
        }
    }
}
