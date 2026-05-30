use plato_timing::{Cadence, EnergyLevel, TensorClock, TimingConfig};

fn main() {
    let config = TimingConfig::default();
    let mut clock = TensorClock::new(config);

    println!("PLATO Tensor Clock initialized");
    println!("BPM: {}", clock.bpm());

    for _ in 0..16 {
        let beat = clock.tick();
        println!(
            "Beat {} | downbeat: {} | offbeat: {} | energy: {:?}",
            beat.beat_number, beat.is_downbeat, beat.is_offbeat, beat.energy
        );
    }

    clock.set_energy(EnergyLevel::High);
    println!("\nEnergy set to High, BPM: {}", clock.bpm());

    for _ in 0..8 {
        let beat = clock.tick();
        println!(
            "Beat {} | downbeat: {} | BPM: {:.1}",
            beat.beat_number, beat.is_downbeat, clock.bpm()
        );
    }
}
