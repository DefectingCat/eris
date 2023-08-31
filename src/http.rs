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
    pub fn upload(&self, path: &Path) -> Result<()> {
        // let file = fs::read(path)?;
        let filename = &path
            .file_name()
            .ok_or(anyhow!("cannot read target filename"))?
            .to_string_lossy();
        println!("Starting upload {}", &filename);

        let form = multipart::Form::new()
            .text("name", String::from(filename.clone()))
            .text("width", "")
            .text("height", "")
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
            println!("Upload {} sucess", &filename);
        } else {
            return Err(anyhow!("Upload {} failed, {:?}", &filename, &res));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResBody {
    code: String,
    data: String,
}
