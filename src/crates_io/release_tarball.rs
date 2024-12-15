use super::IndexFile;
use crate::repo::local_base_dir;
use cargo_metadata::semver::Version;
use eyre::ContextCompat;
use plugin::prelude::*;
use std::os::linux::fs::MetadataExt;

#[derive(Debug)]
pub struct TarballInfo {
    /// size in bytes
    pub size: u64,
    pub modified: Timestamp,
}

impl TarballInfo {
    fn new(tarball: &Utf8Path) -> Result<Self> {
        let meta = std::fs::metadata(tarball)?;
        let size = meta.st_size();
        let modified = Timestamp::from_second(meta.st_mtime())?;
        Ok(Self { size, modified })
    }
}

fn download_tarball(pkg: &str, version: &Version) -> Result<Utf8PathBuf> {
    const TARBALL: &str = "download";

    let url = url(pkg, version);
    info!("wget {url}");

    let dir = local_base_dir();

    duct::cmd!("wget", url, "-O", TARBALL)
        .dir(dir)
        .stdout_null()
        .stderr_null()
        .run()?;

    let tarball = dir.join(TARBALL);
    Ok(tarball)
}

fn url(pkg: &str, version: &Version) -> String {
    // wget https://static.crates.io/crates/os-checker/0.4.1/download
    // for further use: tar xf download && cd os-checker-0.4.1
    const PREFIX: &str = "https://static.crates.io/crates";
    format!("{PREFIX}/{pkg}/{version}/download")
}

fn get_last_release_info(pkg: &str, version: &Version) -> Result<TarballInfo> {
    let tarball = download_tarball(pkg, version)?;
    TarballInfo::new(&tarball)
}

impl IndexFile {
    pub fn get_last_release_info(&mut self) -> Result<()> {
        let last = self.data.last();
        let last = last.with_context(|| "index file is empty")?;
        self.tarball = Some(get_last_release_info(&self.pkg, &last.vers)?);
        Ok(())
    }

    pub fn last_release_size_and_time(&self) -> Option<(u64, Timestamp)> {
        self.tarball
            .as_ref()
            .map(|tarball| (tarball.size, tarball.modified))
    }
}

#[test]
fn test_tarball_info() -> Result<()> {
    // test on Cargo.toml instead of tarball for now
    // dbg!(TarballInfo::new("Cargo.toml".into())?);

    dbg!(get_last_release_info("os-checker", &Version::new(0, 4, 1))?);

    Ok(())
}
