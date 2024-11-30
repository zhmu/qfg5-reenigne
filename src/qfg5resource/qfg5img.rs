/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use byteorder::{ByteOrder, LittleEndian};
use crate::qfg5resource::decode;

const IMG_DATA_OFFSET: usize = 64;

pub struct ImageDecoder {
    height: u16,
    width: u16,
    pixels: Vec<u8>,
}

impl ImageDecoder {
    pub fn new(img_data: &[u8]) -> Result<Self> {
        let width = LittleEndian::read_u16(&img_data[32..34]);
        let height = LittleEndian::read_u16(&img_data[36..38]);
        let mut pixels = vec![ 0u8; width as usize * height as usize ];

        decode::decode_rle(&img_data[IMG_DATA_OFFSET..], &mut pixels);
        Ok(ImageDecoder{ height, width, pixels })
    }

    pub fn get_height(&self) -> u16 { self.height }
    pub fn get_width(&self) -> u16 { self.width}
    pub fn get_pixels(&self) -> &[u8] { &self.pixels}
}
