# audio-clock

<a href="https://crates.io/crates/audio-clock">
    <img src="http://meritbadge.herokuapp.com/audio-clock" alt="crates.io">
</a>

Propagate a musical clock from a real-time audio thread to other threads:

```rust
let tempo = 132.2;
let sample_rate = 44100;
let (mut updater, consumer) = audio_clock(tempo, sample_rate);

// ... somehow send updater to the real-time audio thread.

// From an audio callback, increment the clock,
// from the real-time audio thread
updater.increment(frame_count);


// Somewhere else, use the clock:
println!("frame processed: ", consumer.raw_frames(), 128);
println!("beat count: ", consumer.beat());
println!("beat duration in seconds: ", consumer.beat_duration());

let other_consumer = consumer.clone();
// Send other_consumer to some other thread.

```

# Licence

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

