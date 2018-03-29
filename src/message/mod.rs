// Copyright (c) 2017 mimir developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Message properties handles are used to represent the properties of messages that are enqueued
//! and dequeued using advanced queuing. They are created by calling the function
//! `Connection::new_msg_props()` and are destroyed by releasing the last reference by calling the
//! function `Properties::release()`.
use chrono::{DateTime, Utc};
use error::{ErrorKind, Result};
use odpi::opaque::ODPIMsgProps;
use odpi::structs::ODPITimestamp;
use odpi::{enums, externs};
use std::convert::TryFrom;
use std::ptr;
use util::ODPIStr;

/// ODPI-C Message Props wrapper.
#[derive(Clone)]
pub struct Properties {
    /// The ODPI-C MsgProps pointer.
    inner: *mut ODPIMsgProps,
}

impl Properties {
    #[doc(hidden)]
    pub fn inner(&self) -> *mut ODPIMsgProps {
        self.inner
    }

    /// Returns the number of attempts that have been made to dequeue a message.
    pub fn get_num_attempts(&self) -> Result<i32> {
        let mut attempts = 0;

        try_dpi!(
            externs::dpiMsgProps_getNumAttempts(self.inner, &mut attempts),
            Ok(attempts),
            ErrorKind::MsgProps("dpiMsgProps_getNumAttempts".to_string())
        )
    }

    /// Returns the correlation supplied by the producer when the message was enqueued.
    pub fn get_correlation(&self) -> Result<String> {
        let mut corr_ptr = ptr::null();
        let mut corr_len = 0;

        try_dpi!(
            externs::dpiMsgProps_getCorrelation(self.inner, &mut corr_ptr, &mut corr_len),
            {
                let correlation = if corr_ptr.is_null() {
                    "".to_string()
                } else {
                    let corr_s = ODPIStr::new(corr_ptr, corr_len);
                    corr_s.into()
                };
                Ok(correlation)
            },
            ErrorKind::Lob("dpiMsgProps_getCorrelation".to_string())
        )
    }

    /// Returns the number of seconds the enqueued message will be delayed.
    pub fn get_delay(&self) -> Result<i32> {
        let mut delay = 0;
        try_dpi!(
            externs::dpiMsgProps_getDelay(self.inner, &mut delay),
            Ok(delay),
            ErrorKind::MsgProps("dpiMsgProps_getDelay".to_string())
        )
    }

    /// Returns the mode that was used to deliver the message.
    pub fn get_delivery_mode(&self) -> Result<enums::ODPIMessageDeliveryMode> {
        let mut del_mode_ptr = enums::ODPIMessageDeliveryMode::NotSet;

        try_dpi!(
            externs::dpiMsgProps_getDeliveryMode(self.inner, &mut del_mode_ptr),
            Ok(del_mode_ptr),
            ErrorKind::MsgProps("dpiEnqOptions_getMode".to_string())
        )
    }

    /// Returns the time that the message was enqueued.
    pub fn get_enq_time(&self) -> Result<DateTime<Utc>> {
        let mut timestamp: ODPITimestamp = Default::default();

        try_dpi!(
            externs::dpiMsgProps_getEnqTime(self.inner, &mut timestamp),
            Ok(timestamp.into()),
            ErrorKind::MsgProps("dpiMsgProps_getEnqTime".to_string())
        )
    }

    /// Returns the name of the queue to which the message is moved if it cannot be processed
    /// successfully. See function `MsgProps::set_exception_q()` for more information.
    pub fn get_exception_q(&self) -> Result<String> {
        let mut exception_q_ptr = ptr::null();
        let mut exception_q_len = 0;

        try_dpi!(
            externs::dpiMsgProps_getExceptionQ(
                self.inner,
                &mut exception_q_ptr,
                &mut exception_q_len
            ),
            {
                let exception_q = ODPIStr::new(exception_q_ptr, exception_q_len);
                Ok(exception_q.into())
            },
            ErrorKind::MsgProps("dpiMsgProps_getExceptionQ".to_string())
        )
    }

    /// Returns the number of seconds the message is available to be dequeued. See function
    /// `MsgProps::set_expiration()` for more information.
    pub fn get_expiration(&self) -> Result<i32> {
        let mut seconds = 0;

        try_dpi!(
            externs::dpiMsgProps_getExpiration(self.inner, &mut seconds),
            Ok(seconds),
            ErrorKind::MsgProps("dpiMsgProps_getExpiration".to_string())
        )
    }

    /// Returns the id of the message in the last queue that generated this message. See function
    /// `MsgProps::set_original_msg_id()` for more information.
    pub fn get_original_msg_id(&self) -> Result<String> {
        let mut orig_msg_id_ptr = ptr::null();
        let mut orig_msg_id_len = 0;

        try_dpi!(
            externs::dpiMsgProps_getOriginalMsgId(
                self.inner,
                &mut orig_msg_id_ptr,
                &mut orig_msg_id_len
            ),
            {
                let orig_msg_id = ODPIStr::new(orig_msg_id_ptr, orig_msg_id_len);
                Ok(orig_msg_id.into())
            },
            ErrorKind::MsgProps("dpiMsgProps_getOriginalMsgId".to_string())
        )
    }

    /// Returns the priority assigned to the message. See function `MsgProps::set_priority()` for
    /// more information.
    pub fn get_priority(&self) -> Result<i32> {
        let mut priority = 0;

        try_dpi!(
            externs::dpiMsgProps_getPriority(self.inner, &mut priority),
            Ok(priority),
            ErrorKind::MsgProps("dpiMsgProps_getPriority".to_string())
        )
    }

    /// Returns the state of the message at the time of dequeue.
    pub fn get_state(&self) -> Result<enums::ODPIMessageState> {
        let mut state = enums::ODPIMessageState::Ready;

        try_dpi!(
            externs::dpiMsgProps_getState(self.inner, &mut state),
            Ok(state),
            ErrorKind::MsgProps("dpiMsgProps_getState".to_string())
        )
    }

    /// Sets the correlation of the message to be dequeued. Special pattern matching characters such
    /// as the percent sign (%) and the underscore (_) can be used. If multiple messages satisfy the
    /// pattern, the order of dequeuing is undetermined.
    pub fn set_correlation(&self, correlation: &str) -> Result<()> {
        let correlation_s: ODPIStr = TryFrom::try_from(correlation)?;

        try_dpi!(
            externs::dpiMsgProps_setCorrelation(
                self.inner,
                correlation_s.ptr(),
                correlation_s.len()
            ),
            Ok(()),
            ErrorKind::MsgProps("dpiMsgProps_setCorrelation".to_string())
        )
    }

    /// Sets the number of seconds to delay the message before it can be dequeued. Messages enqueued
    /// with a delay are put into the `Waiting` state. When the delay expires the message is put
    /// into the `Ready` state. Dequeuing directly by message id overrides this delay specification.
    /// Note that delay processing requires the queue monitor to be started.
    pub fn set_delay(&self, delay: i32) -> Result<()> {
        try_dpi!(
            externs::dpiMsgProps_setDelay(self.inner, delay),
            Ok(()),
            ErrorKind::MsgProps("dpiMsgProps_setDelay".to_string())
        )
    }

    /// Sets the name of the queue to which the message is moved if it cannot be processed
    /// successfully. Messages are moved if the number of unsuccessful dequeue attempts has reached
    /// the maximum allowed number or if the message has expired. All messages in the exception
    /// queue are in the `Expired` state.
    pub fn set_exception_q(&self, queue_name: &str) -> Result<()> {
        let queue_name_s: ODPIStr = TryFrom::try_from(queue_name)?;

        try_dpi!(
            externs::dpiMsgProps_setExceptionQ(self.inner, queue_name_s.ptr(), queue_name_s.len()),
            Ok(()),
            ErrorKind::MsgProps("dpiMsgProps_setExceptionQ".to_string())
        )
    }

    /// Sets the number of seconds the message is available to be dequeued. This value is an offset
    /// from the delay. Expiration processing requires the queue monitor to be running. Until this
    /// time elapses, the messages are in the queue in the state `Ready`. After this time elapses
    /// messages are moved to the exception queue in the `Expired` state.
    pub fn set_expiration(&self, seconds: i32) -> Result<()> {
        try_dpi!(
            externs::dpiMsgProps_setExpiration(self.inner, seconds),
            Ok(()),
            ErrorKind::MsgProps("dpiMsgProps_setExpiration".to_string())
        )
    }

    /// Sets the id of the message in the last queue that generated this message.
    pub fn set_original_msg_id(&self, id: &str) -> Result<()> {
        let id_s: ODPIStr = TryFrom::try_from(id)?;

        try_dpi!(
            externs::dpiMsgProps_setOriginalMsgId(self.inner, id_s.ptr(), id_s.len()),
            Ok(()),
            ErrorKind::MsgProps("dpiMsgProps_setOriginalMsgId".to_string())
        )
    }

    /// Sets the priority assigned to the message. A smaller number indicates a higher priority. The
    /// priority can be any number, including negative numbers.
    pub fn set_priority(&self, priority: i32) -> Result<()> {
        try_dpi!(
            externs::dpiMsgProps_setPriority(self.inner, priority),
            Ok(()),
            ErrorKind::MsgProps("dpiMsgProps_setPriority".to_string())
        )
    }
}

impl From<*mut ODPIMsgProps> for Properties {
    fn from(inner: *mut ODPIMsgProps) -> Self {
        Self { inner }
    }
}

impl Drop for Properties {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                externs::dpiMsgProps_release(self.inner);
            }
        }
    }
}
