
use embassy_rp::{flash::{self, Flash}, peripherals::FLASH};

use crate::storage::{constants::{ERASE_PAGE_SIZE, TOTAL_FLASH_SIZE, WRITE_PAGE_SIZE}, errors::{WritePageError, WritePageResult}};


pub type AsyncFlash<'d> = Flash<'d, FLASH, flash::Async, TOTAL_FLASH_SIZE>;

/*
 * Assign version num by setting X of the final bits to 0
 * Flash memory can only set from 1 -> 0 without overhead
 * Assigning version num by removing a bit may save an erase page reset
 */
const fn vault_version_repr(version: u8) -> u16 {
    assert!(version < 16, "vault_version_num above 16 is not supported");
    (0xFFFF >> version) << version
}

#[repr(u16)]
#[derive(Default, Clone, Copy)]
pub enum VaultVersion {
    #[default]
    Unspecified = vault_version_repr(0),
    V1          = vault_version_repr(1)
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum VaultStatus {
    Unused = 0xFF,
    Valid  = 0xA5,
    Stale  = 0x00,
}


/* SAFETY:
 * - size_of::<T>()  MUST BE == PAGE_SIZE
 * - align_of::<T>() MUST BE == PAGE_SIZE
 */
pub unsafe trait FlashPage<const PAGE_SIZE: usize>: Sized {
    const EMPTY: Self;

    fn as_slice(&self) -> &[u8; PAGE_SIZE] {
        // SAFETY: size and alignment is checked when safely implementing trait
        unsafe { &*(self as *const Self as *const [u8; PAGE_SIZE]) }
    }

    fn as_slice_mut(&mut self) -> &mut [u8; PAGE_SIZE] {
        unsafe { &mut *(self as *mut Self as *mut [u8; PAGE_SIZE]) }
    }
}

#[repr(align(256))]
pub struct WritePage([u8; WRITE_PAGE_SIZE]);

impl WritePage {
    pub fn iter(&self) -> core::slice::Iter<'_, u8> {
        self.0.iter()
    }

    // Write pages always start with a VaultStatus
    pub fn is_stale(&self) -> bool {
        self.0[0] == VaultStatus::Stale as u8
    }
}

impl<'a> TryFrom<&'a mut [u8]> for &'a mut WritePage {
    type Error = WritePageError;

    fn try_from(value: &'a mut [u8]) -> WritePageResult<Self> {
        if value.len() != WRITE_PAGE_SIZE {
            return Err(WritePageError::InvalidLength);
        }

        if value.as_ptr() as usize % WRITE_PAGE_SIZE != 0 {
            return Err(WritePageError::InvalidAlignment);
        }

        Ok(unsafe { &mut *(value.as_mut_ptr() as *mut WritePage) })
    }
}

const _: () = assert!(size_of::<WritePage>()  == WRITE_PAGE_SIZE);
const _: () = assert!(align_of::<WritePage>() == WRITE_PAGE_SIZE);
// SAFETY: size and alignment checked above
unsafe impl FlashPage<WRITE_PAGE_SIZE> for WritePage {
    const EMPTY: Self = Self([0xFF; WRITE_PAGE_SIZE]);
}

/* SAFETY:
 * - size_of::<T>()  MUST BE == PAGE_SIZE
 * - align_of::<T>() MUST BE == PAGE_SIZE
 * - The beginning byte of the struct MUST be a VaultStatus field
 */
pub unsafe trait AsWritePage {
    fn as_page(&self) -> &WritePage {
        unsafe { &*(self as *const Self as *const WritePage) }
    }
}

/* SAFETY:
 * - size_of::<T>()  must be == WRITE_PAGE_SIZE
 * - align_of::<T>() must be == WRITE_PAGE_SIZE
 */
pub unsafe trait FromWritePage: Sized {
    // SAFETY: ensure that WritePage that is used contains a valid Self
    unsafe fn from_page(page: &WritePage) -> &Self {
        unsafe { &*(page as *const WritePage as *const Self) }
    }
}


#[repr(align(4096))]
pub struct ErasePage([u8; ERASE_PAGE_SIZE]);

const _: () = assert!(size_of::<ErasePage>()  == ERASE_PAGE_SIZE);
const _: () = assert!(align_of::<ErasePage>() == ERASE_PAGE_SIZE);
// SAFETY: size and alignment checked above
unsafe impl FlashPage<ERASE_PAGE_SIZE> for ErasePage {
    const EMPTY: Self = Self([0xFF; ERASE_PAGE_SIZE]);
}
