use serde::{Serialize, Deserialize};

use super::{Driver, State, Mode};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub driver: Driver,
    pub port: String,
    pub name: String,
    pub id: String,
    pub state: State,
    pub addr: u8,
    pub controller_addr: u8,
    pub pv: Option<f64>,
    pub sv: Option<f64>,
}

impl Device {
    /// Updates either the device state or the Model state, 
    /// depending on the Mode.
    ///
    /// This is one of the things that needs to be updated when new drivers
    /// are added. I'd like to extract this to a trait within `brewdrivers` so
    /// I don't have to expand this when I write a new driver.
    pub async fn update(device: &mut Device, mode: &Mode) {
        use brewdrivers::{
            omega::CN7500,
            relays::STR1,
            relays::Waveshare
        };
        // This is a little bit fucked. I'm maintaining two different states becauese
        // it'll save time and space later.
        use brewdrivers::relays::State as BState;
        
        match device.driver {
            Driver::STR1 => {
                // TODO: make an override for the port with an environment variable or file or something
                // We want to panic! here. This will be run by rocket, so if it panics it will just fail with a message
                let mut board = STR1::connect(device.controller_addr, &device.port).expect("Couldn't connect to STR1!");
                match mode {
                    Mode::Write => {
                        let new_state = match device.state {
                            State::On => BState::On,
                            State::Off => BState::Off
                        };
                        // why not board.set_relay(..., device.state)?? I'm confused
                        board.set_relay(device.addr, new_state).expect("Couldn't set relay");
                    },
                    Mode::Read => {
                        // Don't do anything here, we always read new state
                    }
                }
                // Read|Update
                // TODO: handle the result here
                device.state = match board.get_relay(device.addr).unwrap() {
                    BState::On => State::On,
                    BState::Off => State::Off
                }
            },
            Driver::Waveshare => {
                let mut board = Waveshare::connect(device.controller_addr, &device.port).expect("Couldn't connect to the Waveshare board!");
                match mode {
                    Mode::Write => {
                        let new_state = match device.state {
                            State::On => BState::On,
                            State::Off => BState::Off
                        };
                        board.set_relay(device.addr, new_state).expect("Couldn't set relay");

                    },
                    Mode::Read => {}
                }

                device.state = match board.get_relay(device.addr).unwrap() {
                    BState::On => State::On,
                    BState::Off => State::Off
                }
            }
            Driver::CN7500 => {
                let mut cn7500 = CN7500::new(device.controller_addr, &device.port, 19200).await.expect("Couldn't connect to CN7500!");
                match mode {
                    Mode::Write => {
                        cn7500.set_sv(device.sv.unwrap()).await.expect("Couldn't set SV on CN7500");
                        match device.state {
                            State::On => cn7500.run().await.expect("Couldn't start CN7500"),
                            State::Off => cn7500.stop().await.expect("Couldn't stop CN7500"),
                        };
                    },
                    Mode::Read => {
                        // Don't do anything here, we always read new state
                    }
                }

                // Read|Update
                device.pv = cn7500.get_pv().await.ok();
                device.sv = cn7500.get_sv().await.ok();
                // Again, we're ok with panic!ing here.
                if cn7500.is_running().await.expect("Couldn't communicate with CN7500!") {
                    device.state = State::On;
                } else {
                    device.state = State::Off;
                }
            }
        }

    }
}

