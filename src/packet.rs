use std::{convert::TryInto, io, mem};

use bytes::{Buf, BufMut, BytesMut};

pub(crate) struct Packet {
    pub(crate) id: u32,
    pub(crate) is_response: bool,
    pub(crate) is_from_server: bool,
    pub(crate) words: Vec<String>,
}

impl Packet {
    const HEADER_SIZE: usize = 3 * mem::size_of::<u32>();
    const WORD_HEADER_FOOTER_SIZE: usize = mem::size_of::<u32>() + mem::size_of::<u8>();
    const MAX_SIZE: usize = 16384;

    pub(crate) fn new(
        id: u32,
        is_response: bool,
        is_from_server: bool,
        words: Vec<String>,
    ) -> Packet {
        Packet {
            id,
            is_response,
            is_from_server,
            words,
        }
    }

    pub(crate) fn read(buf: &mut BytesMut) -> io::Result<Option<Packet>> {
        if buf.len() < Packet::HEADER_SIZE {
            return Ok(None);
        }
        let mut size_slice = [0_u8; 4];
        size_slice.copy_from_slice(&buf[4..8]);
        let size: usize = u32::from_le_bytes(size_slice).try_into().unwrap();
        if buf.len() < size {
            return Ok(None);
        }
        let mut header_buf = buf.split_to(Packet::HEADER_SIZE);
        let sequence = header_buf.get_u32_le();
        header_buf.advance(4);
        let word_count: usize = header_buf.get_u32_le().try_into().unwrap();
        let mut body_buf = buf.split_to(size - Packet::HEADER_SIZE);
        let mut words = Vec::with_capacity(word_count);

        for _ in 0..word_count {
            if body_buf.len() < mem::size_of::<u32>() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "malformed packet",
                ));
            }
            let word_size: usize = body_buf.get_u32_le().try_into().unwrap();
            if body_buf.len() < word_size + mem::size_of::<u8>() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "malformed packet",
                ));
            }
            let word_buf = body_buf.split_to(word_size).freeze();
            words.push(
                String::from_utf8(word_buf.to_vec())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            );
            if body_buf.get_u8() != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "malformed packet",
                ));
            }
        }

        let id = sequence & 0x3FFF_FFFF;
        let is_response = sequence & 0x4000_0000 != 0;
        let is_from_server = sequence & 0x8000_0000 != 0;

        Ok(Some(Packet::new(id, is_response, is_from_server, words)))
    }

    pub(crate) fn write(&self, buf: &mut BytesMut) -> io::Result<()> {
        let size = Packet::HEADER_SIZE
            + self
                .words
                .iter()
                .map(|w| Packet::WORD_HEADER_FOOTER_SIZE + w.len())
                .sum::<usize>();
        if size > Packet::MAX_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "max packet size exceeded",
            ));
        }
        buf.reserve(size);
        let mut sequence = self.id & 0x3FFF_FFFF;
        if self.is_response {
            sequence |= 0x4000_0000;
        }
        if self.is_from_server {
            sequence |= 0x8000_0000;
        }
        buf.put_u32_le(sequence);
        buf.put_u32_le(size as u32);
        buf.put_u32_le(self.words.len() as u32);

        for w in &self.words {
            buf.put_u32_le(w.len() as u32);
            buf.put(w.as_bytes());
            buf.put_u8(0);
        }

        Ok(())
    }
}
