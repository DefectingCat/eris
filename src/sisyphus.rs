use anyhow::{anyhow, Context, Result};
use ego_tree::NodeRef;
use html5ever::{
    tendril::{fmt::UTF8, Tendril},
    LocalName, Namespace, QualName,
};

use scraper::{Html, Node, Selector};
use std::{
    fs::{self, DirEntry, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::{
    args::Mode,
    consts::{METHOD_STORED, RESET_CSS},
    errors::{ErisError, ErisResult},
    ziper::Ziper,
};

#[derive(Debug)]
pub struct Sisyphus {
    /// Target directory
    pub directory: PathBuf,
    /// Output directory
    pub output: PathBuf,
    /// Format all files or compress formated files
    mode: Mode,
    // Target zip files
    file_list: Vec<PathBuf>,
    // Ziper
    ziper: Ziper,
}

impl Sisyphus {
    /// Sisyphus builder.
    ///
    /// Create new Sisyphus struct.
    ///
    /// - `directory`: target diretory. Sisyphus will read all zip file in this directory.
    /// - `output`: compress file output directory.
    pub fn new(mode: Mode, directory: &PathBuf, output: &Option<PathBuf>) -> Result<Self> {
        let target_paths = fs::read_dir(directory)?;
        let file_list = target_paths.fold(vec![], |mut prev, path| {
            let target = match path {
                Ok(p) => match format_path(p, mode == Mode::Format) {
                    Ok(path) => path,
                    Err(err) => match err {
                        ErisError::Empty(_) => return prev,
                        ErisError::Other(err) => {
                            eprintln!("{}", err);
                            return prev;
                        }
                    },
                },
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
            println!("Found {} file(s): ", len);
            file_list.iter().for_each(|f| println!("{:?}", f));
        }
        println!();

        let directory = PathBuf::from(directory);
        let output = if let Some(o) = output {
            PathBuf::from(o)
        } else {
            let mut p = PathBuf::from(&directory);
            p.push("output");
            p
        };

        let s = Self {
            directory,
            output,
            mode,
            file_list,
            ziper: Ziper::new(),
        };
        Ok(s)
    }

    /// Unzip target
    ///
    /// - `path` unzip to folder
    fn unzip(&self, path: &Path) -> Result<()> {
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("convert filename failed")),
        };
        // create same name folder
        let name = format_name(&file_name);
        let mut dir_path = PathBuf::from(&self.directory);
        dir_path.push(name);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
        }
        println!("String unzip {:?}", path);
        let dir_path = dir_path.to_string_lossy();
        let ziper = &self.ziper;
        ziper.unzip(Some(&dir_path), path)?;
        Ok(())
    }

    /// Unzip all target zip files, and format target templates.
    fn format_process(&self) -> Result<()> {
        for file in &self.file_list {
            self.unzip(file)?;

            let file = file.to_string_lossy();
            // remove `.zip` str
            let folder_prefix = &file[..file.len() - 4];
            let index_path = {
                let mut p = PathBuf::from(&folder_prefix);
                p.push("index.html");
                p
            };
            let mut index_file = File::options()
                .read(true)
                .write(true)
                // .append(true)
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
            for child in body.children() {
                traverse_node(&child, &mut html)
                    .map_err(|err| anyhow!("Error parse DOM failed {err}"))?;
            }

            let style_path = {
                let mut p = PathBuf::from(folder_prefix);
                p.push("style.css");
                p
            };
            let style_file = fs::read_to_string(style_path)?;
            let styles = format!("<style>\n{}\n{}\n</style>", style_file, RESET_CSS);
            let html = format!("{}\n{}", styles, html);
            // index_file
            //     .set_len(0)
            //     .with_context(|| anyhow!("cannot clean {:?} file", &index_file))?;
            // rewrite body tag to html file
            // index_file
            //     .write_all(html.as_bytes())
            //     .with_context(|| anyhow!("cannot write to file {:?}", &index_file))?;

            // create new template.html
            let mut new_name = PathBuf::from(&index_path);
            new_name.set_file_name("template.html");
            if new_name.exists() {
                fs::remove_file(&new_name)?;
            }
            let mut template = File::options().write(true).create(true).open(&new_name)?;
            template
                .write_all(html.as_bytes())
                .with_context(|| anyhow!("cannot write to file {:?}", &template))?;
            // delete index.html
            fs::remove_file(&index_path)?;
            // fs::rename(&index_path, &new_name)
            //     .with_context(|| anyhow!("cannot rename {:?} to {:?}", index_path, new_name))?;
            println!("{} process done\n", file);
        }
        Ok(())
    }

    /// Traverse all formated directories, compress to zip files.
    fn compress_process(&self) -> Result<()> {
        if !&self.output.exists() {
            fs::create_dir_all(&self.output)?;
        } else {
            fs::remove_dir_all(&self.output)?;
            fs::create_dir_all(&self.output)?;
        }

        for path in &self.file_list {
            let mut out_path = PathBuf::from(&self.output);
            let path_name = path
                .iter()
                .last()
                .ok_or(anyhow!("cannot get folder filename"))?
                .to_string_lossy();
            let filename = format!("{}.zip", &path_name);
            out_path.push(&filename);
            println!("Starting zip {:?}", out_path);

            let file = File::options()
                .write(true)
                .read(true)
                .create_new(true)
                .open(&out_path)
                .with_context(|| anyhow!("open target {:?} failed", out_path))?;

            let mut src_path = PathBuf::from(&self.directory);
            src_path.push(&*path_name);
            let walkdir = WalkDir::new(&src_path);
            let it = walkdir.into_iter();

            let ziper = &self.ziper;
            ziper.zip_dir(
                &mut it.filter_map(|e| e.ok()),
                &src_path,
                file,
                METHOD_STORED.ok_or(anyhow!("cannot use stored compression method"))?,
            )?;
            println!("{} compress done\n", filename);
        }

        Ok(())
    }

    pub fn process(&self) -> Result<()> {
        match self.mode {
            Mode::Format => self.format_process()?,
            Mode::Compress => self.compress_process()?,
        }
        Ok(())
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

/// Add attributes to target element
///
/// - `element` target element
/// - `attr` attribute
/// - `value` attribute value
///
/// `<div data-template="img"></div>`: `add_attr(div_element, "data-template", "img");`
fn add_attr<'a>(element: &mut scraper::node::Element, attr: &'a str, value: &'a str) {
    let (data_img, data_v) = attr_builder(attr, value);
    element.attrs.insert(data_img, data_v);
}

/// Detect trget text is vaild text
///
/// <div class="text-wrapper">人才发展</div>
fn vaild_text(text: &str) -> bool {
    let t = text.split('\n').collect::<Vec<_>>();
    let is_vaild = t.iter().any(|text| !text.trim().is_empty());
    // if is_vaild {
    //     println!("Process text node {}", t.join(''));
    // }
    is_vaild
}

/// Format filename with extention
///
/// - `file_name` target file name, such as `test.zip`
fn format_name(file_name: &str) -> &str {
    let name = file_name.split('.');
    let name = name.collect::<Vec<_>>();
    let name = name.first();
    if let Some(n) = name {
        n
    } else {
        ""
    }
}

/// recursion target node to find node that's contain target text.
/// And add attributes
///
/// - `target`: target node
/// - `html`: whole mutable html string
fn traverse_node(target: &NodeRef<Node>, html: &mut String) -> Result<()> {
    let v = target.value();
    if v.is_text() {
        let is_vaild = vaild_text(&v.as_text().map(|t| t.to_string()).unwrap_or(String::new()));
        if is_vaild {
            let parent = target
                .parent()
                .ok_or(anyhow!("cannot find {:?} parent", target))?;
            let text = parent
                .value()
                .as_element()
                .ok_or(anyhow!("cannot parse element"))?;
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
            traverse_node(&child, html)?
        }
    }
    Ok(())
}

/// Format path in user input directory.
///
/// When `use_file = false` will not return folder that's name equal to `output`.
///
/// - `path` target path in user input directory
/// - `use_file` if true, will only format zip files, otherwise only format folders.
fn format_path(path: DirEntry, use_file: bool) -> ErisResult<PathBuf> {
    let file_name = path.file_name();
    let file_name = file_name.to_string_lossy();
    let file_type = path
        .file_type()
        .map_err(|err| anyhow!("Error: read file {} file type failed {}", file_name, err))?;

    if use_file && file_type.is_file() && file_name.ends_with(".zip") {
        Ok(path.path())
    } else if !use_file && file_type.is_dir() {
        let path = path.path();
        let dir_name = path
            .iter()
            .last()
            .ok_or(anyhow!("Error: cannot read folder name"))?;
        if dir_name == "output" {
            Err(ErisError::Empty(String::new()))
        } else {
            Ok(path)
        }
    } else {
        Err(ErisError::Empty(String::new()))
    }
}
