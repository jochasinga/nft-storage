pub mod client;
mod platform;
pub use client::*;
pub use platform::*;

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{self as jwt};
    use anyhow::Result;
    use tokio::fs::File;
    use url::Url;

    #[tokio::test]
    async fn construct_client() -> Result<()> {
        let token = jwt::encode(
            &jwt::Header::new(jwt::Algorithm::HS256),
            &JWTClaim::default(),
            &jwt::EncodingKey::from_secret("".as_bytes())
        )?;

        let storage_client = NFTStorage::new(&token, Some(url::Url::from_file_path("/tmp/foo.txt").unwrap()))?;
        assert_eq!(storage_client.endpoint, Url::from_file_path("/tmp/foo.txt").unwrap());
        assert_eq!(storage_client.token, token);

        let image = File::open("test.txt").await?;
        let metadata = Metadata {
            name: "hello",
            description: "bar",
            image,
            url: None,
        };

        let cid = storage_client.store(metadata).await?;
        assert!(cid.starts_with("bafkrei"));

        Ok(())
    }
}
