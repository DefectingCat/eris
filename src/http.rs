use reqwest::Client;

use crate::consts::BASE_URL;

#[derive(Debug)]
pub struct Http<'a> {
    base_url: &'a str,
    token: &'a str,
    client: Client,
}
impl<'a> Http<'a> {
    pub fn new(base_url: Option<&'a str>, token: &'a str) -> Self {
        let base_url = base_url.unwrap_or(BASE_URL);

        Self {
            base_url,
            token,
            client: Client::new(),
        }
    }
}
