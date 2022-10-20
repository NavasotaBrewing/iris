use serde::{Deserialize, Serialize};
use log::{trace, error};

use brewdrivers::controllers::RelayBoard;
use brewdrivers::controllers::*;
use brewdrivers::drivers::InstrumentError;

type Result<T> = std::result::Result<T, InstrumentError>;

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
    pub async fn update(&mut self) -> Result<()> {
        trace!("Updating device `{}`", self.id);
        match self.controller {
            Controller::STR1 => {
                trace!("Matched to controller STR1");
                self.handle_relay_board_update(STR1::connect(self.controller_addr, &self.port)?)
                    .await?;
            }
            Controller::Waveshare => {
                trace!("Matched to controller Waveshare");
                self.handle_relay_board_update(Waveshare::connect(
                    self.controller_addr,
                    &self.port,
                )?)
                .await?;
            }
            Controller::CN7500 => {
                trace!("Matched to controller CN7500");
                self.handle_pid_update(CN7500::connect(self.controller_addr, &self.port).await?)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn enact(&mut self) -> Result<()> {
        trace!("Enacting device `{}`", self.id);
        match self.controller {
            Controller::STR1 => {
                trace!("Matched to controller STR1");
                self.handle_relay_board_enact(STR1::connect(self.controller_addr, &self.port)?)
                    .await?;
            }
            Controller::Waveshare => {
                trace!("Matched to controller Waveshare");
                self.handle_relay_board_enact(Waveshare::connect(
                    self.controller_addr,
                    &self.port,
                )?)
                .await?;
            }
            Controller::CN7500 => {
                trace!("Matched to controller CN7500");
                self.handle_pid_enact(CN7500::connect(self.controller_addr, &self.port).await?)
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_relay_board_update<C: RelayBoard<C>>(
        &mut self,
        mut controller: C,
    ) -> Result<()> {

        trace!("Handling relay board update for device: `{}`", self.id);
        self.state = AnyState::BinaryState(controller.get_relay(self.addr)?);
        Ok(())

    }

    async fn handle_relay_board_enact<C: RelayBoard<C>>(
        &mut self,
        mut controller: C,
    ) -> Result<()> {

        trace!("Handling relay board enaction for device: `{}`", self.id);
        match self.state {
            AnyState::BinaryState(new_state) => controller.set_relay(self.addr, new_state)?,
            AnyState::SteppedState(bad_state) => {
                error!("Got the wrong state in enaction, found `{}`", bad_state);
                return Err(InstrumentError::StateError(AnyState::SteppedState(
                    bad_state,
                )))
            }
        }
        Ok(())
        
    }

    async fn handle_pid_update<C: PID<C>>(&mut self, mut controller: C) -> Result<()> {
        trace!("Handling PID update for device `{}`", self.id);
        self.pv = Some(controller.get_pv().await?);
        self.sv = Some(controller.get_sv().await?);
        self.state = match controller.is_running().await? {
            true => AnyState::BinaryState(BinaryState::On),
            false => AnyState::BinaryState(BinaryState::Off),
        };
        Ok(())
    }

    async fn handle_pid_enact<C: PID<C>>(&mut self, mut controller: C) -> Result<()> {
        trace!("Handing PID enaction for device `{}`", self.id);
        match self.state {
            AnyState::BinaryState(BinaryState::On) => controller.run().await?,
            AnyState::BinaryState(BinaryState::Off) => controller.stop().await?,
            AnyState::SteppedState(bad_state) => {
                error!("Got the wrong state in enaction, found `{}`", bad_state);
                return Err(InstrumentError::StateError(AnyState::SteppedState(
                    bad_state,
                )))
            }
        }

        if let Some(new_sv) = self.sv {
            controller.set_sv(new_sv).await?;
        }

        Ok(())
    }
}
