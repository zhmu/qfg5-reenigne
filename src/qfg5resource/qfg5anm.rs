/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::{anyhow, Result};
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::io::{Cursor, Read, Seek};

pub struct AnmBlock {
    pub translation: [ f32; 3 ],
    pub rotation: [ f32; 9 ],
}

pub struct AnmAnim {
    pub blocks: Vec<AnmBlock>,
}

pub struct AnmDecoder {
    pub name: String,
    pub delay: u32,
    pub anims: Vec<AnmAnim>,
}

impl AnmDecoder {
    pub fn new(anm_data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(&anm_data);
        let magic = cursor.read_u32::<LittleEndian>()?;
        if magic != 0x564f5838 && magic != 0x5452494d { return Err(anyhow!("invalid anm magic")); }
        let header_size = cursor.read_u32::<LittleEndian>()?;
        if header_size != 36 { return Err(anyhow!("invalid header size")); }
        let mut name = vec![ 0u8; 16 ];
        cursor.read_exact(&mut name)?;
        let name = String::from_utf8(name)?;

        let num_anims = cursor.read_u32::<LittleEndian>()? as usize;
        let num_anim_blocks = cursor.read_u32::<LittleEndian>()? as usize;
        let delay = cursor.read_u32::<LittleEndian>()?;

        let mut anims = Vec::with_capacity(num_anims);
        for _ in 0..num_anims {
            let mut blocks = Vec::with_capacity(num_anim_blocks);
            for _ in 0..num_anim_blocks {
                let a = cursor.read_u32::<LittleEndian>()?;
                let b = cursor.read_u32::<LittleEndian>()?;
                if a != 1 || b != 0 { return Err(anyhow!("unexpected a/b values {}/{}", a, b)); }
                let mut translation = [ 0f32; 3 ];
                for n in 0..3 {
                    translation[n] = cursor.read_f32::<LittleEndian>()?;
                }
                let mut rotation = [ 0f32; 9 ];
                for n in 0..9 {
                    rotation[n] = cursor.read_f32::<LittleEndian>()?;
                }
                blocks.push(AnmBlock{ translation, rotation });
            }
            anims.push(AnmAnim{ blocks });
        }
        if cursor.stream_position()? != anm_data.len() as u64 {
            return Err(anyhow!("got extra data after decoding"));
        }
        Ok(AnmDecoder{
            name,
            delay,
            anims,
        })
    }
}

