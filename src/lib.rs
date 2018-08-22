use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ClockUpdater {
    clock: Arc<AtomicUsize>
}

impl ClockUpdater {
    fn increment(&mut self, frames: usize) {
        self.clock.store(self.clock.load(Ordering::Relaxed) + frames, Ordering::Release);
    }
}

pub struct ClockConsumer {
    clock: Arc<AtomicUsize>,
    rate: u32,
    tempo: f32
}

impl ClockConsumer {
    fn raw_frames(&self) -> usize {
        self.clock.load(Ordering::Acquire)
    }
    fn beat(&self) -> f32 {
        self.tempo / 60. * (self.raw_frames() as f32 / self.rate as f32)
    }
    fn beat_duration(&self) -> f32 {
        self.tempo / 60.
    }
}

impl Clone for ClockConsumer {
    fn clone(&self) -> ClockConsumer {
        ClockConsumer {
            rate: self.rate,
            tempo: self.tempo,
            clock: self.clock.clone()
        }
    }
}

pub fn audio_clock(tempo: f32, rate: u32) -> (ClockUpdater, ClockConsumer) {
    let c = Arc::new(AtomicUsize::new(0));
    (ClockUpdater { clock: c.clone() }, ClockConsumer { clock: c, tempo, rate })
}

#[cfg(test)]
mod tests {
    use audio_clock;

    #[test]
    fn it_works() {
        let (mut updater, consumer) = audio_clock(132.0, 44100);
        updater.increment(128);
        assert_eq!(consumer.raw_frames(), 128);
        assert_eq!(consumer.beat_duration(), 132.0 / 60.);
        assert_eq!(consumer.beat(), consumer.beat_duration() * (consumer.raw_frames() as f32 / 44100. ));
        updater.increment(64);
        assert_eq!(consumer.raw_frames(), 128 + 64);
        assert_eq!(consumer.beat_duration(), 132.0 / 60.);
        assert_eq!(consumer.beat(), consumer.beat_duration() * (consumer.raw_frames() as f32 / 44100. ));
        let second_consumer = consumer.clone();
        updater.increment(64);
        assert_eq!(consumer.raw_frames(), 128 + 64 + 64);
        assert_eq!(consumer.raw_frames(), second_consumer.raw_frames());
        assert_eq!(consumer.beat_duration(), second_consumer.beat_duration());
        assert_eq!(consumer.beat(), second_consumer.beat());
    }
}
