use soroban_sdk::{Env, IntoVal, Val};

pub const LEDGERS_PER_DAY: u32 = (24 * 3600) / 5;
pub const INSTANCE_TTL_THRESHOLD: u32 = 30 * LEDGERS_PER_DAY;
pub const INSTANCE_TTL_EXTEND_TO: u32 = 60 * LEDGERS_PER_DAY;
pub const PERSISTENT_TTL_THRESHOLD: u32 = 30 * LEDGERS_PER_DAY;
pub const PERSISTENT_TTL_EXTEND_TO: u32 = 60 * LEDGERS_PER_DAY;

/// Extends the TTL of the contract instance if it falls below the threshold.
///
/// # Arguments
/// * `env` - The environment reference
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

/// Extends the TTL of a persistent storage entry if it falls below the threshold.
///
/// # Arguments
/// * `env` - The environment reference
/// * `key` - The storage key to extend TTL for
///
/// # Type Parameters
/// * `K` - The key type that implements IntoVal<Env, Val>
pub fn extend_persistent_ttl<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}
