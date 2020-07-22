/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This is a simple Http client that uses viaduct to retrieve experiment data from the server
//! Currently configured to use Kinto and the old schema, although that would change once we start
//! Working on the real Nimbus schema.

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
        let req = Request::get(url).header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:77.0) Gecko/20100101 Firefox/77.0",
        )?;
        // TODO: Add authentication based on server requirements
        let resp = self.make_request(req)?.json::<RecordsResponse>()?;
        Ok(resp.data)
    }
}
