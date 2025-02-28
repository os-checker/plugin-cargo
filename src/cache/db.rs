use super::{CachedKey, CachedValue, Result};
use redb::{Database, TableDefinition};

const FILE: &str = "cache-plugin-cargo-v0.1.4.redb";
const TABLE: TableDefinition<CachedKey, CachedValue> = TableDefinition::new("plugin-cargo");

pub struct Db {
    db: Database,
}

impl Db {
    pub fn open() -> Result<Self> {
        Ok(Db {
            db: Database::create(FILE)?,
        })
    }

    pub fn load_cache(&self, key: &CachedKey) -> Result<Option<CachedValue>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        Ok(table.get(key)?.map(|val| val.value()))
    }

    // TODO: add a timestamp for each store
    pub fn store_cache(&self, key: &CachedKey, val: &CachedValue) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(key, val)?;
        }
        write_txn.commit()?;
        Ok(())
    }
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
