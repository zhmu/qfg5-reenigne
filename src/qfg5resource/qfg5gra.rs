/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use std::io::{Cursor, Read, Seek, SeekFrom};
use byteorder::{ByteOrder, ReadBytesExt, LittleEndian};
use crate::qfg5resource::decode;

pub struct GraSprite {
    pub pixels: Vec<u8>,
}

pub struct GraSpriteCollection {
    pub x_position: u32,
    pub y_position: u32,
    pub width: u32,
    pub height: u32,
    pub frame_delay: u32,
    pub sprites: Vec<GraSprite>,
}

pub struct GraDecoder {
    pub palette: [ (u8, u8, u8); 256 ],
    pub sprite_collections: Vec<GraSpriteCollection>,
}

fn decode_rgb555_palette(rgb555: &[u8]) -> [ (u8, u8, u8); 256 ] {
    let mut result = [ (0u8, 0u8, 0u8); 256 ];
    for n in 0..256 {
        let v = LittleEndian::read_u16(&rgb555[n*2+0..n*2+2]);
        let r = (v >> 10) & 31;
        let g = (v >> 5) & 31;
        let b = (v >> 0) & 31;
        let r = ((255.0 / 31.0) * r as f32) as u8;
        let g = ((255.0 / 31.0) * g as f32) as u8;
        let b = ((255.0 / 31.0) * b as f32) as u8;
        result[n] = (r, g, b);
    }
    result
}

impl GraDecoder {
    pub fn new(gra_data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(gra_data);

        let colour_mode = cursor.read_u32::<LittleEndian>()?;
        let num_collections = cursor.read_u32::<LittleEndian>()? as usize;
        let mut rgb555 = [ 0u8; 512 ];
        cursor.read_exact(&mut rgb555)?;
        let palette = decode_rgb555_palette(&rgb555);

        let mut sprite_collection_offsets = vec! [ 0u32; num_collections ];
        for n in 0..num_collections {
            sprite_collection_offsets[n] = cursor.read_u32::<LittleEndian>()?;
        }
        println!("colour_mode {} num_collections {}", colour_mode, num_collections);

        let mut sprite_collections = Vec::new();
        for offset in &sprite_collection_offsets {
            cursor.seek(SeekFrom::Start(*offset as u64))?;

            let x_position = cursor.read_u32::<LittleEndian>()?;
            let y_position = cursor.read_u32::<LittleEndian>()?;
            let width = cursor.read_u32::<LittleEndian>()?;
            let height = cursor.read_u32::<LittleEndian>()?;
            let num_sprites = cursor.read_u32::<LittleEndian>()? as usize;
            let frame_delay = cursor.read_u32::<LittleEndian>()?;
            let _flags = cursor.read_u32::<LittleEndian>()?;

            let mut frame_offsets = vec![ 0u32; num_sprites ];
            for n in 0..num_sprites {
                frame_offsets[n] = cursor.read_u32::<LittleEndian>()?;
            }

            let mut sprites = Vec::new();
            for n in 0..num_sprites {
                cursor.seek(SeekFrom::Start((*offset + frame_offsets[n]) as u64))?;

                let data = &gra_data[cursor.stream_position()? as usize..];
                let mut pixels = vec![ 0u8; (width * height) as usize ];
                match colour_mode {
                    0 => {
                        pixels.copy_from_slice(&data[0..(height * width) as usize]);
                    },
                    2 => {
                        decode::decode_rle(&data, &mut pixels);
                    },
                    _ => { todo!("colour mode {}", colour_mode); }
                }

                sprites.push(GraSprite{ pixels });
            }

            sprite_collections.push(GraSpriteCollection{
                x_position, y_position,
                width, height,
                frame_delay,
                sprites
            });
        }
        Ok(GraDecoder{ palette, sprite_collections })
    }
}
