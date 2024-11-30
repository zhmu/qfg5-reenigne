/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use std::path::{Path, PathBuf};
use std::io::Write;
use anyhow::Result;
use std::fs::File;
use clap::{Parser, Subcommand};

use qfg5reenigne::qfg5resource::qfg5spk;

#[derive(Subcommand)]
enum CliCommands {
    /// Extract resources
    Extract {
        /// Output directory
        out_dir: PathBuf,
    },
    /// Lists all resources
    List,
}

/// Extracts Quest for Glory 5 resources from *.SPK to individual files
#[derive(Parser)]
struct Cli {
    /// Input SPK file
    in_spk: PathBuf,
    #[command(subcommand)]
    command: Option<CliCommands>
}

fn list(archive: &qfg5spk::SpkArchive) -> Result<()> {
    for item in archive.get_items() {
        println!("  {:>20}, {:>8} bytes @ offset 0x{:x}", item.filename, item.length, item.offset);
    }
    println!("{} total", archive.get_items().len());
    Ok(())
}

fn extract(out_dir: &Path, archive: &qfg5spk::SpkArchive) -> Result<()> {
    for item in archive.get_items() {
        let data = archive.read_item(item)?;

        let lower_filename = item.filename.to_lowercase();
        let path = Path::new(&lower_filename);

        let dirname = out_dir.join(path.parent().unwrap());
        let filename = path.file_name().unwrap();
        std::fs::create_dir_all(&dirname)?;

        let out_path = dirname.join(filename);
        let mut ff = File::create(out_path)?;
        ff.write(&data)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let f = File::open(args.in_spk)?;
    let archive = qfg5spk::SpkArchive::new(f)?;

    match &args.command {
        Some(CliCommands::Extract { out_dir }) => {
            extract(&out_dir, &archive)?;
        },
        Some(CliCommands::List) => {
            list(&archive)?;
        },
        None => { }
    }
    Ok(())
}
