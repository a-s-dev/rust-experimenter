/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Not implemented yet!!!
//! This is purely boilerplate to communicate over the ffi
//! We should define real variants for our error and use proper
//! error propegation (we can use the `thiserror` crate for that)
use ffi_support::{ErrorCode, ExternError};
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid")]
    Invalid,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<Error> for ExternError {
    fn from(_: Error) -> ExternError {
        let code = ErrorCode::new(1);
        ExternError::new_error(code, "UNEXPECTED")
    }
}

impl Into<Error> for anyhow::Error {
    fn into(self) -> Error {
        Error::Invalid
    }
}
