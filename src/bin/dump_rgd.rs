/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use std::env;
use qfg5reenigne::qfg5resource::qfg5rgd;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().into_iter().collect();
    if args.len() != 2 {
        println!("usage: {} file.rgd", args[0]);
        return Ok(())
    }

    let rgd_data = std::fs::read(&args[1])?;
    let _rgd = qfg5rgd::RgdDecoder::new(&rgd_data)?;
    Ok(())
}
