extern crate atomic;

use atomic::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct InnerClock {
    beat: Atomic<f32>,   // w on audio thread, r+w on all thread
    frames: AtomicUsize, // w on audio thread, r+w on all threads
    tempo: Atomic<f32>,  // r+w on audio thread, r on all threads
    rate: u32,           // const
}

pub struct ClockUpdater {
    inner: Arc<InnerClock>,
}

impl ClockUpdater {
    pub fn increment(&mut self, frames: usize) {
        self.inner.frames.fetch_add(frames, Ordering::Release);
        self.inner.beat.store(
            self.inner.beat.load(Ordering::Acquire)
                + self.inner.tempo.load(Ordering::Acquire) / 60. * frames as f32
                    / self.inner.rate as f32,
            Ordering::Release,
        );
    }
    pub fn set_tempo(&mut self, new_tempo: f32) {
        self.inner.tempo.store(new_tempo, Ordering::Release);
    }
}

pub struct ClockConsumer {
    inner: Arc<InnerClock>,
}

impl ClockConsumer {
    pub fn frames(&self) -> usize {
        self.inner.frames.load(Ordering::Acquire)
    }
    pub fn beat(&self) -> f32 {
        self.inner.beat.load(Ordering::Acquire)
    }
    pub fn beat_duration(&self) -> f32 {
        self.inner.tempo.load(Ordering::Acquire) / 60.
    }
}

impl Clone for ClockConsumer {
    fn clone(&self) -> ClockConsumer {
        ClockConsumer {
            inner: self.inner.clone(),
        }
    }
}

pub fn audio_clock(tempo: f32, rate: u32) -> (ClockUpdater, ClockConsumer) {
    let c = Arc::new(InnerClock {
        beat: Atomic::new(0.),
        frames: AtomicUsize::new(0),
        tempo: Atomic::new(tempo),
        rate,
    });
    (
        ClockUpdater { inner: c.clone() },
        ClockConsumer { inner: c },
    )
}

#[cfg(test)]
mod tests {
    use audio_clock;
    use std::thread;

    #[test]
    fn it_works() {
        let (mut updater, consumer) = audio_clock(132.0, 44100);
        updater.increment(128);
        assert_eq!(consumer.frames(), 128);
        assert_eq!(consumer.beat_duration(), 132.0 / 60.);
        assert_eq!(
            consumer.beat(),
            consumer.beat_duration() * (consumer.frames() as f32 / 44100.)
        );
        updater.increment(64);
        assert_eq!(consumer.frames(), 128 + 64);
        assert_eq!(consumer.beat_duration(), 132.0 / 60.);
        assert_eq!(
            consumer.beat(),
            consumer.beat_duration() * (consumer.frames() as f32 / 44100.)
        );
        let second_consumer = consumer.clone();
        updater.increment(64);
        assert_eq!(consumer.frames(), 128 + 64 + 64);
        assert_eq!(consumer.frames(), second_consumer.frames());
        assert_eq!(consumer.beat_duration(), second_consumer.beat_duration());
        assert_eq!(consumer.beat(), second_consumer.beat());
        updater.set_tempo(130.);
        assert_eq!(consumer.frames(), 128 + 64 + 64);
        assert_eq!(consumer.frames(), second_consumer.frames());
        assert_eq!(consumer.beat_duration(), second_consumer.beat_duration());
        assert_eq!(consumer.beat(), second_consumer.beat());
        updater.increment(64);
        assert_eq!(consumer.frames(), 128 + 64 + 64 + 64);
        assert_eq!(consumer.frames(), second_consumer.frames());
        assert_eq!(consumer.beat_duration(), second_consumer.beat_duration());
        assert_eq!(consumer.beat(), second_consumer.beat());
        match thread::spawn(move || {
            updater.increment(128);
        })
        .join()
        {
            Ok(_) => {
                assert_eq!(consumer.frames(), 128 + 64 + 64 + 64 + 128);
                assert_eq!(consumer.frames(), second_consumer.frames());
                assert_eq!(consumer.beat_duration(), second_consumer.beat_duration());
                assert_eq!(consumer.beat(), second_consumer.beat());
            }
            Err(_) => {
                panic!("!?");
            }
        };
    }
}
