use std::path::Path;

use anyhow::{anyhow, Context, Result};
use reqwest::blocking::{self, multipart};
use serde::{Deserialize, Serialize};

use crate::consts::BASE_URL;

#[derive(Debug)]
pub struct Http<'a> {
    // base_url: &'a str,
    token: &'a str,
    client: blocking::Client,
    upload_url: String,
}
impl<'a> Http<'a> {
    pub fn new(base_url: Option<&'a str>, token: &'a str) -> Self {
        let base_url = base_url.unwrap_or(BASE_URL);

        Self {
            // base_url,
            token,
            client: blocking::Client::new(),
            upload_url: format!("{}admin/Apitemplategrapic/add", base_url),
        }
    }
    pub fn upload(&self, path: &Path, name_suffix: &'a Option<String>) -> Result<()> {
        // let file = fs::read(path)?;
        let filename = &path
            .file_name()
            .ok_or(anyhow!("cannot read target filename"))?
            .to_string_lossy();

        let up_name = if filename.ends_with(".zip") {
            filename
                .strip_suffix(".zip")
                .ok_or(anyhow!("strip filename failed"))?
                .split('_')
                .collect::<Vec<_>>()
        } else {
            filename.split('_').collect::<Vec<_>>()
        };
        let upload_name = if let Some(name) = name_suffix.as_ref() {
            if up_name.len() < 3 {
                eprintln!("Wraning: filename is illegal {}", &filename);
                String::from(filename.clone())
            } else {
                format!("{}_{}", &name, &up_name[1])
            }
        } else {
            String::from(filename.clone())
        };

        let (width, height) = if upload_name.len() > 2 {
            let size = up_name
                .last()
                .ok_or(anyhow!(""))?
                .split('X')
                .collect::<Vec<_>>();

            if size.len() != 2 {
                ("", "")
            } else {
                (size[0], size[1])
            }
        } else {
            ("", "")
        };

        println!(
            "Starting upload {} as {} with width {} height {}",
            &filename, &upload_name, &width, &height
        );

        let form = multipart::Form::new()
            .text("name", upload_name.clone())
            .text("width", String::from(width))
            .text("height", String::from(height))
            .file("file", path)?;

        let res = self
            .client
            .post(&self.upload_url)
            .header("token", self.token)
            .multipart(form)
            .send()
            .with_context(|| anyhow!("send request to {} failed ", &self.upload_url))?
            .json::<ResBody>()
            .with_context(|| anyhow!("parse response failed"))?;

        if res.code == "200" {
            println!("Upload {} succeeded", &upload_name);
        } else {
            return Err(anyhow!("Upload {} failed, {:?}", &upload_name, &res));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResBody {
    code: String,
    data: String,
}
