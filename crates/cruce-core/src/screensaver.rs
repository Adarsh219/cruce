use tracing::debug;
use zbus::blocking::Connection;

const DBUS_DEST: &str = "org.freedesktop.ScreenSaver";
const DBUS_PATH: &str = "/org/freedesktop/ScreenSaver";
const DBUS_IFACE: &str = "org.freedesktop.ScreenSaver";

/// Controls desktop screensaver inhibition during active gameplay via Freedesktop D-Bus API.
#[derive(Debug, Clone, Default)]
pub struct ScreensaverManager;

impl ScreensaverManager {
    pub fn new() -> Self {
        Self
    }

    /// Inhibit the screensaver via D-Bus and return an RAII guard holding the cookie.
    ///
    /// When the guard is dropped or explicitly uninhibited, the D-Bus lock is automatically released.
    pub fn inhibit(&self, application_name: &str, reason: &str) -> ScreensaverGuard {
        let cookie = match Connection::session() {
            Ok(conn) => {
                let reply: Result<u32, zbus::Error> = conn
                    .call_method(
                        Some(DBUS_DEST),
                        DBUS_PATH,
                        Some(DBUS_IFACE),
                        "Inhibit",
                        &(application_name, reason),
                    )
                    .and_then(|r| r.body().deserialize());

                match reply {
                    Ok(c) => {
                        debug!("Screensaver inhibited via D-Bus, cookie: {}", c);
                        Some(c)
                    }
                    Err(e) => {
                        debug!(
                            "D-Bus Screensaver.Inhibit failed (compositor might not support it): {}",
                            e
                        );
                        None
                    }
                }
            }
            Err(e) => {
                debug!("Failed to connect to D-Bus session bus: {}", e);
                None
            }
        };

        ScreensaverGuard { cookie }
    }

    /// Releases a screensaver inhibit lock with the specified cookie.
    pub fn uninhibit(cookie: u32) {
        if cookie == 0 {
            return;
        }

        if let Ok(conn) = Connection::session() {
            let res: Result<(), zbus::Error> = conn
                .call_method(
                    Some(DBUS_DEST),
                    DBUS_PATH,
                    Some(DBUS_IFACE),
                    "UnInhibit",
                    &(cookie,),
                )
                .and_then(|r| r.body().deserialize());

            if let Err(e) = res {
                debug!("Failed to uninhibit screensaver cookie {}: {}", cookie, e);
            } else {
                debug!("Screensaver uninhibited cookie {}", cookie);
            }
        }
    }
}

/// RAII Guard that releases the D-Bus screensaver inhibition lock upon drop.
#[derive(Debug, Default)]
pub struct ScreensaverGuard {
    cookie: Option<u32>,
}

impl ScreensaverGuard {
    pub fn new(cookie: Option<u32>) -> Self {
        Self { cookie }
    }

    /// Return the active D-Bus cookie, if successfully acquired.
    pub fn cookie(&self) -> Option<u32> {
        self.cookie
    }

    /// Explicitly release the inhibit lock early without waiting for drop.
    pub fn release(&mut self) {
        if let Some(cookie) = self.cookie.take() {
            ScreensaverManager::uninhibit(cookie);
        }
    }
}

impl Drop for ScreensaverGuard {
    fn drop(&mut self) {
        self.release();
    }
}
