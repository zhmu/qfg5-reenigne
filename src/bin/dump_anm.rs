/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use std::env;
use qfg5reenigne::qfg5resource::qfg5anm;

fn fmt_f32(num: f32, width: usize) -> String {
    let mut s = format!("{}", num);
    while s.len() < width {
        s = " ".to_owned() + &s;
    }
    s
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().into_iter().collect();
    if args.len() != 2 {
        println!("usage: {} file.anm", args[0]);
        return Ok(())
    }

    let anm_data = std::fs::read(&args[1])?;
    let anm = qfg5anm::AnmDecoder::new(&anm_data)?;
    println!("animation '{}' delay {}", anm.name, anm.delay);
    println!("  {} animations, {} blocks each", anm.anims.len(), anm.anims.first().unwrap().blocks.len());
    for (n, anim) in anm.anims.iter().enumerate() {
        println!("  animation {}", n);
        for (n, block) in anim.blocks.iter().enumerate() {
            let width = 20;
            println!("    block {}: translation {}, {}, {}", n, block.translation[0], block.translation[1], block.translation[2]);
            println!("    rotation:");
            println!("      {}, {}, {}", fmt_f32(block.rotation[0], width), fmt_f32(block.rotation[1], width), fmt_f32(block.rotation[2], width));
            println!("      {}, {}, {}", fmt_f32(block.rotation[3], width), fmt_f32(block.rotation[4], width), fmt_f32(block.rotation[5], width));
            println!("      {}, {}, {}", fmt_f32(block.rotation[6], width), fmt_f32(block.rotation[7], width), fmt_f32(block.rotation[8], width));
        }
    }
    Ok(())
}
