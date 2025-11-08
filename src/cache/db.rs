use super::{CachedKey, CachedValue, Result};
use redb::{Database, ReadableTableMetadata, TableDefinition, TableHandle};

const TABLE: TableDefinition<CachedKey, CachedValue> = TableDefinition::new("plugin-cargo");

fn db_file() -> String {
    const TAG_CACHE: &str = "TAG_CACHE";
    std::env::var(TAG_CACHE)
        .unwrap_or_else(|_| panic!("{TAG_CACHE:?} should be set to specify the db file path."))
}

pub struct Db {
    db: Database,
}

impl Db {
    pub fn open() -> Result<Self> {
        let db = Database::create(db_file())?;

        // create table if not present
        {
            let write_txn = db.begin_write()?;
            write_txn.open_table(TABLE)?;
            write_txn.commit()?;
        }
        {
            let read_txn = db.begin_read()?;
            info!(
                "len = {:?} list tables = {:?}",
                read_txn.open_table(TABLE)?.len(),
                read_txn
                    .list_tables()?
                    .map(|t| t.name().to_owned())
                    .collect::<Vec<_>>(),
            );
        }

        Ok(Db { db })
    }

    pub fn load_cache(&self, key: &CachedKey) -> Result<Option<CachedValue>> {
        info!("begin to load cache");
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let val = table.get(key)?.map(|val| val.value());
        val.as_ref().inspect(|_| info!("cache found"));
        Ok(val)
    }

    // TODO: add a timestamp for each store
    pub fn store_cache(&self, key: &CachedKey, val: &CachedValue) -> Result<()> {
        info!("begin to store cache");
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(key, val)?;
        }
        write_txn.commit()?;
        info!("cache written");
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
    assert_eq!(
        val.into_json(),
        table.get(&key)?.unwrap().value().into_json()
    );

    Ok(())
}

#[test]
fn test_db_list_table() -> Result<()> {
    let db = Database::create(db_file())?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    println!(
        "[begin_read] open_table: {table:?}\nlen = {:?}\nlist tables = {:?}",
        table.len(),
        read_txn
            .list_tables()?
            .map(|t| t.name().to_owned())
            .collect::<Vec<_>>(),
    );

    use redb::ReadableTable;

    let repo_with_err = table.iter()?.find_map(|t| {
        let (k, v) = t.ok()?;
        let (k, v) = (k.value(), v.value());
        (k.user == "shilei-massclouds" && k.repo == "arch_boot").then_some(v)
    });
    dbg!(repo_with_err);

    Ok(())
}
