/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use crate::qfg5resource::{decode, qfg5img};

pub struct ZzzDecoder {
    width: u16,
    height: u16,
    pixels: Vec<u8>,
}

impl ZzzDecoder {
    pub fn new(zzz_data: &[u8], img: &qfg5img::ImageDecoder) -> Result<Self> {
        let width = img.get_width();
        let height = img.get_height();
        let mut pixels = vec![ 0u8; width as usize * height as usize ];
        decode::decode_rle(zzz_data, &mut pixels);
        Ok(ZzzDecoder{ height, width, pixels })
    }

    pub fn get_height(&self) -> u16 { self.height }
    pub fn get_width(&self) -> u16 { self.width}
    pub fn get_pixels(&self) -> &[u8] { &self.pixels}
}
