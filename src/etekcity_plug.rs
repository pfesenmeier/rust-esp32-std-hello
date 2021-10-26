use vesync_rs::{DeviceStatus, VeSyncAccount, VeSyncdevice};
use std::env;

const VESYNC_ACCOUNT: &str = env!("VESYNC_ACCOUNT");
const VESYNC_KEY: &str = env!("VESYNC_KEY");

pub struct EtekcityPlug {
    plug: VeSyncDevice,
}

impl EtekcityPlug {
    fn new() -> Result<EtekcityPlug> {
        let account = VeSyncAccount::login(VESYNC_ACCOUNT, VESYNC_KEY);
        let devices = account.devices()?;
        
        let plug = devices
        .iter()
        .find(|device| device.cid == "Black floor lamp").unwrap();
        EtekcityPlug { plug }
    }

    fn on(&self) -> Result {
       match self.plug.deviceStatus {
          DeviceStatus::Off => plug.device_toggle()?
          _ => ()
       }
    }
    
    fn off(&self) -> Result {
        match self.plug.deviceStatus {
            DeviceStatus::On => plug.device_toggle()?
            _ => ()
        }
    }
}

