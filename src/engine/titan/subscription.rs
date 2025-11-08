use std::collections::{BTreeMap, HashMap};
use std::net::IpAddr;

use solana_sdk::pubkey::Pubkey;

use crate::engine::runtime::strategy::MintSchedule;
use crate::strategy::types::TradePair;

#[derive(Debug, Clone)]
pub struct TitanSubscriptionPlanner;

impl TitanSubscriptionPlanner {
    pub fn build_plan(
        pairs: &[TradePair],
        schedules: &BTreeMap<Pubkey, MintSchedule>,
        ips: &[IpAddr],
    ) -> TitanSubscriptionPlan {
        TitanSubscriptionPlan::from_inputs(pairs, schedules, ips)
    }
}

#[derive(Debug, Clone)]
pub struct TitanSubscriptionPlan {
    assignments: HashMap<TitanBatchKey, IpAddr>,
}

impl TitanSubscriptionPlan {
    fn from_inputs(
        pairs: &[TradePair],
        schedules: &BTreeMap<Pubkey, MintSchedule>,
        ips: &[IpAddr],
    ) -> Self {
        if ips.is_empty() {
            return Self {
                assignments: HashMap::new(),
            };
        }

        let mut entries = collect_candidate_sizes(pairs, schedules);
        let capacity = ips.len().saturating_mul(2);
        if entries.is_empty() || capacity == 0 {
            return Self {
                assignments: HashMap::new(),
            };
        }

        entries.sort_by(|a, b| {
            a.pair
                .input_mint
                .cmp(&b.pair.input_mint)
                .then(a.pair.output_mint.cmp(&b.pair.output_mint))
                .then(a.amount.cmp(&b.amount))
        });
        entries.dedup_by(|a, b| a.pair == b.pair && a.amount == b.amount);

        let mut assignments = HashMap::new();
        let mut ip_slots = ips
            .iter()
            .copied()
            .map(|ip| (ip, 0usize))
            .collect::<Vec<_>>();
        let mut ip_index = 0usize;

        for entry in entries.into_iter() {
            if assignments.len() >= capacity {
                break;
            }

            while ip_index < ip_slots.len() && ip_slots[ip_index].1 >= 2 {
                ip_index += 1;
            }

            if ip_index >= ip_slots.len() {
                break;
            }

            let (ip, count) = ip_slots[ip_index];
            assignments.insert(TitanBatchKey::new(&entry.pair, entry.amount), ip);
            ip_slots[ip_index].1 = count + 1;
        }

        Self { assignments }
    }

    pub fn is_empty(&self) -> bool {
        self.assignments.is_empty()
    }

    pub fn len(&self) -> usize {
        self.assignments.len()
    }

    pub fn preferred_ip(&self, pair: &TradePair, amount: u64) -> Option<IpAddr> {
        if self.assignments.is_empty() {
            return None;
        }
        let key = TitanBatchKey::new(pair, amount);
        self.assignments.get(&key).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TitanBatchKey {
    pub base: Pubkey,
    pub quote: Pubkey,
    pub amount: u64,
}

impl TitanBatchKey {
    fn new(pair: &TradePair, amount: u64) -> Self {
        Self {
            base: pair.input_pubkey,
            quote: pair.output_pubkey,
            amount,
        }
    }
}

struct CandidateSize<'a> {
    pair: &'a TradePair,
    amount: u64,
}

fn collect_candidate_sizes<'a>(
    pairs: &'a [TradePair],
    schedules: &BTreeMap<Pubkey, MintSchedule>,
) -> Vec<CandidateSize<'a>> {
    let mut entries = Vec::new();
    for pair in pairs {
        let Some(schedule) = schedules.get(&pair.input_pubkey) else {
            continue;
        };
        let amounts = schedule.clone_amounts();
        for amount in amounts {
            if amount == 0 {
                continue;
            }
            entries.push(CandidateSize { pair, amount });
        }
    }
    entries
}
