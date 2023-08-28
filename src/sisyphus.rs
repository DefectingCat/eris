use anyhow::{anyhow, Context, Result};
use ego_tree::NodeRef;
use html5ever::{
    tendril::{fmt::UTF8, Tendril},
    LocalName, Namespace, QualName,
};

use scraper::{Html, Node, Selector};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{consts::RESET_CSS, ziper::Ziper};

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
                    let file_name = file_name.to_string_lossy();
                    let file_type = if let Ok(t) = p.file_type() {
                        t
                    } else {
                        eprintln!("Error: read file {} file type failed", file_name);
                        return prev;
                    };
                    if file_type.is_file() && file_name.ends_with(".zip") {
                        p.path()
                    } else {
                        return prev;
                    }
                }
                Err(err) => {
                    eprintln!("Error: read path failed {}", err);
                    return prev;
                }
            };
            prev.push(target);
            prev
        });

        if file_list.is_empty() {
            println!("No zip file found in {:?}", directory);
        } else {
            let len = file_list.len();
            println!("Found {} files", len);
        }

        let s = Self {
            directory: PathBuf::from(directory),
            file_list,
        };
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
        println!("String unzip {:?}", path);
        let mut ziper = Ziper::new(path)?;
        let dir_path = dir_path.to_string_lossy();
        ziper.unzip(Some(&dir_path))?;
        Ok(())
    }

    pub fn process(&self) -> Result<()> {
        for file in &self.file_list {
            self.unzip(file)?;

            let file = file.to_string_lossy();
            let folder_prefix = &file[..file.len() - 4];
            let index_path = {
                let mut p = PathBuf::from(&folder_prefix);
                p.push("index.html");
                p
            };
            let mut index_file = File::options()
                .read(true)
                .write(true)
                .append(true)
                .open(&index_path)
                .context(format!("Cannot open {:?}", index_path))?;
            let mut index = String::new();
            index_file.read_to_string(&mut index)?;
            let doc = Html::parse_document(&index);
            // let doc = self.parse_html(&index_path)?;
            let body_selector =
                Selector::parse("body").map_err(|err| anyhow!("cannot create selector {}", err))?;
            let body = doc
                .select(&body_selector)
                .next()
                .ok_or(anyhow!("select target {:?} failed", body_selector))?;
            let mut html = body.html();

            // add data attributes to images
            let image_selector =
                Selector::parse("img").map_err(|err| anyhow!("cannot create selector {}", err))?;
            for img in body.select(&image_selector) {
                let old_img = img.value();
                let h = format!("{:?}", old_img);
                println!("Processing img tag {}", h);

                let mut new_img = old_img.clone();
                add_attr(&mut new_img, "data-template", "img");
                add_attr(&mut new_img, "data-title", "图片");

                let new_h = format!("{:?}", new_img);
                println!("Processed img tag {}", new_h);
                html = html.replace(&h, &new_h);
            }

            // add data attributes to texts
            body.children().for_each(|child| {
                if let Err(err) = self.traverse_node(&child, &mut html) {
                    eprintln!("Error parse DOM failed {err}");
                }
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
            println!("{} process done\n", file);
        }
        Ok(())
    }

    fn traverse_node(&self, target: &NodeRef<Node>, html: &mut String) -> Result<()> {
        let v = target.value();
        if v.is_text() {
            let is_vaild =
                self.vaild_text(&v.as_text().map(|t| t.to_string()).unwrap_or(String::new()));
            if is_vaild {
                let parent = target
                    .parent()
                    .ok_or(anyhow!("cannot find {:?} parent", target))?;
                let text = parent.value().as_element().ok_or(anyhow!("cannot parse element"))?;
                let old_h = format!("{:?}", text);
                println!("Processing text node {}", old_h);

                let mut new_text = text.clone();
                add_attr(&mut new_text, "data-template", "text");
                add_attr(&mut new_text, "data-title", "标题");

                let new_h = format!("{:?}", new_text);
                println!("Processed text node {}", new_h);
                *html = html.replace(&old_h, &new_h);
            }
        } else {
            for child in target.children() {
                self.traverse_node(&child, html)?
            }
        }
        Ok(())
    }

    /// Detect trget text is vaild text
    ///
    /// <div class="text-wrapper">人才发展</div>
    fn vaild_text(&self, text: &str) -> bool {
        let t = text.split('\n').collect::<Vec<_>>();
        let is_vaild = t.iter().all(|text| !text.trim().is_empty());
        // if is_vaild {
        //     println!("Process text node {}", t.join(''));
        // }
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

/// Build dom attributes
///
/// - `attr` attribute name
/// - `value` attirbute value
///
/// `data-template="img"`: `attr_builder("data-template", "img");`
fn attr_builder<'a>(attr: &'a str, value: &'a str) -> (QualName, Tendril<UTF8>) {
    let data = QualName {
        prefix: None,
        ns: Namespace::from(""),
        local: LocalName::from(attr),
    };
    let data_v: Tendril<UTF8> = Tendril::from(value);
    (data, data_v)
}

fn add_attr<'a>(element: &mut scraper::node::Element, attr: &'a str, value: &'a str) {
    let (data_img, data_v) = attr_builder(attr, value);
    element.attrs.insert(data_img, data_v);
}
