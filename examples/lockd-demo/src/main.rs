use std::{
    thread,
    time::{Duration, SystemTime},
};

use lockd::*;
wit_bindgen_rust::import!("../../wit/lockd.wit");

use anyhow::Result;

fn main() -> Result<()> {
    let lockd = get_lockd()?;

    println!("trying to acquire a lock with 5s time to live");
    let mut now = SystemTime::now();
    let _lock_with_time_to_live =
        lock_with_time_to_live(lockd, "lock_with_time_to_live".as_bytes(), 5)?;
    println!(
        "managed to acquire lock after {:?}s, this lock will be unlocked after 5s",
        now.elapsed()?.as_secs()
    );

    println!("trying to acquire a lock with no specific time to live");
    now = SystemTime::now();
    let lock_with_no_time_to_live = lock(lockd, "lock_with_no_time_to_live".as_bytes())?;
    println!(
        "managed to acquire lock after {:?}s",
        now.elapsed()?.as_secs()
    );
    println!("pretend we are doing work by sleeping for 10s...");
    thread::sleep(Duration::from_secs(10));
    println!("unlocked the lock we just acquired!");
    unlock(lockd, &lock_with_no_time_to_live)?;

    Ok(())
}

impl From<lockd::Error> for anyhow::Error {
    fn from(_: lockd::Error) -> Self {
        anyhow::anyhow!("lockd error")
    }
}
