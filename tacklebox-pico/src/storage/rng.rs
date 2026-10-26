use core::cell::RefCell;

use embassy_rp::{adc::{Adc, Blocking, Channel}, clocks::RoscRng};
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use pbkdf2::sha2::{Digest, Sha256};
use rand_chacha::{ChaCha20Rng, rand_core::{Rng, SeedableRng}};
use zeroize::Zeroize;

use crate::storage::consts::SEED_SAMPLES;


static RNG: Mutex<CriticalSectionRawMutex, RefCell<Option<ChaCha20Rng>>> = Mutex::new(RefCell::new(None));

// Use a reading from sources to make a cryptographically secure seed
// - adc pin reading
// - embassy_time::Instant::now to get time since startup
// - RoscRng::next_u8 generates a non-cryptographic u8
// NOTE: adc_pin should be floating so that it gives the most noise on reads
// NOTE: embassy_time and RoscRng use separate timers to introduce jitter on reads
async fn generate_seed(
    adc: &mut Adc<'static, Blocking>,
    adc_pin: &mut Channel<'static>
) -> [u8; 32] {
    let mut hasher = Sha256::new();

    for _ in 0..SEED_SAMPLES {
        let adc_read = adc.blocking_read(adc_pin).expect("ConversionError");
        let adc_read = (adc_read & 0b111) as u8;
        let time_since_startup = embassy_time::Instant::now().as_ticks().to_ne_bytes();
        let rosc_rng = RoscRng::next_u8();

        hasher.update([adc_read, rosc_rng]);
        hasher.update(time_since_startup);

        embassy_time::Timer::after_millis(10).await;
    }

    hasher.finalize().into()
}


pub async fn init_rng(
    adc: &mut Adc<'static, Blocking>,
    adc_pin: &mut Channel<'static>
) {
    if RNG.lock(|cell| cell.borrow().is_some()) {
        return;
    }

    let mut seed = generate_seed(adc, adc_pin).await;
    let rng = ChaCha20Rng::from_seed(seed);
    seed.zeroize();

    RNG.lock(|cell| *cell.borrow_mut() = Some(rng) );
}

pub fn fill_bytes(dst: &mut[u8]) {
    RNG.lock(|cell| {
        cell
            .borrow_mut()
            .as_mut()
            .expect("RNG was not initialized")
            .fill_bytes(dst);
    });
}
