#![allow(dead_code)]

use std::net::IpAddr;

use dashmap::{DashMap, mapref::entry::Entry};

use super::{HttpClientFactory, NetworkResult};

pub struct IpBoundClientPool<F>
where
    F: HttpClientFactory,
{
    factory: F,
    clients: DashMap<IpAddr, F::Client>,
}

impl<F> IpBoundClientPool<F>
where
    F: HttpClientFactory,
{
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            clients: DashMap::new(),
        }
    }

    pub fn get_or_create(&self, ip: IpAddr) -> NetworkResult<F::Client> {
        if let Some(existing) = self.clients.get(&ip) {
            return Ok(existing.clone());
        }

        let client = self.factory.make(ip)?;
        match self.clients.entry(ip) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                entry.insert(client.clone());
                Ok(client)
            }
        }
    }

    pub fn remove(&self, ip: &IpAddr) {
        self.clients.remove(ip);
    }

    pub fn len(&self) -> usize {
        self.clients.len()
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    pub fn clear(&self) {
        self.clients.clear();
    }
}
