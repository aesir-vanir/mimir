// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Rust bindings over the Oracle Database Programming Interface for Drivers and Applications.
#![deny(missing_docs)]
#![feature(ptr_internals, try_from, untagged_unions)]
#![recursion_limit = "128"]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate getset;
#[macro_use]
mod macros;
#[cfg(feature = "trace")]
#[macro_use]
extern crate slog;
#[cfg(feature = "trace")]
#[macro_use]
extern crate slog_try;

extern crate chrono;
extern crate hex_slice;
#[cfg(not(feature = "trace"))]
extern crate slog;

mod common;
mod connection;
mod context;
mod data;
mod dequeue;
mod enqueue;
mod error;
mod lob;
mod message;
mod object;
mod objectattr;
mod objecttype;
mod odpi;
mod pool;
mod query;
mod rowid;
mod statement;
mod subscription;
mod util;
mod variable;

// Public API

pub use connection::Connection;
pub use context::params::AppContext;
pub use context::Context;
pub use data::{Data, TypeInfo};
pub use dequeue::Options as DeqOptions;
pub use enqueue::Options as EnqOptions;
pub use error::{Error, Result};
pub use lob::Lob;
pub use message::Properties as MsgProps;
pub use object::Object;
pub use objectattr::ObjectAttr;
pub use objecttype::ObjectType;
pub use odpi::structs::{
    ODPIBytes, ODPIData, ODPIDataBuffer, ODPIObjectAttrInfo, ODPIObjectTypeInfo, ODPISubscrMessage,
};
pub use odpi::{constants, enums, flags};
pub use pool::Pool;
pub use query::Info as QueryInfo;
pub use rowid::Rowid;
pub use statement::Statement;
pub use util::ODPIStr;
pub use variable::Var;
