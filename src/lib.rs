use std::collections::HashMap;
use std::fmt::format;
use regex::Regex;
use reqwest::redirect::Policy;

#[derive(Debug, Clone)]
struct Csrf {
    name: String,
    value: String
}

#[derive(Debug, Clone)]
pub(crate) struct AioClient {
    client: reqwest::Client,
    csrf: Option<Csrf>,
    url: String,
    port: i16,
    token: Option<String>
}

async fn extract_csrf(body: &str) -> Result<Csrf, String> {
    // let response = client.get(format!("{}/login", AIO_URL))
    //     .send()
    //     .await
    //     .map_err(|e| e.to_string())?
    //     .text()
    //     .await
    //     .map_err(|e| e.to_string())?;
    let csrf_re = Regex::new(
        r"(?s)<input\s+type=.hidden.\s+name=.csrf_name.\s+value=.(?<csrf_name>[a-z0-9]+).*>.*<input\s+type=.hidden.\s+name=.csrf_value.\s+value=.(?<csrf_value>[a-z0-9A-Z=/]+).*>"
    ).unwrap();

    let Some(captures) = csrf_re.captures(body) else {
        return Err("Did not find csrf values in page".into());
    };
    let (_, [csrf_name, csrf_value]) = captures.extract();
    Ok(Csrf {
        name: csrf_name.into(),
        value: csrf_value.into(),
    })
}

impl AioClient {
    pub(crate) fn new(url: String, port: i16, allow_unsafe_tls: bool) -> Result<AioClient, String> {
        let cookie_store = {
            let file = std::fs::File::open("cookies.json")
                .map(std::io::BufReader::new)
                .unwrap();
            // use re-exported version of `CookieStore` for crate compatibility
            reqwest_cookie_store::CookieStore::load_json(file)
                .map_err(|e| e.to_string())?
        };
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(allow_unsafe_tls)
            .redirect(Policy::none())
            .cookie_store(true)
            .cookie_provider(cookie_store)
            .build()
            .map_err(|e| e.to_string())?;

        Ok(AioClient{
            client,
            url,
            port,
            csrf: None,
            token: None,
        })
    }

    fn build_url(&self) -> String {
        format!("{}:{}", self.url, self.port)
    }

    pub(crate) async fn login(&mut self) -> Result<(), String> {
        let token = match &self.token {
            None => self.fetch_token().await?,
            Some(token) => token.clone()
        };
        match self.client
            .get(format!("{}/api/auth/getlogin?token={}", self.build_url(), token))
            .send()
            .await
        {
            Ok(resp) => match resp.status().as_u16() {
                302 => {
                    let body = self.client.get(format!("{}/containers", self.build_url()))
                        .send()
                        .await
                        .map_err(|e| e.to_string())?
                        .text()
                        .await
                        .map_err(|e| e.to_string())?;
                    self.csrf = extract_csrf(&body).await.ok();
                    match self.csrf {
                        None => Err("Failed to extract csrf from body".into()),
                        Some(_) => Ok(())
                    }
                },
                status => {
                    Err(format!("Unexpected status: {}", status))
                }
            }
            Err(msg) => {
                Err(msg.to_string())
            }
        }
    }


    async fn fetch_token(&mut self) -> Result<String, String> {
        let response: serde_json::Value = self.client.get(format!("{}/ncp.php", self.build_url()))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        self.token = response["token"]
            .as_str()
            .map(|s| s.into());

        self.token.clone().ok_or("failed to parse token from json response".into())
    }


    pub(crate) async fn request(&self, path: &str, body: Option<String>) -> Result<reqwest::Response, String> {
        let Some(csrf) = self.csrf.clone() else {
            return Err("No csrf available - not logged in?".into())
        };
        match body {
            None => {
                let params = [
                    ("csrf_name", csrf.name),
                    ("csrf_value", csrf.value)
                ];
                self.client.get(format!("{}/{}", self.build_url(), path))
                    .form(&params)
                    .send()
                    .await
            }
            Some(body) => {
                let mut json = HashMap::new();
                json.insert("csrf_name", csrf.name);
                json.insert("csrf_value", csrf.value);
                self.client.post(format!("{}/{}", self.build_url(), path))
                    .json(&json)
                    .send()
                    .await
            }
        }.map_err(|e| e.to_string())
    }
}
