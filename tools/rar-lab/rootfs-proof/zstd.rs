//! Bounded, dependency-free RFC 8878 frame decoder foundation.
//!
//! This decoder accepts raw and RLE blocks plus zero-sequence compressed blocks
//! containing raw, RLE, or direct-weight one- or four-stream Huffman literals.
//! Dictionaries, checksums, sequence FSE, skippable frames, and concatenated frames fail
//! with explicit errors.

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
    InvalidHuffmanTree,
    InvalidHuffmanStream,
    InvalidFseTable,
    InvalidFseStream,
    UnsupportedLiteralsFormat,
    UnsupportedLiteralsCompression,
    UnsupportedSequences,
    TrailingBlockData,
    OutputTooLarge,
    ContentSizeMismatch,
    TrailingData,
}

/// Decodes exactly one dictionary-free, checksum-free Zstandard frame.
///
/// Raw/RLE blocks and zero-sequence compressed blocks with raw, RLE, or
/// direct-weight one- or four-stream Huffman literals are supported. Other features
/// fail closed with a distinct error, and no partial output is returned.
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
    let size_format = (first >> 2) & 0x03;
    let (regenerated_size, literals) = match literals_type {
        0 | 1 => {
            let regenerated_size = match size_format {
                0 | 2 => usize::from(first >> 3),
                1 => usize::from(first >> 4) | (usize::from(reader.byte()?) << 4),
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
            (regenerated_size, literals)
        }
        2 => {
            let header_bytes = match size_format {
                0 | 1 => 3,
                2 => 4,
                3 => 5,
                _ => unreachable!(),
            };
            let mut header = u64::from(first);
            for index in 1..header_bytes {
                header |= u64::from(reader.byte()?) << (index * 8);
            }
            let field_bits = match size_format {
                0 | 1 => 10,
                2 => 14,
                3 => 18,
                _ => unreachable!(),
            };
            let field_mask = (1u64 << field_bits) - 1;
            let regenerated_size = usize::try_from((header >> 4) & field_mask)
                .map_err(|_| Error::InvalidLiteralsSection)?;
            let compressed_size = usize::try_from((header >> (4 + field_bits)) & field_mask)
                .map_err(|_| Error::InvalidLiteralsSection)?;
            // Verified RFC 8878 Errata 7297 makes 6 the minimum for both
            // fields in every four-stream compressed-literals format.
            if size_format != 0 && (regenerated_size < 6 || compressed_size < 6) {
                return Err(Error::InvalidLiteralsSection);
            }
            if regenerated_size > block_maximum || compressed_size > block_maximum {
                return Err(Error::InvalidLiteralsSection);
            }
            let literals = decode_direct_huffman_literals(
                reader.take(compressed_size)?,
                regenerated_size,
                size_format != 0,
            )?;
            (regenerated_size, literals)
        }
        3 => return Err(Error::UnsupportedLiteralsCompression),
        _ => unreachable!(),
    };
    if regenerated_size > block_maximum {
        return Err(Error::InvalidLiteralsSection);
    }

    if reader.byte()? != 0 {
        return Err(Error::UnsupportedSequences);
    }
    if reader.remaining() != 0 {
        return Err(Error::TrailingBlockData);
    }
    extend_bounded(output, &literals, maximum_output_bytes)
}

fn decode_direct_huffman_literals(
    content: &[u8],
    regenerated_size: usize,
    four_streams: bool,
) -> Result<Vec<u8>, Error> {
    let mut reader = Reader::new(content);
    let header = reader.byte()?;
    let mut weights = if header >= 128 {
        let described_symbols = usize::from(header - 127);
        let encoded_weight_bytes = described_symbols.div_ceil(2);
        let encoded_weights = reader.take(encoded_weight_bytes)?;
        let mut weights = Vec::with_capacity(described_symbols + 1);
        for symbol in 0..described_symbols {
            let byte = encoded_weights[symbol / 2];
            weights.push(if symbol % 2 == 0 { byte >> 4 } else { byte & 0x0f });
        }
        weights
    } else {
        let description = reader.take(usize::from(header))?;
        decode_fse_weights(description)?
    };

    let mut total = 0usize;
    for weight in &weights {
        if *weight > 11 {
            return Err(Error::InvalidHuffmanTree);
        }
        if *weight != 0 {
            total = total
                .checked_add(1usize << (u32::from(*weight) - 1))
                .ok_or(Error::InvalidHuffmanTree)?;
        }
    }
    if total == 0 {
        return Err(Error::InvalidHuffmanTree);
    }
    let mut table_size = total
        .checked_next_power_of_two()
        .ok_or(Error::InvalidHuffmanTree)?;
    if table_size == total {
        table_size = table_size.checked_mul(2).ok_or(Error::InvalidHuffmanTree)?;
    }
    let inferred = table_size - total;
    if !inferred.is_power_of_two() {
        return Err(Error::InvalidHuffmanTree);
    }
    let inferred_weight = inferred.trailing_zeros() + 1;
    let maximum_bits = table_size.trailing_zeros();
    if maximum_bits == 0 || maximum_bits > 11 || inferred_weight > maximum_bits {
        return Err(Error::InvalidHuffmanTree);
    }
    weights.push(inferred_weight as u8);

    let maximum_bits = maximum_bits as usize;
    let mut rank_count = [0usize; 12];
    for weight in weights.iter().copied().filter(|weight| *weight != 0) {
        let length = maximum_bits + 1 - usize::from(weight);
        rank_count[length] = rank_count[length]
            .checked_add(1)
            .ok_or(Error::InvalidHuffmanTree)?;
    }

    // RFC 8878 assigns ranges from the longest codes toward the shortest,
    // preserving natural symbol order within each rank.  Building the full
    // bounded lookup table also keeps every symbol decode O(1).
    let mut rank_index = [0usize; 12];
    for length in (1..=maximum_bits).rev() {
        let width = 1usize << (maximum_bits - length);
        rank_index[length - 1] = rank_index[length]
            .checked_add(
                rank_count[length]
                    .checked_mul(width)
                    .ok_or(Error::InvalidHuffmanTree)?,
            )
            .ok_or(Error::InvalidHuffmanTree)?;
    }
    if rank_index[0] != table_size {
        return Err(Error::InvalidHuffmanTree);
    }

    let mut table = vec![None; table_size];
    for (symbol, weight) in weights.iter().copied().enumerate() {
        if weight == 0 {
            continue;
        }
        let length = maximum_bits + 1 - usize::from(weight);
        let first = rank_index[length];
        let count = 1usize << (maximum_bits - length);
        let end = first
            .checked_add(count)
            .ok_or(Error::InvalidHuffmanTree)?;
        let slots = table
            .get_mut(first..end)
            .ok_or(Error::InvalidHuffmanTree)?;
        if slots.iter().any(Option::is_some) {
            return Err(Error::InvalidHuffmanTree);
        }
        slots.fill(Some(HuffmanEntry {
            symbol: symbol as u8,
            length: length as u8,
        }));
        rank_index[length] = end;
    }
    if table.iter().any(Option::is_none) {
        return Err(Error::InvalidHuffmanTree);
    }

    if !four_streams {
        let stream = reader.take(reader.remaining())?;
        return decode_huffman_stream(&table, maximum_bits as u8, stream, regenerated_size);
    }
    let first_size = usize::try_from(read_little_endian(&mut reader, 2)?)
        .map_err(|_| Error::InvalidHuffmanStream)?;
    let second_size = usize::try_from(read_little_endian(&mut reader, 2)?)
        .map_err(|_| Error::InvalidHuffmanStream)?;
    let third_size = usize::try_from(read_little_endian(&mut reader, 2)?)
        .map_err(|_| Error::InvalidHuffmanStream)?;
    let declared = first_size
        .checked_add(second_size)
        .and_then(|size| size.checked_add(third_size))
        .ok_or(Error::InvalidHuffmanStream)?;
    if declared > reader.remaining() {
        return Err(Error::InvalidHuffmanStream);
    }
    let fourth_size = reader.remaining() - declared;
    let streams = [
        reader.take(first_size)?,
        reader.take(second_size)?,
        reader.take(third_size)?,
        reader.take(fourth_size)?,
    ];
    let stream_output_size = regenerated_size.div_ceil(4);
    let fourth_output_size = regenerated_size
        .checked_sub(stream_output_size * 3)
        .ok_or(Error::InvalidLiteralsSection)?;
    let output_sizes = [
        stream_output_size,
        stream_output_size,
        stream_output_size,
        fourth_output_size,
    ];
    let mut output = Vec::with_capacity(regenerated_size);
    for (stream, output_size) in streams.into_iter().zip(output_sizes) {
        let decoded = decode_huffman_stream(&table, maximum_bits as u8, stream, output_size)?;
        output.extend_from_slice(&decoded);
    }
    Ok(output)
}

struct FseTable {
    symbols: Vec<u8>,
    bit_counts: Vec<u8>,
    bases: Vec<u16>,
    accuracy_log: u8,
}

fn decode_fse_weights(description: &[u8]) -> Result<Vec<u8>, Error> {
    let mut header = ForwardBits::new(description);
    let accuracy_log = u8::try_from(header.read(4)? + 5).map_err(|_| Error::InvalidFseTable)?;
    if accuracy_log > 7 {
        return Err(Error::InvalidFseTable);
    }
    let mut remaining = 1i32 << accuracy_log;
    let mut frequencies = Vec::new();
    while remaining > 0 {
        if frequencies.len() >= 256 {
            return Err(Error::InvalidFseTable);
        }
        let bits = (32 - u32::try_from(remaining + 1)
            .map_err(|_| Error::InvalidFseTable)?
            .leading_zeros()) as u8;
        let mut value = header.read(bits)?;
        let lower_mask = (1u16 << (bits - 1)) - 1;
        let threshold = (1u16 << bits)
            .checked_sub(1)
            .and_then(|value| value.checked_sub((remaining + 1) as u16))
            .ok_or(Error::InvalidFseTable)?;
        if value & lower_mask < threshold {
            header.rewind_one()?;
            value &= lower_mask;
        } else if value > lower_mask {
            value = value
                .checked_sub(threshold)
                .ok_or(Error::InvalidFseTable)?;
        }
        let probability = i16::try_from(value).map_err(|_| Error::InvalidFseTable)? - 1;
        let magnitude = if probability < 0 {
            -i32::from(probability)
        } else {
            i32::from(probability)
        };
        remaining = remaining
            .checked_sub(magnitude)
            .ok_or(Error::InvalidFseTable)?;
        if remaining < 0 {
            return Err(Error::InvalidFseTable);
        }
        frequencies.push(probability);

        if probability == 0 {
            loop {
                let repeat = usize::from(header.read(2)?);
                if frequencies.len().checked_add(repeat).is_none_or(|size| size > 256) {
                    return Err(Error::InvalidFseTable);
                }
                frequencies.resize(frequencies.len() + repeat, 0);
                if repeat != 3 {
                    break;
                }
            }
        }
    }
    let stream_offset = header.align_to_byte()?;
    if frequencies.len() >= 256 || stream_offset >= description.len() {
        return Err(Error::InvalidFseTable);
    }
    let table = build_fse_table(&frequencies, accuracy_log)?;
    decode_fse_interleaved2(&table, &description[stream_offset..])
}

fn build_fse_table(frequencies: &[i16], accuracy_log: u8) -> Result<FseTable, Error> {
    let size = 1usize << accuracy_log;
    let mask = size - 1;
    let mut symbols = vec![None; size];
    let mut state_descriptions = vec![0u16; frequencies.len()];
    let mut high_threshold = size;
    for (symbol, frequency) in frequencies.iter().copied().enumerate() {
        if frequency < -1 {
            return Err(Error::InvalidFseTable);
        }
        if frequency == -1 {
            high_threshold = high_threshold.checked_sub(1).ok_or(Error::InvalidFseTable)?;
            symbols[high_threshold] = Some(symbol as u8);
            state_descriptions[symbol] = 1;
        }
    }

    let step = (size >> 1) + (size >> 3) + 3;
    let mut position = 0usize;
    for (symbol, frequency) in frequencies.iter().copied().enumerate() {
        if frequency <= 0 {
            continue;
        }
        state_descriptions[symbol] = frequency as u16;
        for _ in 0..frequency {
            if position >= high_threshold || symbols[position].is_some() {
                return Err(Error::InvalidFseTable);
            }
            symbols[position] = Some(symbol as u8);
            let mut probes = 0usize;
            loop {
                position = (position + step) & mask;
                probes += 1;
                if position < high_threshold {
                    break;
                }
                if probes > size {
                    return Err(Error::InvalidFseTable);
                }
            }
        }
    }
    if position != 0 || symbols.iter().any(Option::is_none) {
        return Err(Error::InvalidFseTable);
    }

    let symbols: Vec<u8> = symbols.into_iter().map(Option::unwrap).collect();
    let mut bit_counts = Vec::with_capacity(size);
    let mut bases = Vec::with_capacity(size);
    for symbol in &symbols {
        let description = state_descriptions
            .get_mut(usize::from(*symbol))
            .ok_or(Error::InvalidFseTable)?;
        if *description == 0 {
            return Err(Error::InvalidFseTable);
        }
        let highest = 15 - description.leading_zeros() as u8;
        let bit_count = accuracy_log
            .checked_sub(highest)
            .ok_or(Error::InvalidFseTable)?;
        let base = description
            .checked_shl(u32::from(bit_count))
            .and_then(|value| value.checked_sub(size as u16))
            .ok_or(Error::InvalidFseTable)?;
        bit_counts.push(bit_count);
        bases.push(base);
        *description = description.checked_add(1).ok_or(Error::InvalidFseTable)?;
    }
    Ok(FseTable {
        symbols,
        bit_counts,
        bases,
        accuracy_log,
    })
}

fn decode_fse_interleaved2(table: &FseTable, stream: &[u8]) -> Result<Vec<u8>, Error> {
    let mut bits = PaddedReverseBits::new(stream)?;
    let mut first = bits.read(table.accuracy_log)?;
    let mut second = bits.read(table.accuracy_log)?;
    let mut output = Vec::new();
    loop {
        decode_fse_state(table, &mut bits, &mut first, &mut output)?;
        if bits.remaining < 0 {
            push_fse_symbol(table, second, &mut output)?;
            break;
        }
        decode_fse_state(table, &mut bits, &mut second, &mut output)?;
        if bits.remaining < 0 {
            push_fse_symbol(table, first, &mut output)?;
            break;
        }
    }
    Ok(output)
}

fn decode_fse_state(
    table: &FseTable,
    bits: &mut PaddedReverseBits<'_>,
    state: &mut usize,
    output: &mut Vec<u8>,
) -> Result<(), Error> {
    push_fse_symbol(table, *state, output)?;
    let count = *table.bit_counts.get(*state).ok_or(Error::InvalidFseStream)?;
    let base = usize::from(*table.bases.get(*state).ok_or(Error::InvalidFseStream)?);
    *state = base
        .checked_add(bits.read(count)?)
        .ok_or(Error::InvalidFseStream)?;
    if *state >= table.symbols.len() {
        return Err(Error::InvalidFseStream);
    }
    Ok(())
}

fn push_fse_symbol(table: &FseTable, state: usize, output: &mut Vec<u8>) -> Result<(), Error> {
    if output.len() >= 255 {
        return Err(Error::InvalidFseStream);
    }
    output.push(*table.symbols.get(state).ok_or(Error::InvalidFseStream)?);
    Ok(())
}

struct ForwardBits<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> ForwardBits<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, offset: 0 }
    }

    fn read(&mut self, count: u8) -> Result<u16, Error> {
        if count == 0 || count > 15 {
            return Err(Error::InvalidFseTable);
        }
        let end = self
            .offset
            .checked_add(usize::from(count))
            .ok_or(Error::InvalidFseTable)?;
        if end > self.input.len().saturating_mul(8) {
            return Err(Error::Truncated);
        }
        let mut value = 0u16;
        for shift in 0..count {
            let bit_offset = self.offset + usize::from(shift);
            value |= u16::from((self.input[bit_offset / 8] >> (bit_offset % 8)) & 1) << shift;
        }
        self.offset = end;
        Ok(value)
    }

    fn rewind_one(&mut self) -> Result<(), Error> {
        self.offset = self.offset.checked_sub(1).ok_or(Error::InvalidFseTable)?;
        Ok(())
    }

    fn align_to_byte(&mut self) -> Result<usize, Error> {
        self.offset = self
            .offset
            .checked_add(7)
            .map(|offset| offset & !7)
            .ok_or(Error::InvalidFseTable)?;
        let bytes = self.offset / 8;
        if bytes > self.input.len() {
            return Err(Error::Truncated);
        }
        Ok(bytes)
    }
}

struct PaddedReverseBits<'a> {
    input: &'a [u8],
    next_bit: isize,
    remaining: isize,
}

impl<'a> PaddedReverseBits<'a> {
    fn new(input: &'a [u8]) -> Result<Self, Error> {
        let last = *input.last().ok_or(Error::InvalidFseStream)?;
        if last == 0 {
            return Err(Error::InvalidFseStream);
        }
        let marker = 7 - last.leading_zeros() as usize;
        let remaining = (input.len() - 1)
            .checked_mul(8)
            .and_then(|bits| bits.checked_add(marker))
            .and_then(|bits| isize::try_from(bits).ok())
            .ok_or(Error::InvalidFseStream)?;
        Ok(Self {
            input,
            next_bit: remaining - 1,
            remaining,
        })
    }

    fn read(&mut self, count: u8) -> Result<usize, Error> {
        let mut value = 0usize;
        for _ in 0..count {
            value <<= 1;
            if self.next_bit >= 0 {
                let bit = self.next_bit as usize;
                value |= usize::from((self.input[bit / 8] >> (bit % 8)) & 1);
            }
            self.next_bit -= 1;
            self.remaining -= 1;
        }
        Ok(value)
    }
}

fn decode_huffman_stream(
    table: &[Option<HuffmanEntry>],
    maximum_bits: u8,
    stream: &[u8],
    regenerated_size: usize,
) -> Result<Vec<u8>, Error> {
    let mut bits = ReverseBits::new(stream)?;
    let mut output = Vec::with_capacity(regenerated_size);
    for _ in 0..regenerated_size {
        let entry = table[bits.peek_padded(maximum_bits)?];
        let entry = entry.ok_or(Error::InvalidHuffmanStream)?;
        if usize::from(entry.length) > bits.remaining {
            return Err(Error::InvalidHuffmanStream);
        }
        for _ in 0..entry.length {
            bits.bit()?;
        }
        output.push(entry.symbol);
    }
    if bits.remaining != 0 {
        return Err(Error::InvalidHuffmanStream);
    }
    Ok(output)
}

#[derive(Clone, Copy)]
struct HuffmanEntry {
    symbol: u8,
    length: u8,
}

#[derive(Clone)]
struct ReverseBits<'a> {
    input: &'a [u8],
    byte_index: isize,
    bit_index: i8,
    remaining: usize,
}

impl<'a> ReverseBits<'a> {
    fn new(input: &'a [u8]) -> Result<Self, Error> {
        let last = *input.last().ok_or(Error::InvalidHuffmanStream)?;
        if last == 0 {
            return Err(Error::InvalidHuffmanStream);
        }
        let marker = 7 - last.leading_zeros() as usize;
        Ok(Self {
            input,
            byte_index: input.len() as isize - 1,
            bit_index: marker as i8 - 1,
            remaining: (input.len() - 1)
                .checked_mul(8)
                .and_then(|bits| bits.checked_add(marker))
                .ok_or(Error::InvalidHuffmanStream)?,
        })
    }

    fn bit(&mut self) -> Result<u8, Error> {
        if self.remaining == 0 {
            return Err(Error::InvalidHuffmanStream);
        }
        if self.bit_index < 0 {
            self.byte_index -= 1;
            self.bit_index = 7;
        }
        let byte = *self
            .input
            .get(self.byte_index as usize)
            .ok_or(Error::InvalidHuffmanStream)?;
        let bit = (byte >> self.bit_index) & 1;
        self.bit_index -= 1;
        self.remaining -= 1;
        Ok(bit)
    }

    fn peek_padded(&self, count: u8) -> Result<usize, Error> {
        let mut copy = self.clone();
        let mut value = 0usize;
        for _ in 0..count {
            value <<= 1;
            if copy.remaining != 0 {
                value |= usize::from(copy.bit()?);
            }
        }
        Ok(value)
    }
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

        let truncated_huffman_header = compressed_frame(&[2, 0]);
        assert_eq!(decode_zstd(&truncated_huffman_header, 2), Err(Error::Truncated));

        let treeless = compressed_frame(&[3, 0]);
        assert_eq!(
            decode_zstd(&treeless, 2),
            Err(Error::UnsupportedLiteralsCompression)
        );
    }

    #[test]
    fn decodes_rfc_8878_direct_huffman_single_stream_vector() {
        // RFC 8878 Errata 8195 corrects the published "0145" bitstream from
        // 0x10,0x0d (which encodes "0154") to 0x01,0x0d.
        let literals = [0x42, 0x80, 0x01, 0x84, 0x43, 0x20, 0x10, 0x01, 0x0d, 0];
        let frame = compressed_frame(&literals);
        assert_eq!(decode_zstd(&frame, 4).unwrap(), [0, 1, 4, 5]);
        assert_eq!(
            decode_direct_huffman_literals(&[0x84, 0x43, 0x20, 0x10, 0x03], 1, false)
                .unwrap(),
            [0]
        );
    }

    #[test]
    fn decodes_direct_huffman_four_stream_size_formats() {
        let compressed_content = [
            0x80, 0x10, // two one-bit symbols
            0x01, 0x00, 0x01, 0x00, 0x01, 0x00, // jump table
            0x05, 0x05, 0x05, 0x05, // four streams, each decoding [0, 1]
        ];
        for header in [
            &[0x86, 0x00, 0x03][..],
            &[0x8a, 0x00, 0x30, 0x00][..],
            &[0x8e, 0x00, 0x00, 0x03, 0x00][..],
        ] {
            let mut literals = header.to_vec();
            literals.extend_from_slice(&compressed_content);
            literals.push(0);
            assert_eq!(
                decode_zstd(&compressed_frame(&literals), 8).unwrap(),
                [0, 1, 0, 1, 0, 1, 0, 1]
            );
        }

        let mut uneven = vec![0x76, 0x00, 0x03];
        uneven.extend_from_slice(&[
            0x80, 0x10, // two one-bit symbols
            0x01, 0x00, 0x01, 0x00, 0x01, 0x00, // jump table
            0x05, 0x05, 0x05, 0x03, // [0,1], [0,1], [0,1], [1]
            0,
        ]);
        assert_eq!(
            decode_zstd(&compressed_frame(&uneven), 7).unwrap(),
            [0, 1, 0, 1, 0, 1, 1]
        );
    }

    #[test]
    fn decodes_fse_compressed_huffman_weights() {
        // Accuracy_Log=5 with normalized frequencies [16,16] decodes the
        // interleaved stream to weights [1,1]; the inferred final weight is 2.
        let content = [0x04, 0x10, 0x3f, 0x63, 0x04, 0x03];
        assert_eq!(
            decode_direct_huffman_literals(&content, 1, false).unwrap(),
            [2]
        );

        let literals = [
            0x12, 0x80, 0x01, // regenerated=1, compressed=6
            0x04, 0x10, 0x3f, 0x63, 0x04, 0x03, // tree and Huffman stream
            0,
        ];
        assert_eq!(decode_zstd(&compressed_frame(&literals), 1).unwrap(), [2]);
    }

    #[test]
    fn rejects_invalid_or_unsupported_huffman_literals() {
        let truncated_fse_weights = compressed_frame(&[0x42, 0x40, 0x00, 1, 0, 0]);
        assert_eq!(
            decode_zstd(&truncated_fse_weights, 4),
            Err(Error::Truncated)
        );

        let excessive_accuracy = [0x02, 0x03, 0x01];
        assert_eq!(
            decode_direct_huffman_literals(&excessive_accuracy, 0, false),
            Err(Error::InvalidFseTable)
        );

        let zero_marker = [0x03, 0x10, 0x3f, 0x00];
        assert_eq!(
            decode_direct_huffman_literals(&zero_marker, 0, false),
            Err(Error::InvalidFseStream)
        );

        let treeless = compressed_frame(&[3]);
        assert_eq!(
            decode_zstd(&treeless, 1),
            Err(Error::UnsupportedLiteralsCompression)
        );

        let undersized_four_stream = compressed_frame(&[0x46, 0, 0]);
        assert_eq!(
            decode_zstd(&undersized_four_stream, 4),
            Err(Error::InvalidLiteralsSection)
        );

        let undersized_four_stream_content = compressed_frame(&[0x86, 0, 0]);
        assert_eq!(
            decode_zstd(&undersized_four_stream_content, 8),
            Err(Error::InvalidLiteralsSection)
        );

        let jump_sum_exceeds_streams = compressed_frame(&[
            0x86, 0x00, 0x03, 0x80, 0x10, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00,
            0x05, 0x05, 0x05, 0x05,
        ]);
        assert_eq!(
            decode_zstd(&jump_sum_exceeds_streams, 8),
            Err(Error::InvalidHuffmanStream)
        );

        let empty_required_stream = compressed_frame(&[
            0x86, 0xc0, 0x02, 0x80, 0x10, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
            0x05, 0x05, 0x05,
        ]);
        assert_eq!(
            decode_zstd(&empty_required_stream, 8),
            Err(Error::InvalidHuffmanStream)
        );

        let truncated_jump_table = compressed_frame(&[
            0x86, 0xc0, 0x01, 0x80, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        assert_eq!(
            decode_zstd(&truncated_jump_table, 8),
            Err(Error::Truncated)
        );

        let bad_padding = compressed_frame(&[
            0x42, 0x80, 0x01, 0x84, 0x43, 0x20, 0x10, 0x10, 0x00, 0,
        ]);
        assert_eq!(
            decode_zstd(&bad_padding, 4),
            Err(Error::InvalidHuffmanStream)
        );
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
