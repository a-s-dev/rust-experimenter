/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This is where the persistence logic might go.
//! An idea for what to use here might be [RKV](https://github.com/mozilla/rkv)
//! And that's what's used on this prototype,
//! Either ways, the solution implemented should work regardless of the platform
//! on the other side of the FFI. This means that this module might require the FFI to allow consumers
//! To pass in a path to a database, or somewhere in the file system that the state will be persisted
use anyhow::Result;
use rkv::{Rkv, SingleStore, StoreOptions};
use serde_derive::*;
use std::fs;
use std::path::Path;

pub struct Database {
    rkv: Rkv,

    experiment_store: SingleStore,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let rkv = Self::open_rkv(path)?;
        let experiment_store = rkv
            .open_single("experiments", StoreOptions::create())
            .unwrap();
        Ok(Self {
            rkv,
            experiment_store,
        })
    }

    fn open_rkv<P: AsRef<Path>>(path: P) -> Result<Rkv> {
        let path = std::path::Path::new(path.as_ref()).join("db");
        log::debug!("Database path: {:?}", path.display());
        fs::create_dir_all(&path)?;

        let rkv = Rkv::new(&path).unwrap(); // Rkv errors should impl std::error::Error :( TODO: Impl proper error handling in an error.rs that can propagate
        log::info!("Database initialized");
        Ok(rkv)
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let reader = self.rkv.read().unwrap();
        let val = self.experiment_store.get(&reader, key).unwrap();
        Ok(val.map(|v| {
            if let rkv::Value::Json(val) = v {
                val.to_string()
            } else {
                "".to_string() // BAD IDEA! Remove this!
            }
        }))
    }

    pub fn put(&self, key: &str, persisted_data: PersistedData) -> Result<()> {
        let mut writer = self.rkv.write().unwrap();
        let persisted_json = serde_json::to_string(&persisted_data).unwrap();
        self.experiment_store
            .put(&mut writer, key, &rkv::Value::Json(&persisted_json))
            .unwrap();
        writer.commit().unwrap();
        Ok(())
    }
}

use super::{AppContext, EnrolledExperiment, Experiment};
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PersistedData {
    pub app_ctx: AppContext,
    pub experiments: Vec<Experiment>,
    pub enrolled_experiments: Vec<EnrolledExperiment>,
    pub bucket_no: u32,
    pub uuid: uuid::Uuid,
}
