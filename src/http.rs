use std::path::Path;

use anyhow::Result;
use reqwest::blocking::{self, multipart};
use serde::{Deserialize, Serialize};

use crate::consts::BASE_URL;

#[derive(Debug)]
pub struct Http<'a> {
    base_url: &'a str,
    token: &'a str,
    client: blocking::Client,
}
impl<'a> Http<'a> {
    pub fn new(base_url: Option<&'a str>, token: &'a str) -> Self {
        let base_url = base_url.unwrap_or(BASE_URL);

        Self {
            base_url,
            token,
            client: blocking::Client::new(),
        }
    }
    pub fn upload(&self, path: &Path) -> Result<()> {
        // let file = fs::read(path)?;
        let form = multipart::Form::new()
            .text("name", "")
            .text("width", "")
            .text("height", "")
            .file("file", path)?;

        let res = self
            .client
            .post(format!("{}admin/Apitemplategrapic/add", self.base_url))
            .header("token", self.token)
            .multipart(form)
            .send()?
            .json::<ResBody>();

        println!("{:?}", res);

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResBody {
    code: String,
    data: String,
}
