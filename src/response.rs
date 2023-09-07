//! Responses through the websocket

use serde::Serialize;

use brewdrivers::{
    drivers::InstrumentError,
    model::{Device, RTU},
};
use warp::ws::Message;

/// The valid types of a response
#[derive(Debug, Serialize)]
pub(crate) enum EventResponseType {
    Error,
    DeviceUpdateResult,
    DeviceEnactResult,
    RTUUpdateResult,
}

/// A wrapper around the types of data we may want to attach to a response payload
#[derive(Debug, Serialize)]
pub(crate) enum ResponseData<'a> {
    Device(Device),
    RTU(&'a RTU),
    None,
}

/// A response payload
#[derive(Debug, Serialize)]
pub(crate) struct EventResponse<'a> {
    response_type: EventResponseType,
    message: Option<String>,
    data: ResponseData<'a>,
}

impl<'a> EventResponse<'a> {
    /// Creates a new event response
    pub(crate) fn new(
        response_type: EventResponseType,
        message: Option<String>,
        data: ResponseData<'a>,
    ) -> Self {
        Self {
            response_type,
            message,
            data,
        }
    }

    /// Creates a new event response, marked as an error
    pub(crate) fn error(message: String, data: ResponseData<'a>) -> Self {
        Self {
            response_type: EventResponseType::Error,
            message: Some(message),
            data,
        }
    }

    /// Creates a new event response holding an RTU
    pub(crate) fn rtu(rtu: &'a RTU) -> Self {
        Self {
            response_type: EventResponseType::RTUUpdateResult,
            message: None,
            data: ResponseData::RTU(rtu),
        }
    }

    pub(crate) fn to_msg(&self) -> Message {
        Message::text(serde_json::to_string(self).unwrap())
    }

    pub(crate) fn response_type(&self) -> &EventResponseType {
        &self.response_type
    }

    pub(crate) fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl<'a> From<InstrumentError> for EventResponse<'a> {
    fn from(e: InstrumentError) -> Self {
        Self::error(format!("Instrument error: {e}"), ResponseData::None)
    }
}

