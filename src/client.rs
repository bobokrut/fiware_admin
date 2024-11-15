use std::collections::HashMap;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde_json::json;
use serde_json::Value;
use tracing::debug;
use tracing::error;
use ureq::{Error, Response};

use crate::FiwareConfig;

pub struct MeasurementRequest {
    urn: String,
    name: String,
}

pub struct MeasurementResult {
    urn: String,
    name: String,
    value: String,
    timestamp: Option<DateTime<Utc>>,
}

pub struct Client {
    endpoint: String,
    token: String,
    service: Option<String>,
}

impl Client {
    pub fn new(config: FiwareConfig, service: Option<String>) -> Self {
        Self {
            endpoint: config.config.endpoint,
            token: config.config.token,
            service,
        }
    }
    fn get(&self, request: &String, params: &Vec<(String, String)>) -> Result<Response, Error> {
        let mut req = ureq::get(request).set("X-Auth-Token", &self.token);
        let params: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        if params.len() > 0 {
            req = req.query_pairs(params.to_owned());
        }
        if let Some(s) = &self.service {
            req = req.set("Fiware-Service", s);
        }
        req.call()
    }

    fn post(&self, request: &String, body: &Value) -> Result<Response, Error> {
        let mut req = ureq::post(request).set("X-Auth-Token", &self.token);
        if let Some(s) = &self.service {
            req = req.set("Fiware-Service", s);
        }
        req.send_json(body)
    }

    pub fn get_all_entities(
        &self,
        r#type: &Option<String>,
        attrs: &Option<Vec<String>>,
    ) -> Vec<Value> {
        let call_endpoint = format!("{}/entities", self.endpoint);
        let mut offset: i32 = 0;
        let mut params = vec![
            (
                String::from_str("limit").unwrap(),
                String::from_str("1000").unwrap(),
            ),
            (
                String::from_str("offset").unwrap(),
                String::from_str("0").unwrap(),
            ),
        ];
        let mut final_response = Vec::new();
        let mut response: Result<Response, Error>;
        let temp: String;

        if let Some(t) = r#type {
            params.push((String::from_str("type").unwrap(), t.to_string()));
        }

        if let Some(a) = attrs {
            temp = a.join(",");
            params.push((String::from_str("attrs").unwrap(), temp.to_string()));
        }

        loop {
            params[1].1 = {
                offset += 1000;
                response = self.get(&call_endpoint, &params);
                offset.to_string()
            };

            match response {
                Ok(resp) => {
                    let resp_json = resp
                        .into_json::<Value>()
                        .expect("Failed to parse result as JSON");

                    if let Value::Array(r) = resp_json {
                        if r.is_empty() {
                            break;
                        }
                        final_response.extend(r);
                    } else {
                        error!("Unexpected response format: {:?}", resp_json);
                        break;
                    }
                }
                Err(Error::Status(code, response)) => {
                    error!(
                        "Failed to get entities with code {}: {}",
                        code,
                        response
                            .into_string()
                            .unwrap_or_else(|_| "Invalid response body".to_string())
                    );
                    break;
                }
                Err(e) => {
                    error!("Failed to get entities: {}", e);
                    break;
                }
            }
        }
        final_response
    }

    pub fn delete_all_entities(&self, r#type: &Option<String>) -> Result<Response, Error> {
        let call_endpoint = format!("{}/op/update", self.endpoint);
        let result = self.get_all_entities(&r#type, &None);
        let mut ids: Vec<HashMap<String, String>> = Vec::new();

        for entity in result {
            let id = entity.get("id").unwrap().as_str().unwrap();
            let mut temp = HashMap::new();
            temp.insert("id".to_string(), id.to_string());
            ids.push(temp);
        }

        let payload = json!({
            "actionType": "delete",
            "entities": ids
        });

        self.post(&call_endpoint, &payload)
    }

    pub fn upload_entities(&self, entities: &Value, key_values: bool) -> Result<Response, Error> {
        let mut call_endpoint = format!("{}/op/update", self.endpoint);

        if key_values {
            call_endpoint.push_str("?options=keyValues");
        }

        let payload = json!({
            "actionType": "append_strict",
            "entities": entities
        });

        debug!(
            "Sending request to {} with payload: {}",
            call_endpoint, payload
        );

        self.post(&call_endpoint, &payload)
    }

    pub fn update_entities(&self, entities: &Value, key_values: bool) -> Result<Response, Error> {
        let mut call_endpoint = format!("{}/op/update", self.endpoint);
        if key_values {
            call_endpoint.push_str("?options=keyValues");
        }
        let payload = json!({
            "actionType": "update",
            "entities": entities
        });
        self.post(&call_endpoint, &payload)
    }

    pub fn query_entity(
        &self,
        measurement_request: &MeasurementRequest,
    ) -> Option<MeasurementResult> {
        let call_endpoint = format!("{}/entities/{}", self.endpoint, measurement_request.urn);

        let response = self.get(&call_endpoint, &Vec::new());

        match response {
            Ok(resp) => {
                let resp_json = resp
                    .into_json::<Value>()
                    .expect("Failed to parse result as JSON");
                let mut ts: Option<DateTime<Utc>> = None;

                if let Some(v) = resp_json.get("TimeInstant") {
                    let ts_str = v.get("value").unwrap().as_str().unwrap();
                    ts = DateTime::parse_from_rfc3339(ts_str)
                        .ok()
                        .map(|x| x.with_timezone(&Utc));
                }

                Some(MeasurementResult {
                    urn: measurement_request.urn.clone(),
                    name: measurement_request.name.clone(),
                    value: resp_json
                        .get(&measurement_request.name)
                        .unwrap()
                        .get("value")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    timestamp: ts,
                })
            }
            Err(Error::Status(code, response)) => {
                error!(
                    "Failed to get entities with code {}: {}",
                    code,
                    response.into_string().unwrap()
                );
                return None;
            }
            Err(e) => {
                error!("Failed to get entities: {}", e);
                return None;
            }
        }
    }
}
