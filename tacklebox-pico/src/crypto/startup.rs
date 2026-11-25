use at_cryptoauth::{AtCaClient, memory::Slot};
use embassy_rp::{Peri, gpio::{Level, OutputOpenDrain}, i2c::{Async, I2c}, peripherals::{I2C1, PIN_2, PIN_3}};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer};

use crate::crypto::errors::{CryptoStartupError, CryptoStartupResult};



// Set to true ONLY on a build meant to provision a chip.
// Locking the config zone and the data zone is permanent.
const ALLOW_PERMANENT_LOCKS: bool = true;

pub const AES_KEY_SLOT: Slot = Slot::PrivateKey00;
type SdaPin<'d> = Peri<'d, PIN_2>;
type SclPin<'d> = Peri<'d, PIN_3>;
type I2cAsync<'a> = I2c<'a, I2C1, Async>;
pub type AtCaClientAsync<'a> = AtCaClient<CriticalSectionRawMutex, I2cAsync<'a>>;


const ATECC_ADDR: u8 = 0x60;
// opcode + p1 + p2(2) + (NOT INCLUDING DATA) + crc(2)
const CMD_NECCESSARY_DATA_LEN: usize = 7;
const MAX_CMD_DATA_LEN: usize = 32;
// REGISTER + opcode + p1 + p2(2) + DATA + crc(2)
const MAX_CMD_LEN: usize = 1 + CMD_NECCESSARY_DATA_LEN + MAX_CMD_DATA_LEN;

// Word-address register values (first byte after the I2C address)
const CMD_REG: u8 = 0x03;
const SLEEP_REG: u8 = 0x01;

// Opcodes
const OP_READ: u8 = 0x02;
const OP_WRITE: u8 = 0x12;
const OP_LOCK: u8 = 0x17;
const OP_RANDOM: u8 = 0x1B;

// Param1 values
const READ_P1_CFG_WORD: u8 = 0x00;   // config zone, 4 bytes
const READ_P1_CFG_BLOCK: u8 = 0x80;  // config zone, 32 bytes
const WRITE_P1_CFG_WORD: u8 = 0x00;  // config zone, 4 bytes
const WRITE_P1_DATA_BLOCK: u8 = 0x82; // data zone, 32 bytes
const LOCK_P1_CONFIG: u8 = 0x00;     // lock config zone, verify CRC in Param2
const LOCK_P1_DATA_NO_CRC: u8 = 0x81; // lock data + OTP zones, skip CRC check
const RANDOM_P1_DEFAULT: u8 = 0x00;

// Execution times (max from datasheet, plus margin)
const READ_EXEC: Duration = Duration::from_millis(2);
const WRITE_EXEC: Duration = Duration::from_millis(30);
const LOCK_EXEC: Duration = Duration::from_millis(35);
const RANDOM_EXEC: Duration = Duration::from_millis(25);

// Config zone layout
const LOCK_CFG_WORD: u16 = 21;        // bytes 84..87: ..., LockValue (86), LockConfig (87)
const AES_ENABLE_BYTE: usize = 13;
const SLOT_CONFIG_BASE: u16 = 20;
const KEY_CONFIG_BASE: u16 = 96;
const UNLOCKED: u8 = 0x55;

// KeyConfig bits
const KC_LOW_FIELDS_MASK: u16 = 0b1_1111; // Private, PubInfo, KeyType
const KC_KEYTYPE_AES: u16 = 6 << 2;

// SlotConfig bits
const SC_IS_SECRET: u16 = 1 << 7;
const SC_WRITE_NEVER: u16 = 0b1000 << 12;
const SC_AES_SLOT: u16 = SC_IS_SECRET | SC_WRITE_NEVER;



// Setting sda to low for ~80μs starts wake procedure and we wait 100μs to be safe
// Wake procedure takes ~1.5ms and we wait 3ms to be safe
async fn wake_and_initialize(sda_pin: SdaPin<'_>) {
    let mut sda_wake = OutputOpenDrain::new(
        sda_pin,
        Level::High
    );
    sda_wake.set_low();
    Timer::after(Duration::from_micros(100)).await;
    sda_wake.set_low();
    Timer::after(Duration::from_millis(3)).await;
}

// Writing to SLEEP_REG puts chip to sleep while waiting on more commands
async fn sleep_chip(i2c: &mut I2cAsync<'_>) -> CryptoStartupResult<()> {
    i2c.write_async(ATECC_ADDR, [SLEEP_REG]).await?;
    Ok(())
}

// Generates a CRC-16/ARC sum for a given buffer
fn crc16_cryptoauth(data: &[u8]) -> [u8; 2] {
    let mut crc: u16 = 0;
    for &b in data {
        for i in 0..8 {
            let data_bit = (b >> i) & 1; // LSB first
            let crc_bit = (crc >> 15) as u8;
            crc <<= 1;
            if data_bit != crc_bit {
                crc ^= 0x8005;
            }
        }
    }
    crc.to_le_bytes()
}


async fn execute_with_data(
    i2c: &mut I2cAsync<'_>,
    opcode: u8,
    p1: u8,
    p2: u16,
    data: &[u8], // 0, 4 or 32 bytes
    exec: Duration,
    resp: &mut [u8],
) -> CryptoStartupResult<usize> {
    assert!(data.len() <= MAX_CMD_DATA_LEN);


    let count = CMD_NECCESSARY_DATA_LEN + data.len();
    let mut cmd = [0u8; MAX_CMD_LEN];
    cmd[0] = CMD_REG;
    cmd[1] = count as u8;
    cmd[2] = opcode;
    cmd[3] = p1;
    cmd[4..6].copy_from_slice(&p2.to_le_bytes());
    cmd[6..6 + data.len()].copy_from_slice(data);
    let crc = crc16_cryptoauth(&cmd[1..count-1]);
    cmd[count-1..count+1].copy_from_slice(&crc);

    i2c.write_async(ATECC_ADDR, cmd[..count+1].iter().copied()).await?;
    cmd.fill(0);
    Timer::after(exec).await;
    i2c.read_async(ATECC_ADDR, resp).await?;

    let n = resp[0] as usize;
    if n < 4 || n > resp.len() {
        return Err(CryptoStartupError::IncorrectRespLen(resp[0]));
    }
    if resp[n - 2..n] != crc16_cryptoauth(&resp[..n - 2]) {
        return Err(CryptoStartupError::BadCrc(resp[0]));
    }
    if n == 4 && resp[1] != 0x00 {
        return Err(CryptoStartupError::CommandFailed(resp[1]));
    }

    Ok(n)
}

async fn execute(
    i2c: &mut I2cAsync<'_>,
    opcode: u8,
    p1: u8,
    p2: u16,
    exec: Duration,
    resp: &mut [u8],
) -> CryptoStartupResult<usize> {
    execute_with_data(i2c, opcode, p1, p2, &[], exec, resp).await
}


async fn read_config_word(
    i2c: &mut I2cAsync<'_>,
    word_addr: u16
) -> CryptoStartupResult<[u8; 4]> {
    let mut resp = [0u8; 7]; // count + 4 data + 2 CRC
    let n = execute(
        i2c,
        OP_READ,
        READ_P1_CFG_WORD,
        word_addr,
        READ_EXEC,
        &mut resp
    ).await?;

    if n != 7 {
        return Err(CryptoStartupError::IncorrectRespLen(resp[0]));
    }

    Ok([resp[1], resp[2], resp[3], resp[4]])
}

async fn read_config_zone(i2c: &mut I2cAsync<'_>) -> CryptoStartupResult<[u8; 128]> {
    let mut config = [0u8; 128];
    for block in 0..4u16 {
        let mut resp = [0u8; 35]; // count + 32 data + 2 CRC
        let n = execute(
            i2c,
            OP_READ,
            READ_P1_CFG_BLOCK,
            block << 3,
            READ_EXEC,
            &mut resp
        ).await?;

        if n != 35 {
            return Err(CryptoStartupError::IncorrectRespLen(resp[0]));
        }
        let start = block as usize * 32;
        config[start..start + 32].copy_from_slice(&resp[1..33]);
    }
    Ok(config)
}


async fn modify_config_u16(
    i2c: &mut I2cAsync<'_>,
    byte_addr: u16,
    clear_mask: u16,
    set_bits: u16,
) -> CryptoStartupResult<()> {
    let word_addr = byte_addr / 4;
    let idx = (byte_addr % 4) as usize;

    let mut word = read_config_word(i2c, word_addr).await?;
    let mut field = u16::from_le_bytes([word[idx], word[idx + 1]]);
    field = (field & !clear_mask) | set_bits;
    word[idx..idx + 2].copy_from_slice(&field.to_le_bytes());

    let mut resp = [0u8; 4];
    execute_with_data(
        i2c,
        OP_WRITE,
        WRITE_P1_CFG_WORD,
        word_addr,
        &word,
        WRITE_EXEC,
        &mut resp
    )
        .await?;

    if read_config_word(i2c, word_addr).await? != word {
        return Err(CryptoStartupError::ConfigVerifyFailed);
    }
    Ok(())
}


// Set the KeyConfig of the selected slot to KEYTYPE_AES
// Set the SlotConfig of the selected (SC_IS_SECRET, WRITE_CONFIG = Never)
async fn configure_aes_slot(i2c: &mut I2cAsync<'_>, slot: Slot) -> CryptoStartupResult<()> {
    let s = slot as u16;
    modify_config_u16(
        i2c,
        KEY_CONFIG_BASE + 2 * s,
        KC_LOW_FIELDS_MASK,
        KC_KEYTYPE_AES
    ).await?;

    modify_config_u16(
        i2c,
        SLOT_CONFIG_BASE + 2 * s,
        0xFFFF,
        SC_AES_SLOT
    ).await?;
    Ok(())
}

// Check if AES is enabled on given slot on a given config
fn check_aes_config(config: &[u8; 128], slot: Slot) -> CryptoStartupResult<()> {
    if config[AES_ENABLE_BYTE] & 0x01 == 0 {
        return Err(CryptoStartupError::AesDisabledOnChip); // fixed at manufacture
    }
    let kc_byte = KEY_CONFIG_BASE as usize + 2 * slot as usize;
    let key_type = (config[kc_byte] >> 2) & 0x07;
    if key_type != 6 {
        return Err(CryptoStartupError::SlotNotAes);
    }
    Ok(())
}

// Locks the config zone
// Chip verifies that the CRC-16 of the config zone it contains matches p2 of cmd
async fn lock_config_zone(i2c: &mut I2cAsync<'_>, slot: Slot) -> CryptoStartupResult<()> {
    let config = read_config_zone(i2c).await?;
    check_aes_config(&config, slot)?;

    let crc = u16::from_le_bytes(crc16_cryptoauth(&config));
    let mut resp = [0u8; 4];
    execute(
        i2c,
        OP_LOCK,
        LOCK_P1_CONFIG,
        crc,
        LOCK_EXEC,
        &mut resp
    ).await?;
    Ok(())
}

// NOTE Only truly random if cfg zone is locked
async fn random32(i2c: &mut I2cAsync<'_>) -> CryptoStartupResult<[u8; 32]> {
    let mut resp = [0u8; 35];
    let n = execute(
        i2c,
        OP_RANDOM,
        RANDOM_P1_DEFAULT,
        0,
        RANDOM_EXEC,
        &mut resp
    ).await?;

    if n != 35 {
        return Err(CryptoStartupError::IncorrectRespLen(resp[0]));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&resp[1..33]);
    resp.fill(0);
    Ok(out)
}


// Writes aes key to given slot
async fn write_aes_key_plain(
    i2c: &mut I2cAsync<'_>,
    slot: Slot,
    key: &[u8; 16],
) -> CryptoStartupResult<()> {
    // Slot contains 2 keys so we pad out the rest with 0's
    let mut block = [0u8; 32];
    block[..16].copy_from_slice(key);

    let addr = (slot as u16) << 3; // data zone: slot in bits 3..6, block 0, offset 0
    let mut resp = [0u8; 4];
    let res = execute_with_data(
        i2c,
        OP_WRITE,
        WRITE_P1_DATA_BLOCK,
        addr,
        &block,
        WRITE_EXEC,
        &mut resp
    )
        .await;

    block.fill(0);
    res.map(|_| ())
}


// Retrieve a random AES key
async fn provision_aes_key(i2c: &mut I2cAsync<'_>, slot: Slot) -> CryptoStartupResult<()> {
    // Key length is 16 but random numbers are generated in chunks of 32
    let mut rnd = random32(i2c).await?;
    let mut key = [0u8; 16];
    key.copy_from_slice(&rnd[..16]);
    rnd.fill(0);

    let res = write_aes_key_plain(i2c, slot, &key).await;
    key.fill(0);
    res
}


// Lock data + OTP zones. After this, IsSecret and WriteConfig are enforced.
async fn lock_data_zone(i2c: &mut I2cAsync<'_>) -> CryptoStartupResult<()> {
    let mut resp = [0u8; 4];
    execute(
        i2c,
        OP_LOCK,
        LOCK_P1_DATA_NO_CRC,
        0,
        LOCK_EXEC,
        &mut resp
    ).await?;
    Ok(())
}



async fn prepare_chip(i2c: &mut I2cAsync<'_>) -> CryptoStartupResult<()> {
    let lock_word = read_config_word(i2c, LOCK_CFG_WORD).await?;
    let data_locked = lock_word[2] != UNLOCKED; // LockValue, byte 86
    let config_locked = lock_word[3] != UNLOCKED; // LockConfig, byte 87

    // A: config zone
    if config_locked {
        let config = read_config_zone(i2c).await?;
        check_aes_config(&config, AES_KEY_SLOT)?;
    } else {
        if !ALLOW_PERMANENT_LOCKS {
            return Err(CryptoStartupError::ConfigZoneUnlocked);
        }
        configure_aes_slot(i2c, AES_KEY_SLOT).await?;
        lock_config_zone(i2c, AES_KEY_SLOT).await?;
    }

    // B: data zone (key provisioning)
    if !data_locked {
        if !ALLOW_PERMANENT_LOCKS {
            return Err(CryptoStartupError::DataZoneUnlocked);
        }
        provision_aes_key(i2c, AES_KEY_SLOT).await?;
        lock_data_zone(i2c).await?;
    }

    Ok(())
}


// Configure i2c
// Wake and initialize chip
// Ensure chip is configured correctly
// Instantiate new AtCaClient
pub async fn startup_crypto<'d>(
    i2c1: Peri<'d, I2C1>,
    mut sda_pin: Peri<'d, PIN_2>,
    scl_pin: Peri<'d, PIN_3>,
    irqs: impl embassy_rp::interrupt::typelevel::Binding<
        embassy_rp::interrupt::typelevel::I2C1_IRQ,
        embassy_rp::i2c::InterruptHandler<I2C1>,
    >,
) -> CryptoStartupResult<AtCaClientAsync<'d>> {
    let mut rp_cfg = embassy_rp::i2c::Config::default();
    rp_cfg.frequency = 100_000;
    rp_cfg.sda_pullup = false;
    rp_cfg.scl_pullup = false;

    wake_and_initialize(sda_pin.reborrow()).await;
    let mut i2c = I2c::new_async(i2c1, scl_pin, sda_pin, irqs, rp_cfg);

    // Run the raw steps, then always try to put the chip back to sleep,
    // so AtCaClient's wake sees the 0x11 wake response.
    let res = prepare_chip(&mut i2c).await;
    let sleep_res = sleep_chip(&mut i2c).await;
    res?;
    sleep_res?;

    let atca = AtCaClientAsync::new(i2c);
    Ok(atca)
}
