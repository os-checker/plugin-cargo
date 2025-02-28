use super::{CachedKey, CachedValue, Result};
use redb::{Database, TableDefinition};

const FILE: &str = "cache-plugin-cargo-v0.1.4.redb";
const TABLE: TableDefinition<CachedKey, CachedValue> = TableDefinition::new("plugin-cargo");

pub fn read_cache(key: &CachedKey) -> Result<Option<CachedValue>> {
    let db = Database::create(FILE)?;
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    Ok(table.get(key)?.map(|val| val.value()))
}

#[test]
fn test_os_checker_test_suite() -> Result<()> {
    const FILE: &str = "cache-plugin-cargo-v-test.redb";

    let (key, val) = super::gen_cache("os-checker/os-checker-test-suite")?;

    let db = Database::create(FILE)?;

    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(&key, &val)?;
    }
    write_txn.commit()?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    assert_eq!(val.json(), table.get(&key)?.unwrap().value().json());

    Ok(())
}
