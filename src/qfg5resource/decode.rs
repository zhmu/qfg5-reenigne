/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
pub fn decode_rle(data: &[u8], output: &mut [u8]) {
    let mut output_index: usize = 0;
    let mut n: usize = 0;
    while n < data.len() {
        let count = data[n] as usize;
        if count == 0 {
            // todo!();
            n += 1;
        } else if count < 128 {
            let value = data[n + 1];
            for _ in 0..count {
                output[output_index] = value;
                output_index += 1;
                if output_index == output.len() { break; }
            }
            n += 2;
        } else {
            let count = 256 - count;
            for j in 0..count {
                output[output_index] = data[n + j + 1];
                output_index += 1;
                if output_index == output.len() { break; }
            }
            n += count + 1;
        }
    }
}
