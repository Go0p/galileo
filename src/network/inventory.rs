#![allow(dead_code)]

use std::collections::HashSet;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::Arc;

use if_addrs::{IfAddr, get_if_addrs};

use super::{IpSlot, IpSlotKind, NetworkError, NetworkResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpSource {
    Manual,
    Auto,
}

#[derive(Debug, Clone)]
pub struct IpInventoryConfig {
    pub enable_multiple_ip: bool,
    pub manual_ips: Vec<IpAddr>,
    pub blacklist: Vec<IpAddr>,
    pub allow_loopback: bool,
}

impl Default for IpInventoryConfig {
    fn default() -> Self {
        Self {
            enable_multiple_ip: false,
            manual_ips: Vec::new(),
            blacklist: Vec::new(),
            allow_loopback: false,
        }
    }
}

pub struct IpInventoryBuilder {
    config: IpInventoryConfig,
}

impl IpInventoryBuilder {
    pub fn new() -> Self {
        Self {
            config: IpInventoryConfig::default(),
        }
    }

    pub fn enable_multiple_ip(mut self, enable: bool) -> Self {
        self.config.enable_multiple_ip = enable;
        self
    }

    pub fn manual_ips<I>(mut self, ips: I) -> Self
    where
        I: IntoIterator<Item = IpAddr>,
    {
        self.config.manual_ips = ips.into_iter().collect();
        self
    }

    pub fn blacklist<I>(mut self, ips: I) -> Self
    where
        I: IntoIterator<Item = IpAddr>,
    {
        self.config.blacklist = ips.into_iter().collect();
        self
    }

    pub fn allow_loopback(mut self, allow: bool) -> Self {
        self.config.allow_loopback = allow;
        self
    }

    pub fn build(self) -> NetworkResult<IpInventory> {
        IpInventory::new(self.config)
    }
}

pub struct IpInventory {
    slots: Vec<Arc<IpSlot>>,
    source: IpSource,
}

impl IpInventory {
    pub fn new(config: IpInventoryConfig) -> NetworkResult<Self> {
        let blacklist: HashSet<IpAddr> = config.blacklist.into_iter().collect();
        let manual = sanitize_manual_ips(&config.manual_ips, &blacklist, config.allow_loopback);
        let using_manual = !manual.is_empty();

        let mut discovered = if using_manual {
            manual
        } else {
            discover_interfaces(config.allow_loopback, &blacklist)?
        };

        if discovered.is_empty() {
            return Err(NetworkError::NoEligibleIp);
        }

        if !config.enable_multiple_ip {
            discovered.truncate(1);
        }

        let slots = discovered
            .into_iter()
            .map(|ip| Arc::new(IpSlot::new(ip, IpSlotKind::Ephemeral)))
            .collect::<Vec<_>>();

        Ok(Self {
            slots,
            source: if using_manual {
                IpSource::Manual
            } else {
                IpSource::Auto
            },
        })
    }

    pub fn builder() -> IpInventoryBuilder {
        IpInventoryBuilder::new()
    }

    pub fn source(&self) -> IpSource {
        self.source
    }

    pub fn total(&self) -> usize {
        self.slots.len()
    }

    pub fn slots(&self) -> &[Arc<IpSlot>] {
        &self.slots
    }

    pub fn into_slots(self) -> Vec<Arc<IpSlot>> {
        self.slots
    }
}

fn sanitize_manual_ips(
    manual: &[IpAddr],
    blacklist: &HashSet<IpAddr>,
    allow_loopback: bool,
) -> Vec<IpAddr> {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for ip in manual {
        if blacklist.contains(ip) {
            continue;
        }
        if !allow_loopback && ip.is_loopback() {
            continue;
        }
        if is_unsuitable(ip) {
            continue;
        }
        if !seen.insert(*ip) {
            continue;
        }
        unique.push(*ip);
    }
    unique
}

fn discover_interfaces(
    allow_loopback: bool,
    blacklist: &HashSet<IpAddr>,
) -> NetworkResult<Vec<IpAddr>> {
    let mut results = Vec::new();
    for iface in get_if_addrs().map_err(NetworkError::InterfaceDiscovery)? {
        let ip = match iface.addr {
            IfAddr::V4(v4) => IpAddr::V4(v4.ip),
            IfAddr::V6(v6) => IpAddr::V6(v6.ip),
        };

        if !allow_loopback && ip.is_loopback() {
            continue;
        }

        if is_unsuitable(&ip) {
            continue;
        }

        if blacklist.contains(&ip) {
            continue;
        }

        results.push(ip);
    }

    results.sort();
    results.dedup();

    Ok(results)
}

fn is_unsuitable(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.octets()[0] == 169 && (v4.octets()[1] == 254)
        }
        IpAddr::V6(v6) => {
            v6.is_unspecified()
                || v6.is_multicast()
                || is_unicast_link_local(v6)
                || is_unicast_unique_local(v6)
        }
    }
}

fn is_unicast_link_local(addr: &Ipv6Addr) -> bool {
    addr.segments()[0] & 0xffc0 == 0xfe80
}

fn is_unicast_unique_local(addr: &Ipv6Addr) -> bool {
    addr.segments()[0] & 0xfe00 == 0xfc00
}
