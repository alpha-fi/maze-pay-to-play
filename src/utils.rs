use near_sdk::{env, json_types::U128};

const DAY_MS: u64 = 24 * 3600 * 1000;

pub fn to_yocto_u8(value: u8) -> U128 {
    // Define 10^24 as the multiplier for yoctoNEAR precision
    let multiplier: u128 = 10u128.pow(24);
    // Convert u8 to u128 and multiply
    let result = (value as u128) * multiplier;
    // Wrap the result in U128 to represent as a NEAR compatible type
    U128::from(result)
}

pub fn safe_u128_to_u16(value: u128) -> Result<u16, &'static str> {
    if value <= u16::MAX as u128 {
        Ok(value as u16)
    } else {
        Err("Value exceeds the range of u16")
    }
}

pub fn get_today_day() -> u64 {
    env::block_timestamp_ms() / DAY_MS
}