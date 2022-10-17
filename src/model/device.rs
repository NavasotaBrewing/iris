use serde::{Deserialize, Serialize};

use super::Mode;

use brewdrivers::controllers::RelayBoard;
use brewdrivers::controllers::*;
use brewdrivers::drivers::InstrumentError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub port: String,
    pub addr: u8,
    pub controller: Controller,
    pub controller_addr: u8,
    pub state: AnyState,
    pub pv: Option<f64>,
    pub sv: Option<f64>,
}

impl Device {
    /// Updates either the device state or the Model state,
    /// depending on the Mode.
    pub async fn update(mut device: &mut Device, mode: &Mode) -> Result<(), InstrumentError> {
        match device.controller {
            Controller::STR1 => {
                Self::handle_relay_board_update(
                    &mut STR1::connect(device.controller_addr, &device.port)?,
                    &mut device,
                    &mode,
                )
            }
            Controller::Waveshare => {
                Self::handle_relay_board_update(
                    &mut Waveshare::connect(device.controller_addr, &device.port)?,
                    &mut device,
                    &mode
                )
            }
            Controller::CN7500 => {
                Self::handle_pid_update(
                    &mut CN7500::connect(device.controller_addr, &device.port).await?, 
                    &mut device,
                    &mode 
                ).await
            }
        }
    }

    async fn handle_pid_update<T: PID<T>>(
        controller: &mut T,
        device: &mut Device,
        mode: &Mode
    ) -> Result<(), InstrumentError> {

        if let Mode::Write = mode {
            // TODO: Handle None case
            if let Some(new_sv) = device.sv {
                controller.set_sv(new_sv).await?;
            }

            match device.state {
                AnyState::BinaryState(BinaryState::On) => controller.run().await?,
                AnyState::BinaryState(BinaryState::Off) => controller.stop().await?,
                AnyState::SteppedState(_) => return Err(InstrumentError::StateError(device.state))
            };
        }

        // Read|Update
        device.pv = Some(controller.get_pv().await?);
        device.sv = Some(controller.get_sv().await?);

        // Cargo won't let me implement from<bool> and from<str> at the same time :(
        let new_state = match controller.is_running().await? {
            true => BinaryState::On,
            false => BinaryState::Off
        };
        device.state = AnyState::BinaryState(new_state);

        Ok(())
    }

    fn handle_relay_board_update<C: RelayBoard<C>>(
        controller: &mut C,
        device: &mut Device,
        mode: &Mode,
    ) -> Result<(), InstrumentError> {

        if let Mode::Write = mode {
            match device.state {
                AnyState::BinaryState(new_state) => controller.set_relay(device.addr, new_state)?,
                _ => {
                    return Err(
                        InstrumentError::serialError(
                            format!("State type is incorrect, this device uses a binary state 0 or 1, found `{:?}`", device.state), Some(device.addr)
                        )
                    )
                }
            }
        }

        device.state = AnyState::BinaryState(controller.get_relay(device.addr)?);
        Ok(())
    }
}
