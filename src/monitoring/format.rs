use std::borrow::Cow;

const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

pub fn short_mint_str<'a>(mint: &'a str) -> Cow<'a, str> {
    match mint {
        WSOL_MINT => Cow::Borrowed("WSOL"),
        USDC_MINT => Cow::Borrowed("USDC"),
        _ if mint.len() <= 8 => Cow::Borrowed(mint),
        _ => Cow::Owned(format!("{}..{}", &mint[..4], &mint[mint.len() - 4..])),
    }
}
