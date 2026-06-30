use crate::db::{
    delete_quota_snapshots_for_provider, load_quota_snapshots_from_db, save_quota_snapshot_to_db,
};
use crate::types::{QuotaCache, QuotaSnapshot};
use rusqlite::Connection;

pub(crate) fn load_cache_from_db(
    connection: &Connection,
    quota_cache: &QuotaCache,
) -> Result<(), String> {
    let snapshots = load_quota_snapshots_from_db(connection)?;
    let mut cache = quota_cache
        .snapshots
        .lock()
        .map_err(|_| "failed to lock quota cache".to_string())?;
    for snapshot in snapshots {
        cache.insert(snapshot.provider.clone(), snapshot);
    }
    Ok(())
}

pub(crate) fn write_snapshot_to_cache_and_db(
    connection: &Connection,
    quota_cache: &QuotaCache,
    snapshot: &QuotaSnapshot,
) -> Result<(), String> {
    // Only persist successful snapshots to DB; failed ones stay in memory only
    if snapshot.status == "ok" {
        save_quota_snapshot_to_db(connection, snapshot)?;
    }
    let mut cache = quota_cache
        .snapshots
        .lock()
        .map_err(|_| "failed to lock quota cache".to_string())?;
    cache.insert(snapshot.provider.clone(), snapshot.clone());
    Ok(())
}

pub(crate) fn remove_provider_from_cache_and_db(
    connection: &Connection,
    quota_cache: &QuotaCache,
    provider: &str,
) -> Result<(), String> {
    delete_quota_snapshots_for_provider(connection, provider)?;
    let mut cache = quota_cache
        .snapshots
        .lock()
        .map_err(|_| "failed to lock quota cache".to_string())?;
    cache.remove(provider);
    Ok(())
}

pub(crate) fn read_snapshots_from_cache(
    quota_cache: &QuotaCache,
) -> Result<Vec<QuotaSnapshot>, String> {
    let cache = quota_cache
        .snapshots
        .lock()
        .map_err(|_| "failed to lock quota cache".to_string())?;
    Ok(cache.values().cloned().collect())
}
