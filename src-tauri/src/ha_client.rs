use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntitySummary {
    pub entity_id: String,
    pub friendly_name: String,
    pub state: String,
    pub unit_of_measurement: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntityState {
    pub entity_id: String,
    pub friendly_name: String,
    pub state: String,
    pub unit_of_measurement: String,
}

#[derive(Deserialize, Debug)]
struct HaState {
    entity_id: String,
    state: String,
    attributes: HaAttributes,
}

#[derive(Deserialize, Debug)]
struct HaAttributes {
    #[serde(default)]
    friendly_name: Option<String>,
    #[serde(default)]
    unit_of_measurement: Option<String>,
}

fn build_client() -> Result<Client, String> {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())
}

pub async fn fetch_all_entities(url: &str, token: &str) -> Result<Vec<EntitySummary>, String> {
    let client = build_client()?;
    let response = client
        .get(format!("{}/api/states", url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let states: Vec<HaState> = response.json().await.map_err(|e| e.to_string())?;

    Ok(states
        .into_iter()
        .map(|s| EntitySummary {
            friendly_name: s
                .attributes
                .friendly_name
                .unwrap_or_else(|| s.entity_id.clone()),
            entity_id: s.entity_id,
            state: s.state,
            unit_of_measurement: s.attributes.unit_of_measurement.unwrap_or_default(),
        })
        .collect())
}

pub async fn fetch_selected(
    url: &str,
    token: &str,
    ids: &[String],
) -> Result<Vec<EntityState>, String> {
    let client = build_client()?;
    let base = url.trim_end_matches('/');
    let mut results = Vec::new();

    for id in ids {
        let response = client
            .get(format!("{}/api/states/{}", base, id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let s: HaState = response.json().await.map_err(|e| e.to_string())?;
            results.push(EntityState {
                friendly_name: s
                    .attributes
                    .friendly_name
                    .unwrap_or_else(|| s.entity_id.clone()),
                entity_id: s.entity_id,
                state: s.state,
                unit_of_measurement: s.attributes.unit_of_measurement.unwrap_or_default(),
            });
        }
    }

    Ok(results)
}
