/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;

pub type PaletteEntry = (u8, u8, u8);

pub struct NodDecoder {
    version: u8,
    palette: [ PaletteEntry; 256 ],
}

const NOD_PALETTE_OFFSET: usize = 168;

impl NodDecoder {
    pub fn new(nod_data: &[u8]) -> Result<Self> {
        let version = nod_data[6]; // 0 = demo, 4 = retail
        let mut palette = [ PaletteEntry::default(); 256 ];
        for n in 0..256_usize {
            let offset = NOD_PALETTE_OFFSET + n * 4;
            let r = nod_data[offset+0];
            let g = nod_data[offset+1];
            let b = nod_data[offset+2];
            palette[n] = (r, g, b);
        };
        Ok(Self{ version, palette })
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_palette(&self) -> &[ PaletteEntry; 256 ] { &self.palette }
}
