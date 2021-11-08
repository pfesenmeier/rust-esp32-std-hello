use vesync::{Status, VeSyncAccount, VeSyncDevice};
use std::env;
use anyhow;

const VESYNC_ACCOUNT: &str = env!("VESYNC_ACCOUNT");
const VESYNC_KEY: &str = env!("VESYNC_KEY");

pub struct EtekcityPlug<'a> {
    plug: VeSyncDevice<'a>,
}

impl EtekcityPlug<'_> {
    fn new() -> Result<EtekcityPlug<'static>, anyhow::Error> {
        let account = VeSyncAccount::login(VESYNC_ACCOUNT, VESYNC_KEY);
        let devices = account.unwrap().get_devices().unwrap();
        
        let plug = devices
        .iter()
        .find(|device| device.cid == "Black floor lamp").unwrap();
        let plug = *plug;
        Ok(EtekcityPlug { plug })
    }

    fn on(&self) -> Option<()> {
       match self.plug.deviceStatus {
          Status::Off => self.plug.device_toggle(),
          _ => Ok(()),
       };
       Some(())
    }
    
    fn off(&self) -> Option<()> {
        match self.plug.deviceStatus {
            Status::On => self.plug.device_toggle(),
            _ => Ok(()),
        };
        Some(())
    }
}

