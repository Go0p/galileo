#![allow(dead_code)]

use std::net::IpAddr;

use super::NetworkResult;

pub trait HttpClientFactory: Send + Sync + 'static {
    type Client: Clone + Send + Sync + 'static;

    fn make(&self, ip: IpAddr) -> NetworkResult<Self::Client>;
}

impl<T, F> HttpClientFactory for F
where
    T: Clone + Send + Sync + 'static,
    F: Fn(IpAddr) -> NetworkResult<T> + Send + Sync + 'static,
{
    type Client = T;

    fn make(&self, ip: IpAddr) -> NetworkResult<Self::Client> {
        (self)(ip)
    }
}
