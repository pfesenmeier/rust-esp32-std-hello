use std::sync::{Condvar, Mutex};
use std::{cell::RefCell, env, sync::Arc, thread, time::*};

use anyhow::*;
use log::*;

use embedded_svc::httpd::registry::*;
use embedded_svc::httpd::*;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;
use embedded_svc::wifi::*;

use esp_idf_svc::httpd as idf;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;

use esp_idf_sys;
use vesync::VeSyncAccount;

const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");
const VESYNC_ACCOUNT: &str = env!("VESYNC_ACCOUNT");
const VESYNC_KEY: &str = env!("VESYNC_KEY");
const VESYNC_DEVICE_CID: &str = env!("VESYNC_DEVICE_CID");

thread_local! {
    static TLS: RefCell<u32> = RefCell::new(13);
}

fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let wifi = wifi(
        netif_stack.clone(),
        sys_loop_stack.clone(),
        default_nvs.clone(),
    )?;

    #[cfg(feature = "experimental")]
    test_https_client()?;

    let mutex = Arc::new((Mutex::new(None), Condvar::new()));
    let httpd = httpd(mutex.clone())?;
    let mut wait = mutex.0.lock().unwrap();

    loop {
        if let Some(_cycles) = *wait {
            break;
        } else {
            wait = mutex.1.wait(wait).unwrap();
        }
    }

    for s in 0..3 {
        info!("Shutting down in {} secs", 3 - s);
        thread::sleep(Duration::from_secs(1));
    }

    drop(httpd);
    info!("Httpd stopped");

    drop(wifi);
    info!("Wifi stopped");

    Ok(())
}

fn httpd(_mutex: Arc<(Mutex<Option<u32>>, Condvar)>) -> Result<idf::Server> {
    let server = idf::ServerRegistry::new()
        .at("/")
        .get(|_| {
            info!("Found handler");
            let account = VeSyncAccount::login(VESYNC_ACCOUNT, VESYNC_KEY);
            if let Ok(account) = account {
                info!("Found account");
                if let Ok(devices) = account.get_devices() {
                    info!("Found devices");
                    if let Some(mut device) = devices
                        .into_iter()
                        .find(|device| device.cid == VESYNC_DEVICE_CID)
                    {
                        info!("Found device");
                        device.device_toggle().unwrap();
                    }
                }
            }

            Ok("Device toggled!".into())
        })?
        .at("/foo")
        .get(|_| bail!("Boo, something happened!"))?
        .at("/bar")
        .get(|_| {
            Response::new(403)
                .status_message("No permissions")
                .body("You have no permissions to access this page".into())
                .into()
        })?;
    server.start(&Default::default())
}

fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");

        ping(&ip_settings)?;
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

fn ping(ip_settings: &ipv4::ClientSettings) -> Result<()> {
    info!("About to do some pings for {:?}", ip_settings);

    let ping_summary =
        ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!(
            "Pinging gateway {} resulted in timeouts",
            ip_settings.subnet.gateway
        );
    }

    info!("Pinging done");

    Ok(())
}

#[cfg(feature = "experimental")]
fn test_https_client() -> Result<()> {
    use embedded_svc::http::{self, client::*, status, HttpHeaders, HttpStatus};
    use esp_idf_svc::http::client::*;

    info!("testing my client");

    let url = String::from("https://google.com");

    info!("About to fetch content from {}", url);

    let mut client = EspHttpClient::new_default()?;

    let response = client.get(&url)?.submit()?;

    let mut body = Vec::new();
    io::StdIO(response.into_payload())
        .take(3084)
        .read_to_end(&mut body)?;

    info!(
        "Body (truncated to 3K):\n{:?}",
        String::from_utf8_lossy(&body).into_owned()
    );

    Ok(())
}
