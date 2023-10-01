use anyhow::{anyhow, Result};
use serde::Serialize;
use web_sys::XmlHttpRequest;

pub type Handler = Box<dyn FnMut(Result<&str>)>;

#[allow(dead_code)]
const STATE_UNSENT: u16 = 0;
#[allow(dead_code)]
const STATE_OPENED: u16 = 1;
#[allow(dead_code)]
const STATE_HEADERS_RECEIVED: u16 = 2;
#[allow(dead_code)]
const STATE_LOADING: u16 = 3;
#[allow(dead_code)]
const STATE_DONE: u16 = 4;

struct ReqElem {
    request: XmlHttpRequest,
    callback: Handler,
}

pub struct PollingHttp {
    reqs: Vec<ReqElem>,
}

impl PollingHttp {
    pub fn new() -> Self {
        Self {
            reqs: Default::default(),
        }
    }

    pub fn rest(&self) -> usize {
        self.reqs.len()
    }

    pub fn poll(&mut self) -> usize {
        self.reqs.retain_mut(|elem| {
            let req = &elem.request;
            if req.ready_state() == STATE_DONE {
                // 0 if error
                let status = req.status().unwrap();
                // null if error
                let text = req.response_text().unwrap();

                if (200..300).contains(&status) {
                    (elem.callback)(Ok(&text.unwrap()));
                } else {
                    let err = anyhow!(
                        "HTTP Error, status: {status}, text: {}",
                        text.unwrap_or_default()
                    );
                    (elem.callback)(Err(err));
                }

                false
            } else {
                true
            }
        });

        self.rest()
    }

    pub fn request<F>(&mut self, url: &str, method: &str, data: &str, callback: F)
    where
        F: FnMut(Result<&str>) + 'static,
    {
        let request = XmlHttpRequest::new().unwrap();
        request.open(method, url).unwrap();
        if data.is_empty() {
            request.send().unwrap();
        } else {
            request
                .set_request_header("Content-Type", "application/json")
                .unwrap();
            request.send_with_opt_str(Some(data)).unwrap();
        }

        let callback = Box::new(callback);

        self.reqs.push(ReqElem { request, callback });
    }

    pub fn get<F>(&mut self, url: &str, callback: F)
    where
        F: FnMut(Result<&str>) + 'static,
    {
        self.request(url, "GET", "", callback)
    }

    pub fn post<T, F>(&mut self, url: &str, data: &T, callback: F)
    where
        T: Serialize,
        F: FnMut(Result<&str>) + 'static,
    {
        let data = serde_json::to_string(data).unwrap();
        self.request(url, "POST", &data, callback)
    }
}
