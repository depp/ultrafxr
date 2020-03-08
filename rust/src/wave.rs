use crate::rand::Rand;
use std::cmp::min;
use std::io::{Result as IOResult, Seek, SeekFrom, Write};

/// Parameters for a WAVE file.
#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    pub channel_count: u32,
    pub sample_rate: u32,
}

trait WriteBytes {
    fn write_bytes(&self, buf: &mut [u8]) -> usize;
}

impl WriteBytes for [u8; 4] {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        buf[..4].copy_from_slice(&self[..]);
        4
    }
}

impl WriteBytes for u32 {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        buf[..4].copy_from_slice(&self.to_le_bytes()[..]);
        4
    }
}

impl WriteBytes for u16 {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        buf[..2].copy_from_slice(&self.to_le_bytes()[..]);
        2
    }
}

macro_rules! data {
    ($len:literal, $($type:ty: $value:expr),*,) => ({
        let mut _arr: [u8; $len] = [0; $len];
        let mut _pos: usize = 0;
        $(
            _pos += <$type>::write_bytes(&$value, &mut _arr[_pos..]);
        )*
        debug_assert_eq!(_pos, $len);
        _arr
    });
}

struct Header {
    frame_count: u32,
    parameters: Parameters,
}

impl Header {
    fn to_bytes(&self) -> [u8; 44] {
        let bits_per_byte: u32 = 8;
        let sample_size_bytes: u32 = 2;
        let frame_size_bytes: u32 = self.parameters.channel_count * sample_size_bytes;
        let data_length_bytes: u32 = self.frame_count * frame_size_bytes;
        data![
            44,
            [u8;4]: *b"RIFF", // Chunk ID
            u32: data_length_bytes + 36, // ChunkSize
            [u8;4]: *b"WAVE", // Format
            [u8;4]: *b"fmt ", // Subchunk ID
            u32: 16, // Subchunk size
            u16: 1, // Format: 1 => PCM
            u16: self.parameters.channel_count as u16,
            u32: self.parameters.sample_rate,
            u32: self.parameters.sample_rate * frame_size_bytes, // Byte rate
            u16: frame_size_bytes as u16, // Bytes per frame
            u16: (sample_size_bytes * bits_per_byte) as u16, // Bits per sample
            [u8;4]: *b"data", // Subchunk ID
            u32: data_length_bytes, // Subchunk size
        ]
    }
}

/// Trait for streams that can both seek and write.
pub trait SeekWrite: Seek + Write {}

impl<T> SeekWrite for T
where
    T: Seek,
    T: Write,
{
}

/// WAVE file writer.
pub struct Writer<'a> {
    stream: &'a mut dyn SeekWrite,
    buf: Box<[u8]>,
    buf_pos: usize,
    sample_count: usize,
    rand: Rand,
    parameters: Parameters,
}

impl<'a> Writer<'a> {
    /// Create a WAVE writer from the given stream.
    pub fn from_stream(stream: &'a mut dyn SeekWrite, parameters: &Parameters) -> Self {
        const BUFFER_SIZE: usize = 32 * 1024;
        let mut buf = Vec::<u8>::new();
        buf.resize(BUFFER_SIZE, 0);
        Writer {
            stream,
            buf: Box::from(buf),
            buf_pos: 44,
            sample_count: 0,
            rand: Rand::with_default_seed(),
            parameters: *parameters,
        }
    }

    /// Write floating-point samples to the file. These samples will be
    /// converted to 16-bit.
    pub fn write(&mut self, data: &[f32]) -> IOResult<()> {
        let mut data = data;
        let buf = &mut self.buf[..];
        while !data.is_empty() {
            {
                let buf = &mut buf[self.buf_pos..];
                let n = min(data.len(), buf.len() / 2);
                let (first, rest) = data.split_at(n);
                for (&x, y) in first.iter().zip(buf.chunks_mut(2)) {
                    // Random variable with rectangular distribution for dithering.
                    let r = (self.rand.next() as f32) * (1.0 / 4294967296.0);
                    let x = (x * 32768.0 + r).floor();
                    let x = if x > i16::max_value() as f32 {
                        i16::max_value()
                    } else if x < i16::min_value() as f32 {
                        i16::min_value()
                    } else {
                        x as i16
                    };
                    y.copy_from_slice(&x.to_le_bytes()[..]);
                }
                data = rest;
                self.buf_pos += n * 2;
                self.sample_count += n;
            }
            if self.buf_pos == buf.len() {
                self.stream.write_all(buf)?;
                self.buf_pos = 0;
            }
        }
        Ok(())
    }

    /// Finish writing the file.
    pub fn finish(self) -> IOResult<()> {
        if self.buf_pos > 0 {
            self.stream.write_all(&self.buf[..self.buf_pos])?;
        }
        let header = Header {
            frame_count: (self.sample_count / (self.parameters.channel_count as usize)) as u32,
            parameters: self.parameters,
        };
        let header = header.to_bytes();
        self.stream.seek(SeekFrom::Start(0))?;
        self.stream.write_all(&header[..])
    }
}
