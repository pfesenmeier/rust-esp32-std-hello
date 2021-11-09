use anyhow;
use std::env;
use vesync::{Status, VeSyncAccount, VeSyncDevice};

const VESYNC_ACCOUNT: &str = env!("VESYNC_ACCOUNT");
const VESYNC_KEY: &str = env!("VESYNC_KEY");

pub struct EtekcityPlug {
    account: VeSyncAccount,
}

enum ToggleResult {
    TurnedOn,
    LeftOn,
    TurnedOff,
    LeftOff,
    ToggledBlindly,
    Error,
    PlugNotFound,
}

impl EtekcityPlug {
    fn new() -> Result<EtekcityPlug, anyhow::Error> {
        let account = VeSyncAccount::login(VESYNC_ACCOUNT, VESYNC_KEY).unwrap();
        Ok(EtekcityPlug { account })
    }

    fn on(&self) -> ToggleResult {
        let devices = self.account.get_devices().unwrap();

        let search_result = devices
            .into_iter()
            .find(|device| device.cid == "Black floor lamp");

        if let Some(mut plug) = search_result {
            match plug.deviceStatus {
                Status::Off => match plug.device_toggle() {
                    Ok(_) => ToggleResult::TurnedOn,
                    Err(_) => ToggleResult::Error,
                },
                Status::On => ToggleResult::LeftOn,
                Status::Unknown => {
                    if let Err(_) = plug.device_toggle() {
                        ToggleResult::Error
                    } else {
                        ToggleResult::ToggledBlindly
                    }
                }
            }
        } else {
            ToggleResult::PlugNotFound
        }
    }

    fn off(&self) -> ToggleResult {
        let devices = self.account.get_devices().unwrap();

        let search_result = devices
            .into_iter()
            .find(|device| device.cid == "Black floor lamp");

        if let Some(mut plug) = search_result {
            match plug.deviceStatus {
                Status::On => match plug.device_toggle() {
                    Ok(_) => ToggleResult::TurnedOff,
                    Err(_) => ToggleResult::Error,
                },
                Status::Off => ToggleResult::LeftOff,
                Status::Unknown => {
                    if let Err(_) = plug.device_toggle() {
                        ToggleResult::Error
                    } else {
                        ToggleResult::ToggledBlindly
                    }
                }
            }
        } else {
            ToggleResult::PlugNotFound
        }
    }
}
