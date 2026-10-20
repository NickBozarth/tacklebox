use embassy_rp::flash::WRITE_SIZE;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_storage_async::nor_flash::NorFlash;

use crate::storage::{constants::{ENTRIES_PER_VAULT, ERASE_PAGE_SIZE, FIRST_VAULT_ADDR, MAX_ENTRY_TEXT_LEN, VAULT_COUNT, WRITE_PAGE_SIZE, WRITE_PAGES_PER_VAULT, XIP_BASE}, crypto::{OwnedPlaintext, Passphrase}, errors::{VaultError, VaultResult}, layout::{Vault, VaultEntry, VaultHeader}, types::{AsWritePage, AsyncFlash, ErasePage, FlashPage, FromWritePage, VaultStatus, WritePage}};


/* This exists as a way to access and write data from within a vault in flash
 * Vaults should not be loaded into RAM because they are too large so we index specific sections
 *  that we want to read from at any given time
 */
pub struct VaultManager {
    flash_addr: *const u8,
    vault_index: usize
}

impl VaultManager {
    // Offset from the flash peripheral base address
    fn flash_offset(&self) -> u32 {
        (self.flash_addr as u32) - XIP_BASE
    }

    // Validates if a WritePage can be written to destination without erasing
    fn can_write(src_page: &WritePage, flash_page: &WritePage) -> bool {
        src_page.iter()
            .zip(flash_page.iter())
            .all(|(src, dst)| *src & *dst == *src)
    }

    /* Vaults are the size and alignment of ERASE_PAGE_SIZE
     * Indexing from FIRST_VAULT_ADDR and offseting (N * ERASE_PAGE_SIZE bytes) gives you the vault
     *  at index N
     */
    pub fn vault_at_index(vault_index: usize) -> VaultResult<Self> {
        if vault_index > VAULT_COUNT {
            return Err(VaultError::InvalidVaultIndex);
        }

        // FIRST_VAULT_ADDR is aligned to ERASE_PAGE_SIZE
        // size_of::<Vault>() is ERASE_PAGE_SIZE
        let flash_addr = FIRST_VAULT_ADDR + (vault_index * size_of::<Vault>());

        Ok(Self {
            flash_addr: flash_addr as *const u8,
            vault_index
        })
    }



    fn retrieve_page(&self, page_index: usize) -> VaultResult<&WritePage> {
        if page_index >= WRITE_PAGES_PER_VAULT {
            return Err(VaultError::InvalidPageIndex);
        }

        let page_offset = page_index * WRITE_SIZE;

        // SAFETY:
        // - self.flash_addr will always be a valid address of an erase page that contains a vault
        // - index is bounds checked agains WRITE_PAGES_PER_VAULT
        unsafe { Ok(&*(self.flash_addr.add(page_offset).cast())) }
    }

    pub fn retrieve_header(&self) -> &VaultHeader {
        match self.retrieve_page(0) {
            // SAFETY: Page 0 of a vault will always contain a VaultHeader
            Ok(page) => unsafe { VaultHeader::from_page(page) },
            Err(_) => unreachable!("VaultHeader will be at page 0 and never OOB")
        }
    }

    pub fn retrieve_entry(&self, entry_index: usize) -> VaultResult<&VaultEntry> {
        let page_index = entry_index.checked_add(1)
            .ok_or(VaultError::InvalidEntry)?;

        self.retrieve_page(page_index)
            // SAFETY:
            // - Page 0 is unreachable
            // - Pages 1-15 of a vault will always contain a VaultEntry
            // - Pages 16+ return an Err
            .map(|page| unsafe { VaultEntry::from_page(page) })
    }



    async fn write_page(
        &self,
        flash: &mut AsyncFlash<'_>,
        page: &impl AsWritePage,
        page_index: usize
    ) -> VaultResult<()> {
        let write_page = page.as_page();
        let flash_page = self.retrieve_page(page_index)?;
        let page_offset = page_index * size_of::<VaultEntry>() + size_of::<VaultHeader>();
        let flash_offset = self.flash_offset() + page_offset as u32;

        if !Self::can_write(write_page, flash_page) {
            return Err(VaultError::EraseNecessary);
        }

        flash
            .write(flash_offset, write_page.as_slice())
            .await
            .map_err(|_| VaultError::FlashWriteError)
    }

    pub async fn write_header(
        &self,
        flash: &mut AsyncFlash<'_>,
        header: &VaultHeader
    ) -> VaultResult<()> {
        self.write_page(flash, header, 0).await
    }

    pub async fn write_entry(
        &self,
        flash: &mut AsyncFlash<'_>,
        entry: &VaultEntry,
        entry_index: usize
    ) -> VaultResult<()> {
        let page_index = entry_index.checked_add(1)
            .ok_or(VaultError::InvalidEntry)?;

        self.write_page(flash, entry, page_index).await
    }



    async fn set_page_stale(
        &self,
        flash: &mut AsyncFlash<'_>,
        page_index: usize
    ) -> VaultResult<()> {
        if page_index >= WRITE_PAGES_PER_VAULT {
            return Err(VaultError::InvalidPageIndex);
        }

        let page_offset = page_index * WRITE_PAGE_SIZE;
        let write_offset = self.flash_offset() + page_offset as u32;
        let write_bytes = [VaultStatus::Stale as u8];

        // SAFETY:
        // - self.flash_addr will always be a valid address of an erase page that contains a vault
        // - The begining of a page will always contain a VaultStatus field. This comes with
        //   implementing the AsWritePage trait that is needed to write to flash
        flash
            .write(write_offset, &write_bytes)
            .await
            .map_err(|_| VaultError::FlashWriteError)
    }

    pub async fn set_vault_stale(&self, flash: &mut AsyncFlash<'_>) -> VaultResult<()> {
        self.set_page_stale(flash, 0).await
    }

    pub async fn set_entry_stale(
        &self,
        flash: &mut AsyncFlash<'_>,
        entry_index: usize
    ) -> VaultResult<()> {
        let page_index = entry_index.checked_add(1)
            .ok_or(VaultError::InvalidPageIndex)?;

        self.set_page_stale(flash, page_index).await
    }



    pub async fn erase_stale_pages(&self, flash: &mut AsyncFlash<'_>) -> VaultResult<()> {
        static STATIC_VAULT_BYTES: Mutex<CriticalSectionRawMutex, ErasePage> = Mutex::new(ErasePage::EMPTY);

        let mut static_vault_bytes = STATIC_VAULT_BYTES.lock().await;

        // SAFETY: self.flash_addr will always contain a valid pointer to ERASE_PAGE_SIZE readable
        //         bytes of memory
        let flash_slice = unsafe {
            core::slice::from_raw_parts(self.flash_addr, ERASE_PAGE_SIZE)
        };

        static_vault_bytes.as_slice_mut().copy_from_slice(flash_slice);

        static_vault_bytes.as_slice_mut()
            .chunks_exact_mut(WRITE_PAGE_SIZE)
            .filter_map(|chunk| chunk.try_into().ok())
            .filter(|page: &&mut WritePage| page.is_stale())
            .for_each(|page| *page = WritePage::EMPTY);

        flash
            .write(self.flash_offset(), static_vault_bytes.as_slice())
            .await
            .map_err(|_| VaultError::FlashWriteError)
    }



    async fn write_page_erasing(
        &self,
        flash: &mut AsyncFlash<'_>,
        page: &impl AsWritePage,
        page_index: usize
    ) -> VaultResult<()> {
        match self.write_page(flash, page, page_index).await {
            Err(VaultError::EraseNecessary) => {
                self.set_page_stale(flash, page_index).await?;
                self.erase_stale_pages(flash).await?;
                self.write_page(flash, page, page_index).await
            }
            res => res
        }
    }
}


// This is a struct made to circumvent the need for placing the entire unlocked vault into stack
// Instead, wee take the base address in flash memory and lazilly generate unlocked entries
pub struct UnlockedVaultIterator<'a> {
    vault_manager: &'a VaultManager,
    passphrase: Passphrase,
    entry_num: u8
}

impl<'a> Iterator for UnlockedVaultIterator<'a> {
    type Item = OwnedPlaintext<MAX_ENTRY_TEXT_LEN>;

    fn next(&mut self) -> Option<Self::Item> {
        while (self.entry_num as usize) < ENTRIES_PER_VAULT {
            let entry = self.vault_manager.retrieve_entry(self.entry_num as usize).ok()?;

            self.entry_num += 1;

            if let Ok(unlocked_entry) = entry.unlock(&self.passphrase) {
                return Some(unlocked_entry);
            }
        }

        None
    }
}
