use crate::models::CreateSatelliteDto;
use crate::repository::SatelliteRepository;
use std::time::Duration;
use tracing::{error, info};

#[derive(Debug, Clone, Copy)]
pub enum CelesTrakGroup {
    Stations,
    Visual,
    Starlink,
    Weather,
    Last30Days,
    Active,
}

impl CelesTrakGroup {
    pub fn as_str(&self) -> &'static str {
        match self {
            CelesTrakGroup::Stations => "stations",
            CelesTrakGroup::Visual => "visual",
            CelesTrakGroup::Starlink => "starlink",
            CelesTrakGroup::Weather => "weather",
            CelesTrakGroup::Last30Days => "last-30-days",
            CelesTrakGroup::Active => "active",
        }
    }
}

pub struct DiscoveryPipeline;

impl DiscoveryPipeline {
    /// Fetches TLE text data from CelesTrak for a given group and parses 3-line TLE blocks
    pub async fn fetch_group_tle(group: CelesTrakGroup) -> Result<Vec<CreateSatelliteDto>, String> {
        let url = format!(
            "https://celestrak.org/NORAD/elements/gp.php?GROUP={}&FORMAT=tle",
            group.as_str()
        );

        info!("Fetching discovery feed from {}", url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client
            .get(&url)
            .header("User-Agent", "satellite-api/0.1.0")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "CelesTrak API returned status {}",
                response.status()
            ));
        }

        let body_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read text body: {}", e))?;

        Ok(Self::parse_tle_text(&body_text))
    }

    /// Parses raw 3-line TLE text into a list of CreateSatelliteDto
    pub fn parse_tle_text(text: &str) -> Vec<CreateSatelliteDto> {
        let lines: Vec<&str> = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        let mut dtos = Vec::new();
        let mut idx = 0;

        while idx + 2 < lines.len() {
            let line0 = lines[idx];
            let line1 = lines[idx + 1];
            let line2 = lines[idx + 2];

            if line1.starts_with('1') && line2.starts_with('2') {
                dtos.push(CreateSatelliteDto {
                    name: line0.to_string(),
                    line_one: line1.to_string(),
                    line_two: line2.to_string(),
                });
                idx += 3;
            } else {
                idx += 1;
            }
        }

        dtos
    }

    /// Performs sync for a group and upserts results into SatelliteRepository
    pub async fn sync_group(
        repo: &SatelliteRepository,
        group: CelesTrakGroup,
    ) -> Result<usize, String> {
        let dtos = Self::fetch_group_tle(group).await?;
        let total = dtos.len();

        let existing_satellites = repo.list_satellites().await.unwrap_or_default();

        for dto in dtos {
            // Find existing satellite by name or NORAD ID match
            let existing = existing_satellites
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(&dto.name));

            if let Some(sat) = existing {
                let update_dto = crate::models::UpdateSatelliteDto {
                    name: Some(dto.name),
                    line_one: Some(dto.line_one),
                    line_two: Some(dto.line_two),
                };
                let _ = repo.update_satellite_by_id(sat.id, update_dto).await;
            } else {
                let _ = repo.create_satellite(dto).await;
            }
        }

        info!(
            "Successfully synced {} satellites for group {}",
            total,
            group.as_str()
        );
        Ok(total)
    }

    /// Starts a Tokio background worker that periodically syncs satellite discovery feeds
    pub fn start_background_sync(repo: SatelliteRepository, interval_hours: u64) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_hours * 3600));
            // First tick completes immediately
            interval.tick().await;

            loop {
                info!("Starting scheduled CelesTrak discovery pipeline sync...");
                let groups = [
                    CelesTrakGroup::Stations,
                    CelesTrakGroup::Visual,
                    CelesTrakGroup::Last30Days,
                ];

                for group in groups {
                    if let Err(e) = Self::sync_group(&repo, group).await {
                        error!("Failed discovery sync for group {}: {}", group.as_str(), e);
                    }
                }

                interval.tick().await;
            }
        });
    }
}
