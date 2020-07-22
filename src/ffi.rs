/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::os::raw::c_char;

use super::{error::Result, msg_types, AppContext, Experiments};
use ffi_support::{define_handle_map_deleter, ConcurrentHandleMap, ExternError, FfiStr};

lazy_static::lazy_static! {
    static ref EXPERIMENTS: ConcurrentHandleMap<Experiments> = ConcurrentHandleMap::new();
}

#[no_mangle]
pub extern "C" fn experiments_new(
    app_ctx: *const u8,
    app_ctx_len: i32,
    db_path: FfiStr<'_>,
    error: &mut ExternError,
) -> u64 {
    EXPERIMENTS.insert_with_result(error, || -> Result<Experiments> {
        let app_ctx = unsafe {
            from_protobuf_ptr::<AppContext, msg_types::AppContext>(app_ctx, app_ctx_len).unwrap()
        }; // Todo: make the whole function unsafe and implement proper error handling in error.rs
        log::info!("=================== Initializing experiments ========================");
        Ok(Experiments::new(app_ctx, db_path.as_str()))
    })
}

#[no_mangle]
pub extern "C" fn experiments_get_branch(
    handle: u64,
    branch: FfiStr<'_>,
    error: &mut ExternError,
) -> *mut c_char {
    EXPERIMENTS.call_with_result(error, handle, |experiment| -> Result<String> {
        log::info!("==================== Getting branch ========================");
        let branch_name = experiment.get_experiment_branch(branch.as_str())?;
        Ok(branch_name)
    })
}

define_handle_map_deleter!(EXPERIMENTS, experiements_destroy);

/// # Safety
/// data is a raw pointer to the protobuf data
/// get_buffer will return an error if the length is invalid,
/// or if the pointer is a null pointer
pub unsafe fn from_protobuf_ptr<T, F: prost::Message + Default + Into<T>>(
    data: *const u8,
    len: i32,
) -> anyhow::Result<T> {
    let buffer = get_buffer(data, len)?;
    let item: Result<F, _> = prost::Message::decode(buffer);
    item.map(|inner| inner.into()).map_err(|e| e.into())
}

unsafe fn get_buffer<'a>(data: *const u8, len: i32) -> anyhow::Result<&'a [u8]> {
    match len {
        len if len < 0 => anyhow::bail!("Invalid length"),
        0 => Ok(&[]),
        _ => {
            if data.is_null() {
                anyhow::bail!("Null pointer")
            }
            Ok(std::slice::from_raw_parts(data, len as usize))
        }
    }
}
