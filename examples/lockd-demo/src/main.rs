use std::time::SystemTime;

use lockd::*;
wit_bindgen_rust::import!("../../wit/lockd.wit");

use anyhow::Result;

fn main() -> Result<()> {
    let lockd = get_lockd()?;

    let lock_with_time_to_live =
        lock_with_time_to_live(lockd, "lock_with_time_to_live".as_bytes(), 10)?;
    println!(
        "made a lock with 10s to live that has the following key: {}",
        std::str::from_utf8(&lock_with_time_to_live)?
    );

    println!("trying to acquire the previously created lock...");
    let now = SystemTime::now();
    let early_acquire_lock = lock(lockd, "lock_with_time_to_live".as_bytes())?;
    println!(
        "managed to acquire lock after {:?}s, that lock's time to live was 10s",
        now.elapsed()?.as_secs()
    );
    unlock(lockd, &early_acquire_lock)?;
    println!("unlocked the lock we just acquried!");

    let lock_with_no_time_to_live = lock(lockd, "lock_with_no_time_to_live".as_bytes())?;
    println!(
        "made a lock with no specific time to live that has the following key: {}",
        std::str::from_utf8(&lock_with_no_time_to_live)?
    );
    unlock(lockd, &lock_with_no_time_to_live)?;
    println!("unlocked the lock we just acquried!");

    Ok(())
}

impl From<lockd::Error> for anyhow::Error {
    fn from(_: lockd::Error) -> Self {
        anyhow::anyhow!("lockd error")
    }
}
