//! Decompressor for packed Another World resources.
//!
//! Direct port of `releases/common_data/Unpacker.py`. The algorithm
//! consumes its input bit-stream **backwards** from the end of the
//! packed buffer and likewise writes its output bytes backwards from
//! `raw_data_size - 1` toward `0`. Both Python and Rust implementations
//! must read/write in the exact same order or the output diverges.
//!
//! The unpacker shares one buffer for input and output: the packed
//! bytes occupy positions `0..packed.len()`, and the unpacked bytes
//! end up occupying `0..raw_data_size`. Reads and writes are
//! interleaved; the algorithm is correct precisely because every
//! `copy_data` read at offset `output_index + N` happens after the
//! prior write that put a byte there.

/// Result of `unpack`: either the decompressed bytes, or `None` if the
/// CRC check at the end of the bit-stream fails (matching the Python
/// reference's behavior of returning `None` on CRC mismatch).
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UnpackResult {
    Ok(Vec<u8>),
    CrcFailure,
}

pub fn unpack(packed: &[u8]) -> UnpackResult {
    Unpacker::new(packed).run()
}

struct Unpacker<'a> {
    packed: &'a [u8],
    /// Combined work buffer: positions `[0, raw_data_size)` end up
    /// holding the unpacked output. The packed input lives in the
    /// same buffer at positions `[0, packed.len())` and is consumed
    /// (read backwards) as the output is written (also backwards).
    buf: Vec<u8>,
    /// Next byte index to read from for the bit-stream.
    /// Starts at `packed.len() - 4` and decrements by 4 on each
    /// `read_be_uint32` call. Eventually goes negative; we keep
    /// the type wide enough to make that representable.
    input_index: i64,
    /// Index where the next unpacked byte goes. Starts at
    /// `raw_data_size - 1` and decrements.
    output_index: i64,
    crc: u32,
    chk: u32,
}

impl<'a> Unpacker<'a> {
    fn new(packed: &'a [u8]) -> Self {
        Self {
            packed,
            buf: Vec::new(),
            input_index: 0,
            output_index: 0,
            crc: 0,
            chk: 0,
        }
    }

    fn run(mut self) -> UnpackResult {
        // Sanity guard: the four-uint32 prologue at the end of the
        // packed buffer requires at least 16 bytes. (Less than that is
        // not a valid packed payload.)
        if self.packed.len() < 16 {
            return UnpackResult::CrcFailure;
        }

        self.input_index = self.packed.len() as i64 - 4;

        let raw_data_size = self.read_be_uint32() as usize;
        // Sized to hold both the packed input (front) and the unpacked
        // output (which extends to `raw_data_size`).
        let buf_size = core::cmp::max(self.packed.len(), raw_data_size);
        self.buf = vec![0u8; buf_size];
        self.buf[..self.packed.len()].copy_from_slice(self.packed);

        self.output_index = raw_data_size as i64 - 1;

        self.crc = self.read_be_uint32();
        self.chk = self.read_be_uint32();
        self.crc ^= self.chk;

        loop {
            if self.next_bit() {
                let c = self.get_code(2);
                match c {
                    0 => {
                        let offset = self.get_code(9);
                        self.copy_data(3, offset);
                    }
                    1 => {
                        let offset = self.get_code(10);
                        self.copy_data(4, offset);
                    }
                    2 => {
                        let count = 1 + self.get_code(8) as usize;
                        let offset = self.get_code(12);
                        self.copy_data(count, offset);
                    }
                    3 => {
                        let count = 9 + self.get_code(8) as usize;
                        self.raw_bytes(count);
                    }
                    _ => unreachable!("get_code(2) returns 0..=3"),
                }
            } else if self.next_bit() {
                let offset = self.get_code(8);
                self.copy_data(2, offset);
            } else {
                let count = 1 + self.get_code(3) as usize;
                self.raw_bytes(count);
            }

            if self.output_index < 0 {
                if self.crc != 0 {
                    return UnpackResult::CrcFailure;
                }
                self.buf.truncate(raw_data_size);
                return UnpackResult::Ok(self.buf);
            }
        }
    }

    fn read_be_uint32(&mut self) -> u32 {
        let i = self.input_index as usize;
        self.input_index -= 4;
        ((self.buf_or_packed(i) as u32) << 24)
            | ((self.buf_or_packed(i + 1) as u32) << 16)
            | ((self.buf_or_packed(i + 2) as u32) << 8)
            | (self.buf_or_packed(i + 3) as u32)
    }

    /// Read a byte either from `self.buf` if it has been allocated, or
    /// directly from `self.packed` if not. The first call to
    /// `read_be_uint32` happens BEFORE `self.buf` is populated, so we
    /// transparently dispatch.
    fn buf_or_packed(&self, i: usize) -> u8 {
        if self.buf.is_empty() {
            self.packed[i]
        } else {
            self.buf[i]
        }
    }

    fn raw_bytes(&mut self, count: usize) {
        for _ in 0..count {
            let value = self.get_code(8) as u8;
            let oi = self.output_index as usize;
            self.buf[oi] = value;
            self.output_index -= 1;
        }
    }

    fn copy_data(&mut self, count: usize, offset: u32) {
        for _ in 0..count {
            let oi = self.output_index as usize;
            let value = self.buf[oi + offset as usize];
            self.buf[oi] = value;
            self.output_index -= 1;
        }
    }

    fn get_code(&mut self, num_bits: u32) -> u32 {
        let mut c = 0u32;
        for _ in 0..num_bits {
            c <<= 1;
            if self.next_bit() {
                c |= 1;
            }
        }
        c
    }

    fn next_bit(&mut self) -> bool {
        let cf = self.rcr(false);
        if self.chk == 0 {
            self.chk = self.read_be_uint32();
            self.crc ^= self.chk;
            self.rcr(true)
        } else {
            cf
        }
    }

    /// Right-rotate `chk` by 1 bit; the bit that fell off the right is
    /// returned, and `cf_in` is rotated in at the top.
    fn rcr(&mut self, cf_in: bool) -> bool {
        let r_cf = (self.chk & 1) != 0;
        self.chk >>= 1;
        if cf_in {
            self.chk |= 0x8000_0000;
        }
        r_cf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_too_short_input() {
        assert_eq!(unpack(&[0u8; 8]), UnpackResult::CrcFailure);
    }
}
