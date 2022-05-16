use std::{ops::DerefMut, sync::Arc};
use power_profiles::{ProfileReleasedStream, PowerProfilesProxy};
use tokio::sync::Mutex;
use tokio::join;
use tokio_stream::StreamExt;
use tokio_udev::{MonitorBuilder, AsyncMonitorSocket};
use zbus::Connection;
use anyhow::Result;
use gsettings_macro::gen_settings;

mod power_profiles;

type Cookie = Arc<Mutex<Option<u32>>>;

#[gen_settings(
    file = "./co.tauos.powerd.gschema.xml",
    id = "co.tauos.powerd"
)]
pub struct ApplicationSettings;

async fn handle_released(mut stream: ProfileReleasedStream<'_>, cookie: Cookie) -> Result<()> {    
    while let Some(event) = stream.next().await {
        let args = event.args()?;
        let mut lock = cookie.lock().await;

        let cookie = lock.as_ref();
        if let Some(cookie) = cookie {
            if *cookie == args.cookie {
                *lock = None;
            }
        }
    }

    Ok(())
}

async fn handle_power_event(mut socket: AsyncMonitorSocket, proxy: PowerProfilesProxy<'_>, cookie: Cookie) -> Result<()> {
    let settings = ApplicationSettings::default();

    while let Some(Ok(event)) = socket.next().await {
        // If disabled, let's continue
        if !settings.power_saver_on_unplug() {
            continue;
        }

        let online = event.attribute_value("online");
        if let Some(online) = online {
            let mut lock = cookie.lock().await;

            if online == "0" {
                let val = lock.deref_mut();
                *val = Some(proxy.hold_profile("power-saver", "Power supply is offline", "co.tauos.powerd").await?);
            } else if let Some(cookie) = lock.as_ref() {
                proxy.release_profile(*cookie).await?;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let connection = Connection::system().await?;
    let proxy = power_profiles::PowerProfilesProxy::new(&connection).await?;

    let socket = MonitorBuilder::new()?.match_subsystem("power_supply")?.listen()?;
    let async_socket = AsyncMonitorSocket::new(socket)?;

    let released_stream = proxy.receive_profile_released().await?;

    let cookie: Cookie = Arc::new(Mutex::new(Some(0)));

    let (res1, res2) = join!(handle_released(released_stream, cookie.clone()), handle_power_event(async_socket, proxy, cookie.clone()));
    
    res1?;
    res2?;

    Ok(())
}
