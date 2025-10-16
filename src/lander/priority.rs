use crate::config::LanderSettings;

const DEFAULT_PRICE_MICRO_LAMPORTS: u64 = 413;

pub fn compute_priority_fee_micro_lamports(
    settings: &LanderSettings,
    compute_unit_limit: u32,
) -> u64 {
    if compute_unit_limit == 0 {
        return DEFAULT_PRICE_MICRO_LAMPORTS;
    }

    let priority_lamports = match settings.fixed_priority_fee {
        Some(fixed) => Some(fixed),
        None => settings.random_priority_fee_range.first().copied(),
    };

    priority_lamports
        .and_then(|fee_lamports| {
            fee_lamports
                .checked_mul(1_000_000)
                .map(|micros| micros / u64::from(compute_unit_limit.max(1)))
        })
        .map(|price| price.max(1))
        .unwrap_or(DEFAULT_PRICE_MICRO_LAMPORTS)
}
