use actix::clock::{interval_at, Duration, Instant};
use actix_rt::time::Interval;
use std::ops::Range;

pub trait Delay {
    fn set_delay(self) -> Interval;
}

impl Delay for u64 {
    fn set_delay(self) -> Interval {
        interval_at(
            Instant::now() + Duration::from_secs(self),
            Duration::from_secs(self),
        )
    }
}

impl Delay for Range<u64> {
    fn set_delay(self) -> Interval {
        fastrand::u64(self).set_delay()
    }
}
