/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::io::{Cursor, Seek, SeekFrom};

const QGF_NUM_CHARS: usize = 512;

pub struct QgfChar {
    pub width: u32,
    pub data: Vec<u8>,
}

pub struct QgfDecoder {
    pub max_char_width: u32,
    pub char_height: u32,
    pub is_3d: bool,
    pub chars: Vec<QgfChar>,
}

impl QgfDecoder {
    pub fn new(anm_data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&anm_data);
        let max_char_width = cursor.read_u32::<LittleEndian>()?;
        let char_height = cursor.read_u32::<LittleEndian>()?;
        let _char_space = cursor.read_u32::<LittleEndian>()?;
        let _unk1 = cursor.read_u32::<LittleEndian>()?;
        let flag_3d = cursor.read_u32::<LittleEndian>()?;
        let _unk2 = cursor.read_u32::<LittleEndian>()?;

        let mut char_widths = vec![ 0u8; QGF_NUM_CHARS ];
        for n in 0..char_widths.len() {
            char_widths[n] = cursor.read_u8()?;
        }
        let mut char_offsets = vec![ 0u32; QGF_NUM_CHARS ];
        for n in 0..char_offsets.len() {
            char_offsets[n] = cursor.read_u32::<LittleEndian>()?;
        }

        let mut chars = Vec::new();
        for n in 0..QGF_NUM_CHARS {
            cursor.seek(SeekFrom::Start(char_offsets[n] as u64))?;
            let width = char_widths[n] as u32;

            let mut data = vec![ 0u8; (width * char_height) as usize ];
            let mut offset: usize = 0;
            while offset < data.len() {
                let a = cursor.read_u8()?;
                let _b = cursor.read_u8()?;
                if (a & 0x80) == 0 {
                    data[offset] = a;
                    offset += 1;
                } else {
                    offset += 128 - (a & 0x7f) as usize;
                }
            }
            chars.push(QgfChar{ width, data });
        }
        Ok(QgfDecoder{ max_char_width, char_height, chars, is_3d: flag_3d != 0 })
    }
}
