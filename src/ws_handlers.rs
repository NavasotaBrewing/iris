use brewdrivers::model::{Device, RTU};
use log::*;
use std::time::Duration;

use crate::clients::Clients;
use crate::incoming_event::{IncomingEvent, IncomingEventType};
use crate::outgoing_event::{OutgoingData, OutgoingEvent, OutgoingEventType};

pub async fn handle_event<'a>(event: IncomingEvent, clients: &Clients, _client_id: &str) {
    // Log the incoming event
    info!(
        "Received incoming event type {:?} with {} devices attached",
        event.event_type,
        event.devices.len()
    );
    for device in &event.devices {
        trace!("\tAttached device: {} ({})", device.id, device.name);
    }

    clients.send_to_all(OutgoingEvent::lock()).await;
    let response = match event.event_type {
        IncomingEventType::DeviceEnact => handle_device_enact(event).await,
        IncomingEventType::DeviceUpdate => handle_device_update(event).await,
        IncomingEventType::RTUReset => handle_rtu_reset(event).await,
        IncomingEventType::RTUEnact => handle_rtu_enact(event).await,
    };
    clients.send_to_all(OutgoingEvent::unlock()).await;

    // Send the update/enact result to all clients.
    // This is so that when one client changes a device state, all the other
    // clients will also update immediately without having to wait for the next
    // periodic RTU update.
    //
    // Essentially, if you only change device states from a client, every client
    // *should* be in sync. The periodic updates are just to make sure of that,
    // because of Murphy's Law.
    info!("Event handled, sending a response event to all clients");
    clients.send_to_all(response).await;
}

async fn handle_device_update<'a>(event: IncomingEvent) -> OutgoingEvent<'a> {
    // Note: this method and handle_device_enact are very similar
    // I'm keeping them as separate functions because they might
    // diverge later.

    // If they don't provide at least one device, return an error
    if event.devices.len() < 1 {
        let error_msg = format!(
            "Got DeviceUpdate event with {} devices",
            event.devices.len()
        );
        error!("{}", error_msg);
        return OutgoingEvent::error(error_msg, OutgoingData::Devices(event.devices));
    }

    // The devices to include in the response
    let mut response_devices: Vec<Device> = Vec::new();
    // Devices that errored. If there's any of these, we'll return an error response
    let mut error_devices: Vec<Device> = Vec::new();

    for mut device in event.devices {
        info!("{}: Updating device", device.id);
        match device.update().await {
            Ok(_) => {
                info!("{}: success", device.id);
                response_devices.push(device);
            }
            Err(e) => {
                error!("{}: failed", device.id);
                error!("{e}");
                error_devices.push(device);
            }
        }
    }

    if error_devices.len() > 0 {
        // Return an error here
        let error_msg = format!(
            "{} devices encountered errors. See logs for more details.",
            error_devices.len()
        );
        return OutgoingEvent::error(error_msg, OutgoingData::Devices(error_devices));
    } else {
        // Return a success
        let success_message = format!("{} devices updated successfully", response_devices.len());
        info!("{success_message}");
        return OutgoingEvent::new(
            OutgoingEventType::DeviceUpdateResult,
            Some(success_message),
            OutgoingData::Devices(response_devices),
        );
    }
}

async fn handle_device_enact<'a>(event: IncomingEvent) -> OutgoingEvent<'a> {
    // If they don't provide at least one device, return an error
    if event.devices.len() < 1 {
        let error_msg = format!("Got DeviceEnact event with {} devices", event.devices.len());
        error!("{}", error_msg);
        return OutgoingEvent::error(error_msg, OutgoingData::Devices(event.devices));
    }

    // The devices to include in the response
    let mut response_devices: Vec<Device> = Vec::new();
    // Devices that errored. If there's any of these, we'll return an error response
    let mut error_devices: Vec<Device> = Vec::new();

    let delay = Duration::from_millis(event.time_between);

    for mut device in event.devices {
        info!("{}: Enacting device", device.id);
        match device.enact().await {
            Ok(_) => {
                info!("{}: success", device.id);
                response_devices.push(device);
            }
            Err(e) => {
                error!("{}: failed", device.id);
                error!("{e}");
                error_devices.push(device);

                // If they want to halt after the first error,
                // break from the loop and let the error response go
                if event.halt_if_error {
                    error!("Because of the above error, and halt_if_error = true, halting DeviceEnact event");
                    break;
                }
            }
        }
        // TODO: this will sleep after the last enact, which will delay the whole
        // response by `delay` milliseconds. Not very efficient.
        tokio::time::sleep(delay).await;
    }

    if error_devices.len() > 0 {
        // Return an error here
        let error_msg = format!(
            "{} devices encountered errors. See logs for more details.",
            error_devices.len()
        );
        return OutgoingEvent::error(error_msg, OutgoingData::Devices(error_devices));
    } else {
        // Return a success
        let success_message = format!("{} devices enacted successfully", response_devices.len());
        info!("{success_message}");
        return OutgoingEvent::new(
            OutgoingEventType::DeviceEnactResult,
            Some(success_message),
            OutgoingData::Devices(response_devices),
        );
    }
}

async fn handle_rtu_reset<'a>(mut _event: IncomingEvent) -> OutgoingEvent<'a> {
    // Generate an RTU from the configuration file.
    // This will have all default device states, except where the user specifies a state in the
    // config file.
    let mut original = match crate::generate_rtu() {
        Ok(original) => original,
        Err(e) => {
            return OutgoingEvent::error(
                format!(
                    "Couldn't generate default RTU state from config file: {e}. This is very unusual"
                ),
                OutgoingData::None,
            );
        }
    };

    if let Err(e) = original.enact().await {
        error!("Couldn't reset RTU to default state: {e}");
        return OutgoingEvent::error(
            format!("Couldn't reset RTU to default state: {e}"),
            OutgoingData::None,
        );
    }

    // Create a vec of devices to return
    let mut response_devices: Vec<Device> = Vec::new();
    for device in original.devices {
        response_devices.push(device);
    }

    return OutgoingEvent::new(
        OutgoingEventType::DeviceEnactResult,
        None,
        OutgoingData::Devices(response_devices),
    );
}

async fn handle_rtu_enact<'a>(event: IncomingEvent) -> OutgoingEvent<'a> {
    info!("Handling RTU Enact event");

    if let Some(mut rtu) = event.RTU {
        match rtu.enact().await {
            Ok(_) => {
                return OutgoingEvent::new(
                    OutgoingEventType::RTUEnactResult,
                    None,
                    OutgoingData::None,
                )
            }
            Err(e) => {
                return OutgoingEvent::error(format!("Couldn't enact rtu: {e}"), OutgoingData::None)
            }
        }
    } else {
        let error_msg = "Couldn't enact RTU: there is no RTU attached to the message";
        error!("{error_msg}");
        return OutgoingEvent::error(error_msg.to_string(), OutgoingData::None);
    }
}
