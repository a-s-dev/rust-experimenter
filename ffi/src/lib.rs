/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 use std::os::raw::c_char;

 use ffi_support::{define_handle_map_deleter, ConcurrentHandleMap, ExternError, FfiStr};
 use experiments::{Experiments, error::Result};
 // Hack to allow consumer to do all the viaduct magic
pub use viaduct::*;

 lazy_static::lazy_static! {
     static ref EXPERIMENTS: ConcurrentHandleMap<Experiments> = ConcurrentHandleMap::new();
 }
 
 #[no_mangle]
 pub extern "C" fn experiments_new(error: &mut ExternError, base_url: FfiStr<'_>, collection_name: FfiStr<'_>, bucket_name: FfiStr<'_>) -> u64 {
     EXPERIMENTS.insert_with_result(error, || -> Result<Experiments> {
         Ok(Experiments::new(base_url.as_str(), collection_name.as_str(), bucket_name.as_str()))
     })
 }
 

 #[no_mangle]
 pub extern "C" fn experiements_get_branch(
     handle: u64,
     branch: FfiStr<'_>,
     error: &mut ExternError,
 ) -> *mut c_char {
     EXPERIMENTS.call_with_result(error, handle, |experiment| -> Result<String> {
         let ret = experiment.get_experiment_branch(branch.as_str())?;
         Ok(ret)
     })
 }

 define_handle_map_deleter!(EXPERIMENTS, experiements_destroy);
 