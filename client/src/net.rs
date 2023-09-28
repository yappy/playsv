use anyhow::{anyhow, Result};
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

    pub fn request<F>(&mut self, url: &str, callback: F)
    where
        F: FnMut(Result<&str>) + 'static,
    {
        let request = XmlHttpRequest::new().unwrap();
        request.open("GET", url).unwrap();
        request.send().unwrap();

        let callback = Box::new(callback);

        self.reqs.push(ReqElem { request, callback });
    }
}
