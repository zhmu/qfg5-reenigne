/*-
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * Copyright (c) 2024 Rink Springer <rink@rink.nu>
 * For conditions of distribution and use, see LICENSE file
 */
use anyhow::{anyhow, Result};
use std::io::Read;
use byteorder::{ByteOrder, ReadBytesExt, LittleEndian};
use std::io::Cursor;
use std::fmt;

const FLAG_TEXT_MANGLED: u16 = 4;

fn demangle_text(data: &[u8]) -> String {
    let mut output = String::new();
    let mut data_cursor = Cursor::new(&data);
    // Process 4 bytes at a time
    while let Ok(mut v) = data_cursor.read_u32::<LittleEndian>() {
        v = v ^ 0xf1acc1d;
        v= v.rotate_right(15);
        let mut chars = [ 0u8; 4 ];
        LittleEndian::write_u32(&mut chars, v);
        for ch in chars {
            output.push(ch as char);
        }
    }
    // Process remaining bytes
    while let Ok(mut v) = data_cursor.read_u8() {
        v = !v;
        output.push(v as char);
    }
    output
}

#[derive(Debug)]
pub struct QgmLabel {
    value: [ u8; 12 ],
}

fn encode_digit_base_36(v: u16) -> Option<char> {
    if v < 10 {
        char::from_u32(('0' as u16 + v) as u32)
    } else if v < 36 {
        char::from_u32(('A' as u16 + (v - 10)) as u32)
    } else {
        None
    }
}
fn encode_base_36(v: u16, num_digits: usize) -> Option<String> {
    if v >= 36_u16.pow(num_digits as u32) {
        return None; // can't fit in this amount of digits
    }
    let mut chars = vec![ 0u8; num_digits ];
    let mut v = v;
    for n in (0..num_digits).rev() {
        chars[n] = encode_digit_base_36(v % 36).unwrap() as u8;
        v = v / 36;
    }
    String::from_utf8(chars).ok()
}

impl QgmLabel {
    pub fn new(cursor: &mut Cursor<&[u8]>) -> Result<QgmLabel> {
        let mut message = [ 0u8; 13 ];
        cursor.read_exact(&mut message)?;
        if message[12] != 0 { return Err(anyhow!("label does not end in zero byte")); }

        let mut value = [ 0u8; 12 ];
        value.copy_from_slice(&message[0..12]);
        Ok(QgmLabel{ value })
    }

    pub fn encode(qgm: &QgmDecoder, m: &QgmMessage) -> String {
        format!("{}{}{}.{}{}",
            encode_base_36(qgm.file_id, 3).unwrap(),
            encode_base_36(m.id[0], 2).unwrap(),
            encode_base_36(m.id[1], 2).unwrap(),
            encode_base_36(m.id[2], 2).unwrap(),
            encode_base_36(m.id[3], 1).unwrap())
    }
}

impl fmt::Display for QgmLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}{}{}{}{}{}{}{}{}",
               self.value[0] as char, self.value[1] as char, self.value[2] as char,
               self.value[3] as char, self.value[4] as char, self.value[5] as char,
               self.value[6] as char, self.value[7] as char, self.value[8] as char,
               self.value[9] as char, self.value[10] as char, self.value[11] as char)
    }
}

pub struct QgmMessage {
    pub id: [ u16; 4 ],
    pub speaker_id: u16,
    pub msg_id: u16,
    pub message_label: Option<QgmLabel>,
    pub dialog_options: Vec<QgmLabel>,
    pub text: String,
}

pub struct QgmDecoder {
    pub file_id: u16,
    pub messages: Vec<QgmMessage>,
}

impl QgmDecoder {
    pub fn new(data: &[u8]) -> Result<Self> {
        // decode header (16 bytes)
        let mut cursor = Cursor::new(data);
        let magic = cursor.read_u32::<LittleEndian>()?;
        if magic != 0x51474d20 { return Err(anyhow!("invalid magic")); }
        let _version = cursor.read_u32::<LittleEndian>()?;
        // TODO verify version
        let num_messages = cursor.read_u32::<LittleEndian>()?;
        let _unk1 = cursor.read_u16::<LittleEndian>()?;
        let file_id = cursor.read_u16::<LittleEndian>()?;

        let mut messages = Vec::new();
        for _ in 0..num_messages {
            // message block header (32 bytes)
            let id1 = cursor.read_u16::<LittleEndian>()?;
            let id2 = cursor.read_u16::<LittleEndian>()?;
            let id3 = cursor.read_u16::<LittleEndian>()?;
            let id4 = cursor.read_u16::<LittleEndian>()?;
            let speaker_id = cursor.read_u16::<LittleEndian>()?; // maybe
            let _unk2 = cursor.read_u16::<LittleEndian>()?;
            let _unk3 = cursor.read_u16::<LittleEndian>()?;
            let _unk4 = cursor.read_u16::<LittleEndian>()?;
            let num_dialog_options = cursor.read_u16::<LittleEndian>()?;
            let flags = cursor.read_u16::<LittleEndian>()?;
            let _unk5 = cursor.read_u16::<LittleEndian>()?;
            let msg_id  = cursor.read_u16::<LittleEndian>()?;
            let msg_length = cursor.read_u16::<LittleEndian>()?;
            let _msg_flag = cursor.read_u16::<LittleEndian>()?;
            let msg_label_flag = cursor.read_u16::<LittleEndian>()?;
            let _unk6 = cursor.read_u16::<LittleEndian>()?;

            let message_label: Option<QgmLabel>;
            if msg_label_flag != 0 {
                let label = QgmLabel::new(&mut cursor)?;
                message_label = Some(label);
            } else {
                message_label = None;
            }

            let mut dialog_options = Vec::new();
            for _ in 0..num_dialog_options {
                let label = QgmLabel::new(&mut cursor)?;
                dialog_options.push(label);
            }

            let mut text_data = vec![ 0u8; msg_length as usize ];
            cursor.read_exact(&mut text_data)?;
            let _unk8 = cursor.read_u32::<LittleEndian>()?;

            let text = if (flags & FLAG_TEXT_MANGLED) != 0 {
                demangle_text(&text_data)
            } else {
                String::from_utf8(text_data)?
            };

            log::debug!("id {}/{}/{}/{} speaker_id {} unk2345 {} {} {} {} {} {}: {}",
                id1, id2, id3, id4,
                speaker_id,
                _unk2, _unk3, _unk4, _unk5, _unk6, _unk8, text);

            messages.push(QgmMessage{
                id: [ id1, id2, id3, id4 ],
                speaker_id, msg_id,
                message_label,
                dialog_options,
                text,
            });
        }
        Ok(QgmDecoder{ file_id, messages })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_digit_base_36() {
        assert_eq!(encode_digit_base_36(0), Some('0'));
        assert_eq!(encode_digit_base_36(9), Some('9'));
        assert_eq!(encode_digit_base_36(10), Some('A'));
        assert_eq!(encode_digit_base_36(35), Some('Z'));
        assert!(encode_digit_base_36(36).is_none())
    }

    #[test]
    fn test_encode_base_36() {
        assert_eq!(encode_base_36(0, 0), Some("".to_string()));
        assert_eq!(encode_base_36(0, 5), Some("00000".to_string()));
        assert_eq!(encode_base_36(36, 2), Some("10".to_string()));
        assert_eq!(encode_base_36(46655, 3), Some("ZZZ".to_string()));
        assert_eq!(encode_base_36(415, 3), Some("0BJ".to_string()));
        assert!(encode_base_36(36, 1).is_none());
    }
}
