use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use std::fs::OpenOptions;

const PIPE_NAME: &str = r"\\.\pipe\NephNodeSide";

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    id: String,
    method: &'a str,
    params: Value,
}

#[derive(Deserialize)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<Value>,
}

pub struct NodesideClient {
    #[cfg(windows)]
    pipe: Arc<Mutex<std::fs::File>>,
}

impl NodesideClient {
    pub fn connect() -> Result<Self> {
        #[cfg(windows)]
        {
            let mut last = anyhow!("pipe unavailable");
            for _ in 0..3 {
                match OpenOptions::new().read(true).write(true).open(PIPE_NAME) {
                    Ok(f) => return Ok(Self { pipe: Arc::new(Mutex::new(f)) }),
                    Err(e) => {
                        last = e.into();
                        std::thread::sleep(std::time::Duration::from_millis(150));
                    }
                }
            }
            Err(last)
        }
        #[cfg(not(windows))]
        {
            anyhow::bail!("NodesideClient is Windows-only")
        }
    }

    fn call(&self, method: &str, params: Value) -> Result<Value> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: format!("{}", rand::random::<u64>()),
            method,
            params,
        };
        let line = format!("{}\n", serde_json::to_string(&req)?);
        #[cfg(windows)]
        {
            let mut pipe = self.pipe.lock().map_err(|_| anyhow!("pipe mutex poisoned"))?;
            pipe.write_all(line.as_bytes())?;
            pipe.flush()?;
            let mut reader = BufReader::new(&*pipe);
            let mut resp_line = String::new();
            reader.read_line(&mut resp_line)?;
            let resp: RpcResponse = serde_json::from_str(&resp_line)?;
            if let Some(err) = resp.error {
                anyhow::bail!("nodeside error: {}", err);
            }
            resp.result.ok_or_else(|| anyhow!("empty nodeside result"))
        }
        #[cfg(not(windows))]
        {
            anyhow::bail!("not on Windows")
        }
    }

    pub fn browser_read_page(&self, profile: &str, url: &str) -> Result<(String, String)> {
        let v = self.call(
            "browser.read_page",
            serde_json::json!({ "profile": profile, "url": url }),
        )?;
        let title = v.get("title").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        let text = v.get("text").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        Ok((title, text))
    }

    pub fn browser_search(&self, profile: &str, query: &str) -> Result<(String, String)> {
        let v = self.call(
            "browser.search",
            serde_json::json!({ "profile": profile, "query": query }),
        )?;
        let title = v.get("title").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        let text = v.get("text").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        Ok((title, text))
    }

    pub fn browser_click(&self, profile: &str, url: &str, selector: &str) -> Result<(String, String)> {
        let v = self.call(
            "browser.click",
            serde_json::json!({ "profile": profile, "url": url, "selector": selector }),
        )?;
        let title = v.get("title").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        let text = v.get("text").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        Ok((title, text))
    }

    pub fn browser_fill_form(
        &self,
        profile: &str,
        url: &str,
        fields: &serde_json::Value,
        submit_selector: Option<&str>,
    ) -> Result<(String, String)> {
        let v = self.call(
            "browser.fill_form",
            serde_json::json!({
                "profile": profile,
                "url": url,
                "fields": fields,
                "submit_selector": submit_selector
            }),
        )?;
        let title = v.get("title").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        let text = v.get("text").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        Ok((title, text))
    }
}

