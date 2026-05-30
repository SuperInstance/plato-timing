use serde::{Deserialize, Serialize};

/// Energy levels that drive BPM adaptation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl EnergyLevel {
    /// Returns the target BPM range (min, max) for this energy level.
    pub fn bpm_range(&self) -> (f64, f64) {
        match self {
            EnergyLevel::Low => (60.0, 72.0),
            EnergyLevel::Medium => (72.0, 88.0),
            EnergyLevel::High => (88.0, 108.0),
            EnergyLevel::Critical => (108.0, 120.0),
        }
    }

    /// Returns the midpoint target BPM for this energy level.
    pub fn target_bpm(&self) -> f64 {
        let (lo, hi) = self.bpm_range();
        (lo + hi) / 2.0
    }
}

/// Configuration for timing behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConfig {
    /// Minimum allowed BPM.
    pub bpm_min: f64,
    /// Maximum allowed BPM.
    pub bpm_max: f64,
    /// Swing factor 0.0–1.0. 0 = straight, 1 = max swing.
    pub swing_factor: f64,
    /// How many beats between downbeats.
    pub downbeat_interval: u32,
}

impl Default for TimingConfig {
    fn default() -> Self {
        TimingConfig {
            bpm_min: 60.0,
            bpm_max: 120.0,
            swing_factor: 0.5,
            downbeat_interval: 4,
        }
    }
}

/// A single beat event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beat {
    /// Sequential beat number (1-based).
    pub beat_number: u64,
    /// Timestamp in seconds since clock start.
    pub timestamp: f64,
    /// True on the first beat of every downbeat group.
    pub is_downbeat: bool,
    /// True on off-beats (not downbeat and not beat 1 of subgroup).
    pub is_offbeat: bool,
    /// Current energy level.
    pub energy: EnergyLevel,
}

/// Timing info for when a cadence should speak on a given beat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatTiming {
    /// Offset within the beat when speech should start (0.0 = on the beat).
    pub speak_at: f64,
    /// Suggested duration of the speech in seconds.
    pub duration: f64,
    /// Whether this beat is affected by swing timing.
    pub is_swing: bool,
}

/// Agent speaking cadence with timing preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cadence {
    /// Precise, on-the-beat communication. Speaks on downbeats.
    Technical,
    /// Relaxed, off-beat communication. Speaks on off-beats.
    Casual,
    /// Structured but flexible. Speaks on every other beat.
    Analytical,
    /// Rhythmic, expressive. Speaks on swing beats.
    Poetic,
}

impl Cadence {
    /// Returns the timing parameters for speaking on a given beat.
    pub fn timing_for_beat(&self, beat: &Beat) -> BeatTiming {
        let beat_duration = 60.0; // placeholder; caller can compute from BPM
        match self {
            Cadence::Technical => BeatTiming {
                speak_at: 0.0,
                duration: beat_duration * 0.8,
                is_swing: false,
            },
            Cadence::Casual => {
                if beat.is_offbeat {
                    BeatTiming {
                        speak_at: 0.0,
                        duration: beat_duration * 0.6,
                        is_swing: true,
                    }
                } else {
                    BeatTiming {
                        speak_at: beat_duration * 0.5,
                        duration: beat_duration * 0.4,
                        is_swing: false,
                    }
                }
            }
            Cadence::Analytical => {
                if beat.beat_number % 2 == 1 {
                    BeatTiming {
                        speak_at: 0.0,
                        duration: beat_duration * 0.7,
                        is_swing: false,
                    }
                } else {
                    BeatTiming {
                        speak_at: beat_duration * 0.25,
                        duration: beat_duration * 0.3,
                        is_swing: false,
                    }
                }
            }
            Cadence::Poetic => BeatTiming {
                speak_at: if beat.is_offbeat { beat_duration * 0.33 } else { 0.0 },
                duration: beat_duration * 0.9,
                is_swing: beat.is_offbeat,
            },
        }
    }

    /// Whether this cadence should speak on the given beat.
    pub fn should_speak(&self, beat: &Beat) -> bool {
        match self {
            Cadence::Technical => beat.is_downbeat,
            Cadence::Casual => beat.is_offbeat,
            Cadence::Analytical => beat.beat_number % 2 == 1,
            Cadence::Poetic => true,
        }
    }
}

/// A scheduled event that fires N beats before a downbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TMinusEvent {
    /// Unique event ID.
    pub id: u64,
    /// Callback identifier for the event.
    pub callback_id: u64,
    /// How many beats before the downbeat to fire.
    pub n_beats: u32,
    /// The beat number at which this event fires.
    pub fire_at_beat: u64,
    /// Whether this event has already fired.
    pub fired: bool,
}

/// The master tensor clock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorClock {
    config: TimingConfig,
    bpm: f64,
    target_bpm: f64,
    beat_count: u64,
    elapsed_time: f64,
    energy: EnergyLevel,
    tminus_events: Vec<TMinusEvent>,
    next_event_id: u64,
}

impl TensorClock {
    /// Create a new clock with the given configuration.
    pub fn new(config: TimingConfig) -> Self {
        let energy = EnergyLevel::Medium;
        let target_bpm = energy.target_bpm();
        TensorClock {
            bpm: target_bpm,
            target_bpm,
            config,
            beat_count: 0,
            elapsed_time: 0.0,
            energy,
            tminus_events: Vec::new(),
            next_event_id: 0,
        }
    }

    /// Current BPM.
    pub fn bpm(&self) -> f64 {
        self.bpm
    }

    /// Current beat count.
    pub fn beat_count(&self) -> u64 {
        self.beat_count
    }

    /// Current energy level.
    pub fn energy(&self) -> EnergyLevel {
        self.energy
    }

    /// Duration of one beat in seconds.
    pub fn beat_duration(&self) -> f64 {
        60.0 / self.bpm
    }

    /// Swing delay applied to off-beats.
    pub fn swing_delay(&self) -> f64 {
        self.config.swing_factor * (self.beat_duration() * 0.33)
    }

    /// How many beats until the next downbeat.
    pub fn beats_until_downbeat(&self) -> u32 {
        let interval = self.config.downbeat_interval;
        if self.beat_count == 0 {
            // Haven't started yet; first tick will be downbeat
            return 0;
        }
        // Beat 1 was downbeat, next is beat (interval+1)
        // beats_until = next_downbeat - current
        let position = ((self.beat_count - 1) % interval as u64) as u32;
        interval - position
    }

    /// Whether a given beat number is a downbeat.
    fn is_downbeat(&self, beat_number: u64) -> bool {
        beat_number % self.config.downbeat_interval as u64 == 1
    }

    /// Whether a given beat number is an off-beat.
    fn is_offbeat(&self, beat_number: u64) -> bool {
        !self.is_downbeat(beat_number)
    }

    /// Advance one beat and return the beat event.
    pub fn tick(&mut self) -> Beat {
        self.beat_count += 1;

        // Gradual BPM adaptation (±2 per beat)
        let diff = self.target_bpm - self.bpm;
        if diff.abs() > 0.01 {
            let step = diff.signum() * diff.abs().min(2.0);
            self.bpm = (self.bpm + step).clamp(self.config.bpm_min, self.config.bpm_max);
        }

        let beat_duration = self.beat_duration();
        self.elapsed_time += beat_duration;

        let beat = Beat {
            beat_number: self.beat_count,
            timestamp: self.elapsed_time,
            is_downbeat: self.is_downbeat(self.beat_count),
            is_offbeat: self.is_offbeat(self.beat_count),
            energy: self.energy,
        };

        // Check t-minus events
        let _next_downbeat = self.beat_count + self.beats_until_downbeat() as u64;
        for event in &mut self.tminus_events {
            if !event.fired && self.beat_count >= event.fire_at_beat {
                event.fired = true;
            }
        }

        beat
    }

    /// Adapt BPM to the given energy level.
    pub fn set_energy(&mut self, energy: EnergyLevel) {
        self.energy = energy;
        self.target_bpm = energy.target_bpm();
    }

    /// Schedule an event N beats before the next downbeat.
    pub fn schedule_tminus(&mut self, n_beats: u32, callback_id: u64) -> TMinusEvent {
        let beats_to_downbeat = self.beats_until_downbeat();
        let fire_at_beat = if beats_to_downbeat <= n_beats {
            // Not enough room; schedule for the following downbeat
            self.beat_count + beats_to_downbeat as u64 + self.config.downbeat_interval as u64 - n_beats as u64
        } else {
            self.beat_count + beats_to_downbeat as u64 - n_beats as u64
        };

        let event = TMinusEvent {
            id: self.next_event_id,
            callback_id,
            n_beats,
            fire_at_beat,
            fired: false,
        };
        self.next_event_id += 1;
        self.tminus_events.push(event.clone());
        event
    }

    /// Get all fired t-minus events since last check, clearing their fired status.
    pub fn drain_fired_events(&mut self) -> Vec<TMinusEvent> {
        let mut fired = Vec::new();
        for event in &mut self.tminus_events {
            if event.fired {
                fired.push(event.clone());
            }
        }
        self.tminus_events.retain(|e| !e.fired);
        fired
    }

    /// Get pending t-minus events.
    pub fn pending_events(&self) -> &[TMinusEvent] {
        &self.tminus_events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_creation_default() {
        let config = TimingConfig::default();
        let clock = TensorClock::new(config);
        assert_eq!(clock.beat_count(), 0);
        assert!(clock.bpm() >= 60.0 && clock.bpm() <= 120.0);
    }

    #[test]
    fn tick_advances_beat_count() {
        let mut clock = TensorClock::new(TimingConfig::default());
        assert_eq!(clock.beat_count(), 0);
        let b = clock.tick();
        assert_eq!(b.beat_number, 1);
        assert_eq!(clock.beat_count(), 1);
        clock.tick();
        assert_eq!(clock.beat_count(), 2);
    }

    #[test]
    fn bpm_stays_in_range() {
        let config = TimingConfig {
            bpm_min: 60.0,
            bpm_max: 120.0,
            ..Default::default()
        };
        let mut clock = TensorClock::new(config);
        for _ in 0..200 {
            clock.tick();
            assert!(clock.bpm() >= 60.0 && clock.bpm() <= 120.0);
        }
    }

    #[test]
    fn bpm_adapts_to_energy_low() {
        let mut clock = TensorClock::new(TimingConfig::default());
        clock.set_energy(EnergyLevel::Low);
        for _ in 0..100 {
            clock.tick();
        }
        let (lo, hi) = EnergyLevel::Low.bpm_range();
        assert!(clock.bpm() >= lo && clock.bpm() <= hi);
    }

    #[test]
    fn bpm_adapts_to_energy_critical() {
        let mut clock = TensorClock::new(TimingConfig::default());
        clock.set_energy(EnergyLevel::Critical);
        for _ in 0..100 {
            clock.tick();
        }
        let (lo, hi) = EnergyLevel::Critical.bpm_range();
        assert!(clock.bpm() >= lo && clock.bpm() <= hi);
    }

    #[test]
    fn bpm_changes_gradually() {
        let mut clock = TensorClock::new(TimingConfig::default());
        let initial_bpm = clock.bpm();
        clock.set_energy(EnergyLevel::Critical);
        // After one tick, BPM shouldn't jump more than 2
        clock.tick();
        let diff = (clock.bpm() - initial_bpm).abs();
        assert!(diff <= 2.01, "BPM jumped by {diff}, expected ≤ 2");
    }

    #[test]
    fn downbeat_detection() {
        let config = TimingConfig {
            downbeat_interval: 4,
            ..Default::default()
        };
        let mut clock = TensorClock::new(config);
        // Beats 1, 5, 9, 13 are downbeats
        for i in 1..=16 {
            let beat = clock.tick();
            let expected = i % 4 == 1;
            assert_eq!(
                beat.is_downbeat, expected,
                "Beat {} downbeat mismatch: expected {}, got {}",
                i, expected, beat.is_downbeat
            );
        }
    }

    #[test]
    fn beats_until_downbeat_calculation() {
        let config = TimingConfig {
            downbeat_interval: 4,
            ..Default::default()
        };
        let mut clock = TensorClock::new(config);
        // Before any ticks, beats_until_downbeat should be 0 (first tick is downbeat)
        assert_eq!(clock.beats_until_downbeat(), 0);
        clock.tick(); // beat 1 (downbeat)
        assert_eq!(clock.beats_until_downbeat(), 4);
        clock.tick(); // beat 2
        assert_eq!(clock.beats_until_downbeat(), 3);
        clock.tick(); // beat 3
        assert_eq!(clock.beats_until_downbeat(), 2);
        clock.tick(); // beat 4
        assert_eq!(clock.beats_until_downbeat(), 1);
    }

    #[test]
    fn tminus_scheduling_and_firing() {
        let config = TimingConfig {
            downbeat_interval: 8,
            ..Default::default()
        };
        let mut clock = TensorClock::new(config);
        clock.tick(); // beat 1 (downbeat)
        // Schedule 3 beats before next downbeat (beat 9)
        let event = clock.schedule_tminus(3, 42);
        assert_eq!(event.n_beats, 3);
        assert_eq!(event.callback_id, 42);
        assert!(!event.fired);
        // Fire should be at beat 6 (9 - 3)
        assert_eq!(event.fire_at_beat, 6);
    }

    #[test]
    fn tminus_event_fires() {
        let config = TimingConfig {
            downbeat_interval: 4,
            ..Default::default()
        };
        let mut clock = TensorClock::new(config);
        clock.tick(); // beat 1 (downbeat)
        let event = clock.schedule_tminus(2, 10);
        assert_eq!(event.fire_at_beat, 3);
        clock.tick(); // beat 2
        clock.tick(); // beat 3 → should fire
        let fired = clock.drain_fired_events();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].callback_id, 10);
    }

    #[test]
    fn swing_timing_nonzero() {
        let config = TimingConfig {
            swing_factor: 1.0,
            ..Default::default()
        };
        let clock = TensorClock::new(config);
        let swing = clock.swing_delay();
        assert!(swing > 0.0);
        // swing_delay = 1.0 * (beat_duration * 0.33)
        let expected = clock.beat_duration() * 0.33;
        assert!((swing - expected).abs() < 0.001);
    }

    #[test]
    fn swing_timing_zero() {
        let config = TimingConfig {
            swing_factor: 0.0,
            ..Default::default()
        };
        let clock = TensorClock::new(config);
        assert_eq!(clock.swing_delay(), 0.0);
    }

    #[test]
    fn cadence_technical_speaks_on_downbeats() {
        let config = TimingConfig {
            downbeat_interval: 4,
            ..Default::default()
        };
        let mut clock = TensorClock::new(config);
        for i in 1..=8 {
            let beat = clock.tick();
            let should = Cadence::Technical.should_speak(&beat);
            assert_eq!(should, i % 4 == 1, "Technical should_speak at beat {i}");
        }
    }

    #[test]
    fn cadence_casual_speaks_on_offbeats() {
        let config = TimingConfig {
            downbeat_interval: 4,
            ..Default::default()
        };
        let mut clock = TensorClock::new(config);
        for i in 1..=8 {
            let beat = clock.tick();
            let should = Cadence::Casual.should_speak(&beat);
            assert_eq!(should, i % 4 != 1, "Casual should_speak at beat {i}");
        }
    }

    #[test]
    fn cadence_poetic_speaks_always() {
        let mut clock = TensorClock::new(TimingConfig::default());
        for _ in 1..=8 {
            let beat = clock.tick();
            assert!(Cadence::Poetic.should_speak(&beat));
        }
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = TimingConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TimingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.bpm_min, deserialized.bpm_min);
        assert_eq!(config.bpm_max, deserialized.bpm_max);
        assert_eq!(config.swing_factor, deserialized.swing_factor);
        assert_eq!(config.downbeat_interval, deserialized.downbeat_interval);
    }

    #[test]
    fn clock_serialization_roundtrip() {
        let mut clock = TensorClock::new(TimingConfig::default());
        clock.tick();
        clock.tick();
        clock.set_energy(EnergyLevel::High);
        let json = serde_json::to_string(&clock).unwrap();
        let deserialized: TensorClock = serde_json::from_str(&json).unwrap();
        assert_eq!(clock.beat_count(), deserialized.beat_count());
        assert_eq!(clock.energy(), deserialized.energy());
    }

    #[test]
    fn rapid_energy_changes_stay_in_range() {
        let mut clock = TensorClock::new(TimingConfig::default());
        let energies = [
            EnergyLevel::Critical,
            EnergyLevel::Low,
            EnergyLevel::High,
            EnergyLevel::Low,
            EnergyLevel::Critical,
            EnergyLevel::Medium,
        ];
        for &e in &energies {
            clock.set_energy(e);
            for _ in 0..5 {
                clock.tick();
                assert!(
                    clock.bpm() >= 60.0 && clock.bpm() <= 120.0,
                    "BPM {} out of range",
                    clock.bpm()
                );
            }
        }
    }
}
