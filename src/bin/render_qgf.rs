/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use bmp::{Image, Pixel, px};
use std::env;
use qfg5reenigne::qfg5resource::qfg5qgf;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().into_iter().collect();
    if args.len() != 2 {
        println!("usage: {} file.qgf", args[0]);
        return Ok(())
    }

    let qgf = std::fs::read(&args[1])?;
    let qgf = qfg5qgf::QgfDecoder::new(&qgf)?;

    let bitmap_width = 640;
    let chars_per_line = (bitmap_width / (qgf.max_char_width + 1)) as u32;
    let bitmap_height = ((qgf.chars.len() as u32 + chars_per_line - 1) / chars_per_line) * (qgf.char_height + 1);
    let mut bmp = Image::new(bitmap_width, bitmap_height);
    for (x, y) in bmp.coordinates(){
        bmp.set_pixel(x, y, px!(255, 0, 255));
    }

    for (n, ch) in qgf.chars.iter().enumerate() {
        let base_x = (n as u32 % chars_per_line) * (qgf.max_char_width + 1);
        let base_y = (n as u32 / chars_per_line) * (qgf.char_height + 1);
        for y in 0..qgf.char_height {
            for x in 0..ch.width {
                let v = ch.data[((ch.width * y) + x) as usize];
                let pixel = if v != 0 {
                    if qgf.is_3d {
                        let a = 255 - (v * 8);
                        px!(a, a, a)
                    } else {
                        px!(v, v, v)
                    }
                } else {
                    px!(255, 255, 255)
                };
                bmp.set_pixel(base_x + x, base_y + y, pixel);
            }
        }
    }

    bmp.save("/tmp/f.bmp")?;
    Ok(())
}
