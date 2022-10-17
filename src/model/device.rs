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
    pub async fn update(&mut self, mode: &Mode) -> Result<(), InstrumentError> {
        match self.controller {
            Controller::STR1 => {
                self.handle_relay_board_update(
                    &mut STR1::connect(self.controller_addr, &self.port)?,
                    &mode,
                )
            }
            Controller::Waveshare => {
                self.handle_relay_board_update(
                    &mut Waveshare::connect(self.controller_addr, &self.port)?,
                    &mode
                )
            }
            Controller::CN7500 => {
                self.handle_pid_update(
                    &mut CN7500::connect(self.controller_addr, &self.port).await?, 
                    &mode 
                ).await
            }
        }
    }

    async fn handle_pid_update<T: PID<T>>(
        &mut self,
        controller: &mut T,
        mode: &Mode
    ) -> Result<(), InstrumentError> {

        if let Mode::Write = mode {
            if let Some(new_sv) = self.sv {
                controller.set_sv(new_sv).await?;
            }

            match self.state {
                AnyState::BinaryState(BinaryState::On) => controller.run().await?,
                AnyState::BinaryState(BinaryState::Off) => controller.stop().await?,
                AnyState::SteppedState(_) => return Err(InstrumentError::StateError(self.state))
            };
        }

        // Read|Update
        self.pv = Some(controller.get_pv().await?);
        self.sv = Some(controller.get_sv().await?);

        // Cargo won't let me implement from<bool> and from<str> at the same time :(
        let new_state = match controller.is_running().await? {
            true => BinaryState::On,
            false => BinaryState::Off
        };
        self.state = AnyState::BinaryState(new_state);

        Ok(())
    }

    fn handle_relay_board_update<C: RelayBoard<C>>(
        &mut self,
        controller: &mut C,
        mode: &Mode,
    ) -> Result<(), InstrumentError> {

        if let Mode::Write = mode {
            match self.state {
                AnyState::BinaryState(new_state) => controller.set_relay(self.addr, new_state)?,
                _ => {
                    return Err(
                        InstrumentError::serialError(
                            format!("State type is incorrect, this device uses a binary state 'On' or 'Off', found `{:?}`", self.state), Some(self.addr)
                        )
                    )
                }
            }
        }

        self.state = AnyState::BinaryState(controller.get_relay(self.addr)?);
        Ok(())
    }
}
