use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;

use crate::{data::DeepData, error::{DeepError, Result}};

#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
}

impl DeepData {
    /// Fetch the latest release from pocamind/data
    pub async fn latest_release() -> Result<GithubRelease> {
        const OWNER: &str = "pocamind";
        const REPO: &str = "data";
        
        Self::latest_release_from(OWNER, REPO).await
    }

    /// Fetch the latest release from a fork
    pub async fn latest_release_from(
        owner: &str,
        repo: &str
    ) -> Result<GithubRelease> {
        let url = format!(
            "https://api.github.com/repos/{owner}/{repo}/releases/latest"
        );

        let client = reqwest::Client::new();

        let release = client
            .get(url)
            .header(USER_AGENT, "my-app/0.1")
            .header(ACCEPT, "application/vnd.github+json")
            .send()
            .await?
            .error_for_status()?
            .json::<GithubRelease>()
            .await?;

        Ok(release)
    }

    pub async fn from_release(release: &GithubRelease) -> Result<DeepData> {
        let asset = release.assets.iter().find(|asset| asset.name == "all.json");

        if let Some(asset) = asset {
            let client = reqwest::Client::new();

            let asset_url = &asset.browser_download_url;

            let content = client
                .get(asset_url)
                .header(USER_AGENT, "my-app/0.1")
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?;

            DeepData::from_json(&content)
        } else {
            Err(
                DeepError::FetchError(
                    format!("Failed to find 'all.json', found files [{}] instead.", release.assets.iter().map(|a| a.name.clone()).collect::<Vec<String>>().join(", "))
                )
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::DeepData;

    #[tokio::test]
    pub async fn fetch_data() {
        let release = DeepData::latest_release().await.unwrap();

        let _ = DeepData::from_release(&release).await.unwrap();
    }
}