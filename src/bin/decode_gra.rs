/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use std::env;
use qfg5reenigne::qfg5resource::qfg5gra;
use bmp::{Image, Pixel, px};

fn main() -> Result<()> {
    let args: Vec<_> = env::args().into_iter().collect();
    if args.len() != 2 {
        println!("usage: {} file.gra", args[0]);
        return Ok(())
    }

    let gra_data = std::fs::read(&args[1])?;
    let gra = qfg5gra::GraDecoder::new(&gra_data)?;
    for sprite_collection in &gra.sprite_collections {
        let mut bmp = Image::new(sprite_collection.width, sprite_collection.height);
        for sprite in &sprite_collection.sprites {
            for (x, y) in bmp.coordinates() {
                let value = sprite.pixels[((y * sprite_collection.width) as u32 + x) as usize];
                let p = gra.palette[value as usize];
                let p = px!(p.0, p.1, p.2);
                bmp.set_pixel(x, y, p);
            }

        }
        bmp.save("/tmp/g.bmp")?;
    }
    Ok(())
}
