/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::Experiment;
use anyhow::Result;
use serde_derive::*;
use url::Url;
use viaduct::{status_codes, Request, Response};

// Making this a trait so that we can mock those later.
pub(crate) trait SettingsClient {
    fn get_experiements_metadata(&self) -> Result<String>;
    fn get_experiments(&self) -> Result<Vec<Experiment>>;
}

#[derive(Deserialize)]
struct RecordsResponse {
    data: Vec<Experiment>,
}

pub struct Client {
    base_url: Url,
    collection_name: String,
    bucket_name: String,
}

impl Client {
    pub fn new(base_url: Url, collection_name: String, bucket_name: String) -> Self {
        Self {
            base_url,
            collection_name,
            bucket_name,
        }
    }

    fn make_request(&self, request: Request) -> Result<Response> {
        let resp = request.send()?;
        if resp.is_success() || resp.status == status_codes::NOT_MODIFIED {
            Ok(resp)
        } else {
            anyhow::bail!("Error in request: {}", resp.text())
        }
    }
}

impl SettingsClient for Client {
    fn get_experiements_metadata(&self) -> Result<String> {
        let path = format!(
            "buckets/{}/collections/{}",
            &self.bucket_name, &self.collection_name
        );
        let url = self.base_url.join(&path)?;
        let req = Request::get(url).header(
            "User-Agent",
            "Experiments Rust Component <teshaq@mozilla.com>",
        )?;
        let resp = self.make_request(req)?;
        let res = serde_json::to_string(&resp.body)?;
        Ok(res)
    }

    fn get_experiments(&self) -> Result<Vec<Experiment>> {
        let path = format!(
            "buckets/{}/collections/{}/records",
            &self.bucket_name, &self.collection_name
        );
        let url = self.base_url.join(&path)?;
        let req = Request::get(url)
            .header(
                "User-Agent",
                "Experiments Rust Component <teshaq@mozilla.com>",
            )?;
            // Note: I removed the auth info which was for a test account that is public
            // But gitgaurdian complained so I removed it.
        let resp = self.make_request(req)?.json::<RecordsResponse>()?;
        Ok(resp.data)
    }
}
