//! Bounded, dependency-free RFC 8878 frame decoder foundation.
//!
//! This first slice accepts one standard frame containing raw and RLE blocks.
//! Compressed blocks, dictionaries, checksums, skippable frames, and concatenated
//! frames fail with explicit errors until their independently reviewed slices land.

use super::MAX_LAYER_BYTES;

const MAGIC: [u8; 4] = [0x28, 0xb5, 0x2f, 0xfd];
const MAX_INPUT_BYTES: usize = MAX_LAYER_BYTES;
const MAX_BLOCK_BYTES: usize = 131_072;
const MAX_BLOCKS: usize = 8_192;
const MAX_WINDOW_BYTES: usize = 8_388_608;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InputTooLarge,
    OutputLimitTooLarge,
    Truncated,
    InvalidMagic,
    ReservedDescriptorBit,
    UnsupportedDictionary,
    UnsupportedChecksum,
    WindowTooLarge,
    ContentSizeTooLarge,
    InvalidBlock,
    TooManyBlocks,
    InvalidLiteralsSection,
    UnsupportedLiteralsCompression,
    UnsupportedSequences,
    TrailingBlockData,
    OutputTooLarge,
    ContentSizeMismatch,
    TrailingData,
}

/// Decodes exactly one dictionary-free, checksum-free Zstandard frame.
///
/// Raw and RLE blocks are supported. All other frame features fail closed with
/// a distinct error and no partially decoded output is returned.
pub fn decode_zstd(input: &[u8], maximum_output_bytes: usize) -> Result<Vec<u8>, Error> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(Error::InputTooLarge);
    }
    if maximum_output_bytes > MAX_LAYER_BYTES {
        return Err(Error::OutputLimitTooLarge);
    }

    let mut reader = Reader::new(input);
    if reader.take(4)? != MAGIC.as_slice() {
        return Err(Error::InvalidMagic);
    }
    let descriptor = reader.byte()?;
    if descriptor & 0x08 != 0 {
        return Err(Error::ReservedDescriptorBit);
    }

    let content_size_flag = descriptor >> 6;
    let single_segment = descriptor & 0x20 != 0;
    let checksum = descriptor & 0x04 != 0;
    let dictionary_size = match descriptor & 0x03 {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 4,
        _ => unreachable!(),
    };

    let window_size = if single_segment {
        None
    } else {
        let window_descriptor = reader.byte()?;
        let exponent = u32::from(window_descriptor >> 3);
        let mantissa = usize::from(window_descriptor & 0x07);
        let base = 1usize
            .checked_shl(10 + exponent)
            .ok_or(Error::WindowTooLarge)?;
        let add = (base / 8)
            .checked_mul(mantissa)
            .ok_or(Error::WindowTooLarge)?;
        let size = base.checked_add(add).ok_or(Error::WindowTooLarge)?;
        if size > MAX_WINDOW_BYTES {
            return Err(Error::WindowTooLarge);
        }
        Some(size)
    };

    if dictionary_size != 0 && read_little_endian(&mut reader, dictionary_size)? != 0 {
        return Err(Error::UnsupportedDictionary);
    }

    let content_size_bytes = match content_size_flag {
        0 if single_segment => 1,
        0 => 0,
        1 => 2,
        2 => 4,
        3 => 8,
        _ => unreachable!(),
    };
    let content_size = if content_size_bytes == 0 {
        None
    } else {
        let encoded = read_little_endian(&mut reader, content_size_bytes)?;
        let size = if content_size_bytes == 2 {
            encoded.checked_add(256).ok_or(Error::ContentSizeTooLarge)?
        } else {
            encoded
        };
        let size = usize::try_from(size).map_err(|_| Error::ContentSizeTooLarge)?;
        if size > maximum_output_bytes {
            return Err(Error::ContentSizeTooLarge);
        }
        Some(size)
    };

    if checksum {
        return Err(Error::UnsupportedChecksum);
    }

    let block_maximum = window_size
        .or(content_size)
        .unwrap_or(MAX_BLOCK_BYTES)
        .min(MAX_BLOCK_BYTES);
    let mut output = Vec::new();
    let mut block_count = 0usize;
    loop {
        block_count = block_count.checked_add(1).ok_or(Error::TooManyBlocks)?;
        if block_count > MAX_BLOCKS {
            return Err(Error::TooManyBlocks);
        }
        let header_bytes = reader.take(3)?;
        let header = u32::from(header_bytes[0])
            | (u32::from(header_bytes[1]) << 8)
            | (u32::from(header_bytes[2]) << 16);
        let last = header & 1 != 0;
        let block_type = (header >> 1) & 0x03;
        let block_size = usize::try_from(header >> 3).map_err(|_| Error::InvalidBlock)?;
        if block_size > block_maximum {
            return Err(Error::InvalidBlock);
        }
        match block_type {
            0 => extend_bounded(
                &mut output,
                reader.take(block_size)?,
                maximum_output_bytes,
            )?,
            1 => {
                let byte = reader.byte()?;
                resize_bounded(&mut output, block_size, byte, maximum_output_bytes)?;
            }
            2 => {
                let block = reader.take(block_size)?;
                decode_literal_only_compressed_block(
                    block,
                    &mut output,
                    block_maximum,
                    maximum_output_bytes,
                )?;
            }
            _ => return Err(Error::InvalidBlock),
        }
        if last {
            break;
        }
    }

    if reader.remaining() != 0 {
        return Err(Error::TrailingData);
    }
    if content_size.is_some_and(|expected| expected != output.len()) {
        return Err(Error::ContentSizeMismatch);
    }
    Ok(output)
}

fn decode_literal_only_compressed_block(
    block: &[u8],
    output: &mut Vec<u8>,
    block_maximum: usize,
    maximum_output_bytes: usize,
) -> Result<(), Error> {
    let mut reader = Reader::new(block);
    let first = reader.byte()?;
    let literals_type = first & 0x03;
    if literals_type >= 2 {
        return Err(Error::UnsupportedLiteralsCompression);
    }
    let size_format = (first >> 2) & 0x03;
    let regenerated_size = match size_format {
        0 | 2 => usize::from(first >> 3),
        1 => {
            usize::from(first >> 4) | (usize::from(reader.byte()?) << 4)
        }
        3 => {
            usize::from(first >> 4)
                | (usize::from(reader.byte()?) << 4)
                | (usize::from(reader.byte()?) << 12)
        }
        _ => unreachable!(),
    };
    if regenerated_size > block_maximum {
        return Err(Error::InvalidLiteralsSection);
    }

    let mut literals = Vec::new();
    match literals_type {
        0 => extend_bounded(
            &mut literals,
            reader.take(regenerated_size)?,
            regenerated_size,
        )?,
        1 => resize_bounded(
            &mut literals,
            regenerated_size,
            reader.byte()?,
            regenerated_size,
        )?,
        _ => unreachable!(),
    }

    if reader.byte()? != 0 {
        return Err(Error::UnsupportedSequences);
    }
    if reader.remaining() != 0 {
        return Err(Error::TrailingBlockData);
    }
    extend_bounded(output, &literals, maximum_output_bytes)
}

fn read_little_endian(reader: &mut Reader<'_>, count: usize) -> Result<u64, Error> {
    let bytes = reader.take(count)?;
    let mut value = 0u64;
    for (shift, byte) in bytes.iter().enumerate() {
        value |= u64::from(*byte) << (shift * 8);
    }
    Ok(value)
}

fn extend_bounded(output: &mut Vec<u8>, bytes: &[u8], maximum: usize) -> Result<(), Error> {
    if output
        .len()
        .checked_add(bytes.len())
        .is_none_or(|length| length > maximum)
    {
        return Err(Error::OutputTooLarge);
    }
    output.extend_from_slice(bytes);
    Ok(())
}

fn resize_bounded(
    output: &mut Vec<u8>,
    additional: usize,
    byte: u8,
    maximum: usize,
) -> Result<(), Error> {
    let length = output
        .len()
        .checked_add(additional)
        .ok_or(Error::OutputTooLarge)?;
    if length > maximum {
        return Err(Error::OutputTooLarge);
    }
    output.resize(length, byte);
    Ok(())
}

struct Reader<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, offset: 0 }
    }

    fn byte(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], Error> {
        let end = self.offset.checked_add(count).ok_or(Error::Truncated)?;
        let bytes = self.input.get(self.offset..end).ok_or(Error::Truncated)?;
        self.offset = end;
        Ok(bytes)
    }

    fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_single_segment_raw_blocks() {
        let frame = single_segment_frame(&[(false, 0, b"abc"), (true, 0, b"def")], 6);
        assert_eq!(decode_zstd(&frame, 6).unwrap(), b"abcdef");
    }

    #[test]
    fn decodes_windowed_rle_blocks() {
        let mut frame = MAGIC.to_vec();
        frame.extend_from_slice(&[0x00, 0x00]);
        append_block(&mut frame, true, 1, 5, b"x");
        assert_eq!(decode_zstd(&frame, 5).unwrap(), b"xxxxx");
    }

    #[test]
    fn enforces_content_window_block_and_output_bounds() {
        let frame = single_segment_frame(&[(true, 0, b"abc")], 3);
        assert_eq!(decode_zstd(&frame, 2), Err(Error::ContentSizeTooLarge));

        let mut large_window = MAGIC.to_vec();
        large_window.extend_from_slice(&[0x00, 0xff]);
        assert_eq!(decode_zstd(&large_window, MAX_LAYER_BYTES), Err(Error::WindowTooLarge));

        let mut oversized_block = MAGIC.to_vec();
        oversized_block.extend_from_slice(&[0x00, 0x00]);
        append_block(&mut oversized_block, true, 1, 1025, b"x");
        assert_eq!(decode_zstd(&oversized_block, 2048), Err(Error::InvalidBlock));

        let rle = single_segment_frame(&[(true, 1, b"x")], 5);
        assert_eq!(decode_zstd(&rle, 4), Err(Error::ContentSizeTooLarge));
    }

    #[test]
    fn rejects_unsupported_or_reserved_features() {
        let mut compressed = MAGIC.to_vec();
        compressed.extend_from_slice(&[0x20, 0x00]);
        append_block(&mut compressed, true, 2, 0, b"");
        assert_eq!(
            decode_zstd(&compressed, 0),
            Err(Error::UnsupportedLiteralsCompression)
        );

        for descriptor_error in [
            (0x28, Error::ReservedDescriptorBit),
            (0x21, Error::UnsupportedDictionary),
            (0x24, Error::UnsupportedChecksum),
        ] {
            let mut frame = MAGIC.to_vec();
            frame.push(descriptor_error.0);
            if descriptor_error.0 & 1 != 0 {
                frame.push(1);
            }
            frame.push(0);
            assert_eq!(decode_zstd(&frame, 0), Err(descriptor_error.1));
        }

        let mut reserved_block = MAGIC.to_vec();
        reserved_block.extend_from_slice(&[0x20, 0x00]);
        append_block(&mut reserved_block, true, 3, 0, b"");
        assert_eq!(decode_zstd(&reserved_block, 0), Err(Error::InvalidBlock));
    }

    #[test]
    fn rejects_truncation_trailing_data_and_too_many_blocks() {
        let valid = single_segment_frame(&[(true, 0, b"abc")], 3);
        for end in 0..valid.len() {
            assert!(decode_zstd(&valid[..end], 3).is_err());
        }
        let mut trailing = valid.clone();
        trailing.push(0);
        assert_eq!(decode_zstd(&trailing, 3), Err(Error::TrailingData));

        let mut many = MAGIC.to_vec();
        many.extend_from_slice(&[0x20, 0x00]);
        for _ in 0..MAX_BLOCKS {
            append_block(&mut many, false, 0, 0, b"");
        }
        append_block(&mut many, true, 0, 0, b"");
        assert_eq!(decode_zstd(&many, 0), Err(Error::TooManyBlocks));
    }

    #[test]
    fn covers_fcs_widths_zero_dictionary_ids_and_runtime_output_limit() {
        for (descriptor, encoded_size) in [
            (0x60, 44u64),
            (0xa0, 300u64),
            (0xe0, 300u64),
        ] {
            let fcs_bytes = match descriptor >> 6 {
                1 => 2,
                2 => 4,
                3 => 8,
                _ => unreachable!(),
            };
            let mut frame = MAGIC.to_vec();
            frame.push(descriptor);
            frame.extend_from_slice(&encoded_size.to_le_bytes()[..fcs_bytes]);
            let payload = vec![b'x'; 300];
            append_block(&mut frame, true, 0, payload.len(), &payload);
            assert_eq!(decode_zstd(&frame, 300).unwrap(), payload);
        }

        for (descriptor, dictionary_bytes) in [(0x01, 1), (0x02, 2), (0x03, 4)] {
            let mut frame = MAGIC.to_vec();
            frame.extend_from_slice(&[descriptor, 0x00]);
            frame.resize(frame.len() + dictionary_bytes, 0);
            append_block(&mut frame, true, 0, 3, b"abc");
            assert_eq!(decode_zstd(&frame, 3).unwrap(), b"abc");
        }

        let mismatch = single_segment_frame(&[(true, 0, b"abc")], 4);
        assert_eq!(decode_zstd(&mismatch, 4), Err(Error::ContentSizeMismatch));

        let mut unbounded_by_fcs = MAGIC.to_vec();
        unbounded_by_fcs.extend_from_slice(&[0x00, 0x00]);
        append_block(&mut unbounded_by_fcs, true, 1, 5, b"x");
        assert_eq!(
            decode_zstd(&unbounded_by_fcs, 4),
            Err(Error::OutputTooLarge)
        );
    }

    #[test]
    fn decodes_literal_only_compressed_blocks_and_size_formats() {
        let raw = compressed_frame(&[0x18, b'a', b'b', b'c', 0]);
        assert_eq!(decode_zstd(&raw, 3).unwrap(), b"abc");

        let rle = compressed_frame(&[0x29, b'x', 0]);
        assert_eq!(decode_zstd(&rle, 5).unwrap(), b"xxxxx");

        let payload_40 = vec![b'y'; 40];
        let mut format_01 = vec![0x84, 0x02];
        format_01.extend_from_slice(&payload_40);
        format_01.push(0);
        assert_eq!(
            decode_zstd(&compressed_frame(&format_01), 40).unwrap(),
            payload_40
        );

        let payload_4096 = vec![b'z'; 4096];
        let mut format_11 = vec![0x0c, 0x00, 0x01];
        format_11.extend_from_slice(&payload_4096);
        format_11.push(0);
        assert_eq!(
            decode_zstd(&compressed_frame(&format_11), 4096).unwrap(),
            payload_4096
        );

        let format_10 = compressed_frame(&[0x08, b'q', 0]);
        assert_eq!(decode_zstd(&format_10, 1).unwrap(), b"q");
    }

    #[test]
    fn rejects_malformed_or_sequence_bearing_compressed_blocks() {
        let sequence = compressed_frame(&[0x08, b'x', 1]);
        assert_eq!(decode_zstd(&sequence, 1), Err(Error::UnsupportedSequences));

        let trailing = compressed_frame(&[0x08, b'x', 0, 0]);
        assert_eq!(decode_zstd(&trailing, 1), Err(Error::TrailingBlockData));

        let truncated = compressed_frame(&[0x18, b'a', b'b']);
        assert_eq!(decode_zstd(&truncated, 3), Err(Error::Truncated));

        let oversized = compressed_frame(&[0x1c, 0x00, 0x02, 0]);
        assert_eq!(
            decode_zstd(&oversized, 1),
            Err(Error::InvalidLiteralsSection)
        );

        for literals_type in [2u8, 3u8] {
            let frame = compressed_frame(&[literals_type, 0]);
            assert_eq!(
                decode_zstd(&frame, 2),
                Err(Error::UnsupportedLiteralsCompression)
            );
        }
    }

    fn single_segment_frame(blocks: &[(bool, u32, &[u8])], content_size: u8) -> Vec<u8> {
        let mut frame = MAGIC.to_vec();
        frame.extend_from_slice(&[0x20, content_size]);
        for (last, block_type, content) in blocks {
            let regenerated_size = if *block_type == 1 {
                usize::from(content_size)
            } else {
                content.len()
            };
            append_block(&mut frame, *last, *block_type, regenerated_size, content);
        }
        frame
    }

    fn compressed_frame(block: &[u8]) -> Vec<u8> {
        let mut frame = MAGIC.to_vec();
        frame.extend_from_slice(&[0x00, 0x18]);
        append_block(&mut frame, true, 2, block.len(), block);
        frame
    }

    fn append_block(
        frame: &mut Vec<u8>,
        last: bool,
        block_type: u32,
        block_size: usize,
        content: &[u8],
    ) {
        let header = ((block_size as u32) << 3)
            | (block_type << 1)
            | if last { 1 } else { 0 };
        frame.extend_from_slice(&header.to_le_bytes()[..3]);
        frame.extend_from_slice(content);
    }
}
