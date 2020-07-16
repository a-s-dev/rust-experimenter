/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This is where the persistence logic might go.
//! An idea for what to use here might be [RKV](https://github.com/mozilla/rkv)
//! Either ways, the solution implemented should work regardless of the plateform
//! on the other side of the FFI. This means that this module might require the FFI to allow consumers
//! To pass in a path to a database, or somewhere in the file system that the state will be persisted
