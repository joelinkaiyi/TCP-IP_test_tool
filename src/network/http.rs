use eframe::egui;
use reqwest::Client;
use tokio::runtime::Runtime;
use std::time::Instant;

#[derive(Default)]
pub struct HttpTester {
    url: String,
    method: HttpMethod,
    request_body: String,
    headers: Vec<(String, String)>,
    response: String,
    runtime: Option<Runtime>,
}

#[derive(Default, PartialEq, Clone)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
}

impl HttpTester {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if self.runtime.is_none() {
            self.runtime = Runtime::new().ok();
        }

        ui.horizontal(|ui| {
            ui.label("URL:");
            ui.text_edit_singleline(&mut self.url);
        });

        ui.horizontal(|ui| {
            ui.label("Method:");
            ui.selectable_value(&mut self.method, HttpMethod::GET, "GET");
            ui.selectable_value(&mut self.method, HttpMethod::POST, "POST");
            ui.selectable_value(&mut self.method, HttpMethod::PUT, "PUT");
            ui.selectable_value(&mut self.method, HttpMethod::DELETE, "DELETE");
        });

        // Headers 編輯區
        ui.collapsing("Headers", |ui| {
            if ui.button("Add Header").clicked() {
                self.headers.push(("".to_string(), "".to_string()));
            }
            
            let mut remove_idx = None;
            for (idx, (key, value)) in self.headers.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(key);
                    ui.text_edit_singleline(value);
                    if ui.button("✖").clicked() {
                        remove_idx = Some(idx);
                    }
                });
            }
            
            if let Some(idx) = remove_idx {
                self.headers.remove(idx);
            }
        });

        if self.method != HttpMethod::GET {
            ui.label("Request Body:");
            ui.text_edit_multiline(&mut self.request_body);
        }

        if ui.button("Send Request").clicked() {
            self.send_request();
        }

        if !self.response.is_empty() {
            ui.group(|ui| {
                ui.label("Response:");
                ui.label(&self.response);
            });
        }
    }

    fn send_request(&mut self) {
        if let Some(runtime) = &self.runtime {
            let url = self.url.clone();
            let method = self.method.clone();
            let body = self.request_body.clone();
            let headers = self.headers.clone();

            runtime.block_on(async {
                let client = Client::new();
                let start_time = Instant::now();
                
                let mut request = match method {
                    HttpMethod::GET => client.get(&url),
                    HttpMethod::POST => client.post(&url),
                    HttpMethod::PUT => client.put(&url),
                    HttpMethod::DELETE => client.delete(&url),
                };

                // 添加 headers
                for (key, value) in headers {
                    if !key.is_empty() && !value.is_empty() {
                        request = request.header(&key, &value);
                    }
                }

                // 添加請求內容
                if method != HttpMethod::GET && !body.is_empty() {
                    request = request.body(body);
                }

                match request.send().await {
                    Ok(response) => {
                        let status = response.status();
                        let elapsed = start_time.elapsed();
                        
                        match response.text().await {
                            Ok(text) => {
                                self.response = format!(
                                    "Status Code: {}\nTime: {:.2}ms\n\n{}",
                                    status,
                                    elapsed.as_secs_f64() * 1000.0,
                                    text
                                );
                            }
                            Err(e) => {
                                self.response = format!("Failed to read response content: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.response = format!("Request failed: {}", e);
                    }
                }
            });
        } else {
            self.response = "Failed to initialize runtime".to_string();
        }
    }
} 