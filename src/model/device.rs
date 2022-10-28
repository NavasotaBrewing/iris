use serde::{Deserialize, Serialize};
use log::{trace, error};

use brewdrivers::controllers::*;
use brewdrivers::drivers::InstrumentError;

type Result<T> = std::result::Result<T, InstrumentError>;

/// A digital represenation of a device
/// 
/// Devices are not controllers. They operate on controllers, and sometimes there is 1 device for 1 controllers.
/// And example is that each relay on a relay board is it's own device, so 1 controller -> 8 devices (or similar).
/// Or we could have 1 PID controller that controls 1 Thermometer device.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Device {
    /// The ID of the device, must be unique among all devices on all RTUs
    pub id: String,
    /// A pretty name, for display purposes
    pub name: String,
    /// The serial port the device runs on.
    /// 
    /// This will probably be `/dev/ttyUSB0` or `/dev/ttyAMA0`
    pub port: String,
    /// The devices specific address (ie. relay number, etc.)
    /// 
    /// If the device has no specific address within the controller, set to 0
    pub addr: u8,
    /// The type of controller the device runs on
    pub controller: Controller,
    /// The address of the controller on the RS485 bus
    pub controller_addr: u8,
    /// The state of the device. Different devices use different types of state.
    pub state: AnyState,
    /// The process value of a PID
    pub pv: Option<f64>,
    /// The setpoint value of a PID
    pub sv: Option<f64>,
}

impl Device {
    /// Polls a device for it's state and updates `self` to match
    /// 
    /// ```rust
    /// use nbc_iris::model::Device;
    /// use brewdrivers::controllers::*;
    /// 
    /// // This assumes we have a CN7500 on Address 22 (0x16) through /dev/ttyUSB0
    /// let mut dev = Device {
    ///     id: String::from("my-dev"),
    ///     name: String::from("My Dev"),
    ///     port: String::from("/dev/ttyUSB0"),
    ///     addr: 0,
    ///     controller: Controller::CN7500,
    ///     controller_addr: 22,
    ///     state: AnyState::BinaryState(BinaryState::On),
    ///     pv: None,
    ///     sv: None
    /// };
    /// 
    /// tokio_test::block_on(async {
    ///     dev.update().await.unwrap();
    /// });
    /// 
    /// println!("{:?}", dev);
    /// assert!(dev.pv.is_some());
    /// ```
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

    /// Writes the state of `self` onto the controller
    /// 
    /// ```rust
    /// use nbc_iris::model::Device;
    /// use brewdrivers::controllers::*;
    /// 
    /// // This assumes we have a CN7500 on Address 22 (0x16) through /dev/ttyUSB0
    /// let mut dev = Device {
    ///     id: String::from("my-dev"),
    ///     name: String::from("My Dev"),
    ///     port: String::from("/dev/ttyUSB0"),
    ///     addr: 0,
    ///     controller: Controller::CN7500,
    ///     controller_addr: 22,
    ///     state: AnyState::BinaryState(BinaryState::On),
    ///     pv: None,
    ///     sv: Some(167.0)                                 // This will change the value on the CN7500
    /// };
    /// 
    /// 
    /// tokio_test::block_on(async {
    ///     dev.enact().await.unwrap();
    /// });
    /// 
    /// println!("{:?}", dev);
    /// assert_eq!(dev.sv.unwrap(), 167.0);
    /// ```
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
