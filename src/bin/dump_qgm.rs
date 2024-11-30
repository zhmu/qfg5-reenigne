/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::Result;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use qfg5reenigne::qfg5resource::qfg5qgm;

#[derive(Subcommand)]
enum CliCommands {
    /// Lists all resources
    List,
}

/// Extracts Quest for Glory 5 messages from *.QGM
#[derive(Parser)]
struct Cli {
    /// Input QGM file
    in_qgm: PathBuf,
    #[command(subcommand)]
    command: Option<CliCommands>
}

fn list(qgm: &qfg5qgm::QgmDecoder) -> Result<()> {
    println!("qgm file id: {}", qgm.file_id);
    for m in &qgm.messages {
        let message_id = qfg5qgm::QgmLabel::encode(&qgm, &m);
        println!("{} message {}: '{}'", message_id, m.msg_id, m.text);
        if let Some(ml) = &m.message_label {
            println!("  message label: {}", ml);
        }
        for dlo in &m.dialog_options {
            println!("  dialog option: {}", dlo);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();
    let data = std::fs::read(args.in_qgm)?;
    let qgm = qfg5qgm::QgmDecoder::new(&data)?;

    match &args.command {
        Some(CliCommands::List) => {
            list(&qgm)?;
        }
        None => { },
    }
    Ok(())
}
