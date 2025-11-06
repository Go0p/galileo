use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::config::LanderJitoUuidConfig;

#[derive(Clone)]
pub(crate) struct UuidPool {
    entries: Vec<UuidEntry>,
    cursor: usize,
}

impl UuidPool {
    pub fn new(configs: &[LanderJitoUuidConfig]) -> Option<Self> {
        let mut entries = Vec::new();
        for cfg in configs {
            let uuid = cfg.uuid.trim();
            if uuid.is_empty() {
                continue;
            }
            let limiter = cfg.rate_limit.and_then(|limit| {
                if limit == 0 {
                    None
                } else {
                    Some(RateLimiter::new(limit))
                }
            });
            entries.push(UuidEntry {
                uuid: uuid.to_string(),
                limiter,
                sequence: 0,
            });
        }

        if entries.is_empty() {
            None
        } else {
            Some(Self { entries, cursor: 0 })
        }
    }

    pub fn next_ticket(&mut self) -> UuidTicketOutcome {
        if self.entries.is_empty() {
            return UuidTicketOutcome::Empty;
        }

        let mut smallest = None;
        let len = self.entries.len();
        for _ in 0..len {
            let entry = &mut self.entries[self.cursor];
            self.cursor = (self.cursor + 1) % len;
            match entry.try_next() {
                EntryOutcome::Ticket(ticket) => return UuidTicketOutcome::Ticket(ticket),
                EntryOutcome::RateLimited(duration) => {
                    smallest = Some(
                        smallest
                            .map(|current: Duration| current.min(duration))
                            .unwrap_or(duration),
                    );
                }
            }
        }

        if let Some(duration) = smallest {
            UuidTicketOutcome::RateLimited { cooldown: duration }
        } else {
            UuidTicketOutcome::Empty
        }
    }
}

#[derive(Clone)]
struct UuidEntry {
    uuid: String,
    limiter: Option<RateLimiter>,
    sequence: u64,
}

enum EntryOutcome {
    Ticket(UuidTicket),
    RateLimited(Duration),
}

impl UuidEntry {
    fn try_next(&mut self) -> EntryOutcome {
        if let Some(limiter) = &mut self.limiter {
            if limiter.try_acquire() {
                self.sequence = self.sequence.wrapping_add(1);
                EntryOutcome::Ticket(UuidTicket {
                    uuid: self.uuid.clone(),
                    bundle_id: format!("{}-{}", self.uuid, self.sequence),
                })
            } else {
                EntryOutcome::RateLimited(limiter.time_until_ready())
            }
        } else {
            self.sequence = self.sequence.wrapping_add(1);
            EntryOutcome::Ticket(UuidTicket {
                uuid: self.uuid.clone(),
                bundle_id: format!("{}-{}", self.uuid, self.sequence),
            })
        }
    }
}

#[derive(Clone)]
struct RateLimiter {
    capacity: u64,
    rate: f64,
    tokens: f64,
    last: Instant,
}

impl RateLimiter {
    fn new(limit: u64) -> Self {
        let now = Instant::now();
        Self {
            capacity: limit,
            rate: limit as f64,
            tokens: limit as f64,
            last: now,
        }
    }

    fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn time_until_ready(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let deficit = 1.0 - self.tokens.max(0.0);
            if self.rate <= f64::EPSILON {
                Duration::from_secs(1)
            } else {
                Duration::from_secs_f64((deficit / self.rate).max(0.0))
            }
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last.elapsed().as_secs_f64();
        if elapsed <= 0.0 {
            return;
        }
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity as f64);
        self.last = Instant::now();
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UuidTicket {
    pub uuid: String,
    pub bundle_id: String,
}

impl UuidTicket {
    pub fn options_value(&self) -> Value {
        json!({
            "bundleId": self.bundle_id,
            "uuid": self.uuid,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) enum UuidTicketOutcome {
    Ticket(UuidTicket),
    RateLimited { cooldown: Duration },
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_pool_respects_rate_limit() {
        let configs = vec![LanderJitoUuidConfig {
            uuid: "1234".to_string(),
            rate_limit: Some(1),
        }];
        let mut pool = UuidPool::new(&configs).expect("pool");

        match pool.next_ticket() {
            UuidTicketOutcome::Ticket(ticket) => {
                assert_eq!(ticket.uuid, "1234");
            }
            other => panic!("expected ticket, got {other:?}"),
        }

        match pool.next_ticket() {
            UuidTicketOutcome::RateLimited { .. } => {}
            other => panic!("expected rate limited, got {other:?}"),
        }
    }
}
