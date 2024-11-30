/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::{anyhow, Result};
use std::io::{Read, Seek, SeekFrom};
use byteorder::{ReadBytesExt, LittleEndian};
use std::fs::File;
use std::os::unix::fs::FileExt;

pub struct SpkItem {
    pub filename: String,
    pub offset: u64,
    pub length: usize,
}

pub struct SpkArchive {
    f: File,
    items: Vec<SpkItem>
}

impl SpkArchive {
    pub fn new(mut f: File) -> Result<SpkArchive> {
        let file_len = f.seek(SeekFrom::End(0))? as u32;

        // Last 22 bytes of the SPK archive contain a end-of-directory structure.
        f.seek(SeekFrom::End(-22))?;
        let pk = f.read_u16::<LittleEndian>()?;
        if pk != 0x4b50 { return Err(anyhow!("invalid PK magic in end-of-directory record")); }
        let id = f.read_u16::<LittleEndian>()?;
        if id != 0x0705 { return Err(anyhow!("invalid PK id in end-of-directory record")); }
        f.seek(SeekFrom::Current(4))?; // 0, unknown purpose
        let num_files = f.read_u16::<LittleEndian>()?;
        let num_files_dup = f.read_u16::<LittleEndian>()?;
        if num_files != num_files_dup { return Err(anyhow!("file counts do not match ({} vs {})", num_files, num_files_dup)); }
        // 
        let a = f.read_u32::<LittleEndian>()?;
        let b = f.read_u32::<LittleEndian>()?;

        let local_file_start = (file_len - a) - b - 0x16;

        let central_directory_offset = (file_len - a) - 0x16;
        f.seek(SeekFrom::Start(central_directory_offset as u64))?;

        let mut items = Vec::<SpkItem>::with_capacity(num_files as usize);
        for n in 0..num_files {
            f.seek(SeekFrom::Current(20))?;
            let compr_size = f.read_u32::<LittleEndian>()?;
            let decompr_size = f.read_u32::<LittleEndian>()?;
            if compr_size != decompr_size { return Err(anyhow!("compressed entries are not supported")); }
            let fname_len = f.read_u32::<LittleEndian>()?;
            f.seek(SeekFrom::Current(10))?;
            let item_location = f.read_u32::<LittleEndian>()?;
            // All entries are prefixed by a "local file header", which can be skipped
            let offset = local_file_start + item_location + 0x42 + fname_len;
            let mut fname = vec![ 0u8; fname_len as usize ];
            f.read_exact(&mut fname)?;

            let filename = String::from_utf8(fname).unwrap_or_else(|_| format!("<corrupt-{}>", n));
            items.push(SpkItem{ filename, length: decompr_size as usize, offset: offset as u64 });
        }
        Ok(Self{ f, items })
    }

    pub fn get_items(&self) -> &Vec<SpkItem> {
        &self.items
    }

    pub fn read_item(&self, item: &SpkItem) -> Result<Vec<u8>> {
        let mut buf = vec![ 0u8; item.length ];
        self.f.read_exact_at(&mut buf, item.offset)?;
        Ok(buf)
    }
}

