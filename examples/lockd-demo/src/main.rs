use lockd::*;
wit_bindgen_rust::import!("../../wit/lockd.wit");

use anyhow::Result;

fn main() -> Result<()> {
    let lockd = get_lockd()?;

    let leaseless_lock_key = lock(&lockd, "leaseless-lock".as_bytes())?;
    println!("made leaseless lock with the following key: {}", std::str::from_utf8(&leaseless_lock_key)?);

    unlock(&lockd, &leaseless_lock_key)?;
    println!("unlocked leaseless lock!");

    println!("making lease of 60s...");
    let lease_id = grant_lease(&lockd, 60)?;

    let leasefull_lock_key = lock_with_lease(&lockd, "leasefull-lock".as_bytes(), lease_id)?;
    println!("made leasefull lock with the following key: {}", std::str::from_utf8(&leasefull_lock_key)?);

    unlock(&lockd, &leasefull_lock_key)?;
    println!("unlocked leasefull lock!");
    Ok(())
}

impl From<lockd::Error> for anyhow::Error {
    fn from(_: lockd::Error) -> Self {
        anyhow::anyhow!("lockd error")
    }
}
