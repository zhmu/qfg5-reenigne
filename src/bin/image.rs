/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use bmp::{Image, Pixel, px};
use qfg5reenigne::qfg5resource::{qfg5nod, qfg5img, qfg5zzz};

fn main() -> Result<()> {
    // let id = 5700; // hades
    // let id = 4150; // entrace to erasmus
    // let id = 2900; // inn
    // let id = 7300; // dragon pool
    let id = 2000;
    let img_data = std::fs::read(format!("../data/img/{}.img", id))?;
    let nod_data = std::fs::read(format!("../data/nod/{}.nod", id))?;
    let zzz_data = std::fs::read(format!("../data/zzz/{}.zzz", id))?;

    let nod = qfg5nod::NodDecoder::new(&nod_data)?;
    let img = qfg5img::ImageDecoder::new(&img_data)?;
    let zzz = qfg5zzz::ZzzDecoder::new(&zzz_data, &img)?;
    let mut bmp = Image::new(img.get_height() as u32, img.get_width() as u32);
    for (x, y) in bmp.coordinates() {
        let value = img.get_pixels()[(x * img.get_width() as u32 + y) as usize];
        let pal = nod.get_palette()[value as usize];
        let p = px!(pal.0, pal.1, pal.2);
        bmp.set_pixel(x, y, p);
    }
    bmp.save("/tmp/i.bmp")?;

    let mut zzz_img = Image::new(zzz.get_height() as u32, zzz.get_width() as u32);
    for (x, y) in zzz_img.coordinates() {
        let value = zzz.get_pixels()[(y * zzz_img.get_width() as u32 + x) as usize];
        let p = px!(value, value, value);
        zzz_img.set_pixel(x, y, p);
    }
    zzz_img.save("/tmp/z.bmp")?;
    Ok(())
}
