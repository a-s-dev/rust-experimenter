/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Experiments library that hopes to be cross-plateform.
//! Still a work in progress and essentially has zero functionality so far

use url::Url;
mod buckets;
pub mod error;
mod http_client;
mod persistence;
use error::Result;
use http_client::{Client, SettingsClient};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde_derive::*;
const MAX_BUCKET_NO: u32 = 1000;

/// Experiments is the main struct representing the experiements state
/// It should hold all the information needed to communcate a specific user's
/// Experiementation status (note: This should have some type of uuid)
pub struct Experiments {
    experiments: Vec<Experiment>,
    enrolled_experiments: Vec<EnrolledExperiment>,
    bucket_no: u32,
}

impl Experiments {
    // A new experiments struct is created this is where some preprocessing happens
    // It should look for persisted state first (once that is implemented) and setup some type
    // Of interval retrieval from the server for any experiment updates
    pub fn new(base_url: &str, collection_name: &str, bucket_name: &str) -> Self {
        let http_client = Client::new(
            Url::parse(base_url).unwrap(),
            collection_name.to_string(),
            bucket_name.to_string(),
        );
        let resp = http_client.get_experiments().unwrap();
        log::info!("Creating experiement....");

        let uuid: [u8; 4] = rand::random(); // This is ***not*** a  real uuid, it's purely for testing/demoing purposes
        let bucket_no: u32 = u32::from_be_bytes(uuid) % MAX_BUCKET_NO;
        let mut num = StdRng::seed_from_u64(bucket_no as u64); // Don't look at me, I'm not a good way to generate a random number!!!!
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
            .collect();
        Self {
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

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct Bucket {
    pub count: u32,
    pub start: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Branch {
    pub name: String,
    ratio: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Matcher {
    app_id: Option<String>,
    app_display_version: Option<String>,
    app_min_version: Option<String>, // Do what AC does and have a VersionOptionString instead?
    app_max_version: Option<String>, //Dito
    locale_language: Option<String>,
    locale_country: Option<String>,
    device_manufacturer: Option<String>,
    device_model: Option<String>,
    regions: Vec<String>,
    debug_tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
