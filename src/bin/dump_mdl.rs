/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use std::env;
use qfg5reenigne::qfg5resource::qfg5mdl;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().into_iter().collect();
    if args.len() != 2 {
        println!("usage: {} file.mdl", args[0]);
        return Ok(())
    }

    let mdl_data = std::fs::read(&args[1])?;
    let mdl = qfg5mdl::Qfg5Model::new(&mdl_data)?;
    println!("model '{}', {} submeshes", mdl.name, mdl.submeshes.len());
    for sm in &mdl.submeshes {
        println!("  submesh '{}', {} vertices, {} texcoords, {} faces, {} lighting faces",
            sm.name, sm.vertices.len(), sm.texcoords.len(), sm.faces.len(), sm.lighting_vertices.len());
    }
    Ok(())
}
