/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Experiments library that hopes to be cross-plateform.
//! Still a work in progress, but good enough for people to poke around

mod buckets;
pub mod error;
pub mod ffi;
mod http_client;
mod persistence;

use error::Result;
pub use ffi::{experiements_destroy, experiments_get_branch, experiments_new};
use http_client::{Client, SettingsClient};
use persistence::Database;
use persistence::PersistedData;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde_derive::*;
use std::convert::TryInto;
use std::path::Path;
use url::Url;
use uuid::Uuid;
pub use viaduct;

const BASE_URL: &str = "https://kinto.dev.mozaws.net/v1/";
const COLLECTION_NAME: &str = "messaging-collection";
const BUCKET_NAME: &str = "main";
const MAX_BUCKET_NO: u32 = 10000;

// We'll probably end up doing what is done in A-S with regards to
// protobufs if we take this route...
// But for now, using the build.rs seems convenient
// ref: https://github.com/mozilla/application-services/tree/main/tools/protobuf-gen
pub mod msg_types {
    include!(concat!(
        env!("OUT_DIR"),
        "/mozilla.telemetry.glean.protobuf.rs"
    ));
}

/// Experiments is the main struct representing the experiements state
/// It should hold all the information needed to communcate a specific user's
/// Experiementation status (note: This should have some type of uuid)
pub struct Experiments {
    // Uuid not used yet, but we'll be using it later
    #[allow(unused)]
    uuid: Uuid,
    #[allow(unused)]
    app_ctx: AppContext,
    experiments: Vec<Experiment>,
    enrolled_experiments: Vec<EnrolledExperiment>,
    bucket_no: u32,
}

impl Experiments {
    /// A new experiments struct is created this is where some preprocessing happens
    /// It should look for persisted state first and setup some type
    /// Of interval retrieval from the server for any experiment updates (not implemented)
    pub fn new<P: AsRef<Path>>(app_ctx: AppContext, path: P) -> Self {
        let database = Database::new(path).unwrap();
        let persisted_data = database.get("persisted").unwrap();
        if let Some(data) = persisted_data {
            log::info!("Retrieving data from persisted state...");
            let persisted_data = serde_json::from_str::<PersistedData>(&data).unwrap();
            return Self {
                app_ctx,
                uuid: persisted_data.uuid,
                experiments: persisted_data.experiments,
                enrolled_experiments: persisted_data.enrolled_experiments,
                bucket_no: persisted_data.bucket_no,
            };
        }
        let http_client = Client::new(
            Url::parse(BASE_URL).unwrap(),
            COLLECTION_NAME.to_string(),
            BUCKET_NAME.to_string(),
        );
        let resp = http_client.get_experiments().unwrap();

        let uuid = uuid::Uuid::new_v4();
        let bucket_no: u32 =
            u32::from_be_bytes(uuid.as_bytes()[..4].try_into().unwrap()) % MAX_BUCKET_NO;
        let mut num = StdRng::seed_from_u64(bucket_no as u64);
        let enrolled_experiments = resp
            .iter()
            .filter_map(|e| {
                let branch = num.gen::<usize>() % e.branches.len();
                if bucket_no > e.buckets.count && bucket_no < e.buckets.start {
                    Some(EnrolledExperiment {
                        id: e.id.clone(),
                        branch: e.branches[branch].name.clone(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<EnrolledExperiment>>();
        database
            .put(
                "persisted",
                PersistedData {
                    app_ctx: app_ctx.clone(),
                    uuid,
                    bucket_no,
                    enrolled_experiments: enrolled_experiments.clone(),
                    experiments: resp.clone(),
                },
            )
            .unwrap();
        Self {
            app_ctx,
            uuid,
            experiments: resp,
            bucket_no,
            enrolled_experiments,
        }
    }

    /// Retrieves the branch the user is in, in the experiment. Errors if the user is not enrolled (This should be an option, but for ffi + test it errors)
    pub fn get_experiment_branch(&self, exp_name: &str) -> Result<String> {
        self.enrolled_experiments
            .iter()
            .find(|e| e.id == exp_name)
            .map(|e| e.branch.clone())
            .ok_or_else(|| anyhow::format_err!("No branch").into()) // Should be returning an option! But for now...
    }

    pub fn get_enrolled_experiments(&self) -> &Vec<EnrolledExperiment> {
        &self.enrolled_experiments
    }

    pub fn get_experiments(&self) -> &Vec<Experiment> {
        &self.experiments
    }

    pub fn get_bucket(&self) -> u32 {
        self.bucket_no
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EnrolledExperiment {
    id: String,
    branch: String,
}

impl EnrolledExperiment {
    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_branch(&self) -> &String {
        &self.branch
    }
}

// ============ Below are a bunch of types that gets serialized/deserialized and stored in our `Experiments` struct ============
// ============ They currently follow the old schema, and need to be updated to match the new Nimbus schema         ============

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Experiment {
    pub id: String,
    pub description: String,
    pub last_modified: u64,
    pub schema_modified: Option<u64>,
    pub buckets: Bucket,
    pub branches: Vec<Branch>,
    #[serde(rename = "match")]
    pub matcher: Matcher,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Bucket {
    pub count: u32,
    pub start: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Branch {
    pub name: String,
    ratio: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Matcher {
    pub app_id: Option<String>,
    pub app_display_version: Option<String>,
    pub app_min_version: Option<String>, // Do what AC does and have a VersionOptionString instead?
    pub app_max_version: Option<String>, //Dito
    pub locale_language: Option<String>,
    pub locale_country: Option<String>,
    pub device_manufacturer: Option<String>,
    pub device_model: Option<String>,
    pub regions: Vec<String>,
    pub debug_tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AppContext {
    pub app_id: Option<String>,
    pub app_version: Option<String>,
    pub locale_language: Option<String>,
    pub locale_country: Option<String>,
    pub device_manufacturer: Option<String>,
    pub device_model: Option<String>,
    pub region: Option<String>,
    pub debug_tag: Option<String>,
}

impl From<msg_types::AppContext> for AppContext {
    fn from(proto_ctx: msg_types::AppContext) -> Self {
        Self {
            app_id: proto_ctx.app_id,
            app_version: proto_ctx.app_version,
            locale_language: proto_ctx.locale_language,
            locale_country: proto_ctx.locale_country,
            device_manufacturer: proto_ctx.device_manufacturer,
            device_model: proto_ctx.device_model,
            region: proto_ctx.region,
            debug_tag: proto_ctx.debug_tag,
        }
    }
}

// No tests implemented just yet
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
