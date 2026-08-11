use std::process;

use crate::agent::build_agent;
use crate::protection::Protection;

impl Protection {
    pub(crate) fn check_http(&self) {
        let agent = build_agent(self.timeout_secs);
        match agent.get(&self.http_url).call() {
            Ok(r) if r.status() == 200 => process::exit(0),
            _ => {}
        }
    }
}
