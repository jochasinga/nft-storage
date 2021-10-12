use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use reqwest::{header, Body};
use anyhow::Result;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

pub(crate) const STORAGE_URL: &str = "https://api.nft.storage/";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct JWTClaim {
   pub(crate) sub: String,
   pub(crate) iss: String,
   pub(crate) iat: i64,
   pub(crate) name: String,
}

#[derive(Debug)]
pub struct Metadata<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub image: File,
    pub url: Option<url::Url>,
}

fn file_to_body(file: File) -> Body {
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);
    body
}


impl Default for JWTClaim {
   fn default() -> Self {
      Self {
         sub: "did:ethr:0x39c221E391b43034847677f2eFe7584F536f389a".to_string(),
         iss: "nft-storage".to_string(),
         iat: 1629229027112,
         name: "My NFT App".to_string(),
      }
   }
}

#[derive(Clone, Debug)]
pub struct NFTStorage<'a> {
    pub token: &'a str,
    pub endpoint: url::Url,
    pub(crate) http_client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    ok: bool,
    value: serde_json::value::Value
}

impl<'a> NFTStorage<'a> {
    pub fn new(token: &'a str, endpoint: Option<url::Url>) -> Result<Self, url::ParseError> {
        let url: url::Url;
        if let Some(u) = endpoint {
            url = u;
        } else {
            url = url::Url::parse(STORAGE_URL)?;
        }

        Ok(Self {
            token,
            endpoint: url,
            http_client: reqwest::Client::new(),
        })
    }

    async fn post<T: Into<Body>>(
        &self,
        path: &str,
        body: T,
        headers: header::HeaderMap,
    ) -> Result<UploadResponse> {
        let url = self.endpoint.join(path)?;
        let resp = self.http_client
            .post(url)
            .bearer_auth(self.token)
            .body(body)
            .headers(headers)
            .send()
            .await?
            .json::<UploadResponse>()
            .await?;

        Ok(resp)
    }

    pub async fn store(&self, metadata: Metadata<'a>) -> Result<String> {
        // FIXME: Escape hatch for testing
        if self.endpoint.scheme() == "file" {
            return Ok("bafkrei".to_string());
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/octet-stream")
        );

        let resp = self.post(
            "/upload",
            file_to_body(metadata.image),
            headers
        ).await?;

        let mut cid: Option<String> = None;
        if let UploadResponse { value: Value::Object(m), .. } = resp {
            if let Some(Value::String(s)) = m.get("cid") {
                cid.replace(s.to_string()).unwrap();
            }
        }

        Ok(cid.unwrap())
    }
}