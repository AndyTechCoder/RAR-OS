//! Bounded, dependency-free RFC 1951/RFC 1952 decoder.

use super::MAX_LAYER_BYTES;

const MAX_GZIP_INPUT_BYTES: usize = MAX_LAYER_BYTES;
const MAX_GZIP_HEADER_BYTES: usize = 1_048_576;
pub(crate) const MAX_DEFLATE_BLOCKS: usize = 8_192;
const MAX_BITS: usize = 15;
const NO_CHILD: usize = usize::MAX;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InputTooLarge,
    OutputLimitTooLarge,
    Truncated,
    InvalidHeader,
    UnsupportedMethod,
    InvalidFlags,
    HeaderTooLarge,
    HeaderChecksumMismatch,
    InvalidBlockType,
    TooManyBlocks,
    InvalidStoredLength,
    InvalidHuffmanTree,
    InvalidCode,
    InvalidLength,
    InvalidDistance,
    OutputTooLarge,
    TrailingDeflateData,
    DataChecksumMismatch,
    DataSizeMismatch,
}

/// Decodes exactly one gzip member into at most `maximum_output_bytes`.
pub fn decode_gzip(input: &[u8], maximum_output_bytes: usize) -> Result<Vec<u8>, Error> {
    if input.len() > MAX_GZIP_INPUT_BYTES {
        return Err(Error::InputTooLarge);
    }
    if maximum_output_bytes > MAX_LAYER_BYTES {
        return Err(Error::OutputLimitTooLarge);
    }
    let payload_offset = parse_header(input)?;
    let trailer_offset = input.len().checked_sub(8).ok_or(Error::Truncated)?;
    if payload_offset > trailer_offset {
        return Err(Error::Truncated);
    }
    let output = decode_deflate(
        &input[payload_offset..trailer_offset],
        maximum_output_bytes,
    )?;
    let expected_crc = read_le_u32(&input[trailer_offset..trailer_offset + 4])?;
    let expected_size = read_le_u32(&input[trailer_offset + 4..])?;
    if crc32(&output) != expected_crc {
        return Err(Error::DataChecksumMismatch);
    }
    if output.len() as u64 != u64::from(expected_size) {
        return Err(Error::DataSizeMismatch);
    }
    Ok(output)
}

fn parse_header(input: &[u8]) -> Result<usize, Error> {
    if input.len() < 18 {
        return Err(Error::Truncated);
    }
    if input[0] != 0x1f || input[1] != 0x8b {
        return Err(Error::InvalidHeader);
    }
    if input[2] != 8 {
        return Err(Error::UnsupportedMethod);
    }
    let flags = input[3];
    if flags & 0xe0 != 0 {
        return Err(Error::InvalidFlags);
    }
    let trailer_offset = input.len() - 8;
    let mut offset = 10usize;
    if flags & 0x04 != 0 {
        let length_end = offset.checked_add(2).ok_or(Error::HeaderTooLarge)?;
        if length_end > trailer_offset {
            return Err(Error::Truncated);
        }
        let extra_length = usize::from(input[offset]) | (usize::from(input[offset + 1]) << 8);
        offset = length_end
            .checked_add(extra_length)
            .ok_or(Error::HeaderTooLarge)?;
        require_header_offset(offset, trailer_offset)?;
    }
    if flags & 0x08 != 0 {
        offset = skip_zero_terminated(input, offset, trailer_offset)?;
    }
    if flags & 0x10 != 0 {
        offset = skip_zero_terminated(input, offset, trailer_offset)?;
    }
    if flags & 0x02 != 0 {
        let checksum_offset = offset;
        offset = offset.checked_add(2).ok_or(Error::HeaderTooLarge)?;
        require_header_offset(offset, trailer_offset)?;
        let expected = u16::from_le_bytes([input[checksum_offset], input[checksum_offset + 1]]);
        if crc32(&input[..checksum_offset]) as u16 != expected {
            return Err(Error::HeaderChecksumMismatch);
        }
    }
    Ok(offset)
}

fn skip_zero_terminated(
    input: &[u8],
    mut offset: usize,
    trailer_offset: usize,
) -> Result<usize, Error> {
    loop {
        require_header_offset(offset, trailer_offset)?;
        if offset == trailer_offset {
            return Err(Error::Truncated);
        }
        let byte = input[offset];
        offset += 1;
        if byte == 0 {
            return Ok(offset);
        }
    }
}

fn require_header_offset(offset: usize, trailer_offset: usize) -> Result<(), Error> {
    if offset > MAX_GZIP_HEADER_BYTES {
        return Err(Error::HeaderTooLarge);
    }
    if offset > trailer_offset {
        return Err(Error::Truncated);
    }
    Ok(())
}

fn decode_deflate(input: &[u8], maximum_output_bytes: usize) -> Result<Vec<u8>, Error> {
    let mut bits = BitReader::new(input);
    let mut output = Vec::new();
    let fixed = fixed_trees()?;
    let mut block_count = 0usize;
    loop {
        block_count = block_count
            .checked_add(1)
            .ok_or(Error::TooManyBlocks)?;
        if block_count > MAX_DEFLATE_BLOCKS {
            return Err(Error::TooManyBlocks);
        }
        let final_block = bits.read_bits(1)? != 0;
        match bits.read_bits(2)? {
            0 => decode_stored_block(&mut bits, &mut output, maximum_output_bytes)?,
            1 => {
                decode_huffman_block(
                    &mut bits,
                    &mut output,
                    maximum_output_bytes,
                    &fixed.0,
                    Some(&fixed.1),
                )?;
            }
            2 => {
                let (literal, distance) = dynamic_trees(&mut bits)?;
                decode_huffman_block(
                    &mut bits,
                    &mut output,
                    maximum_output_bytes,
                    &literal,
                    distance.as_ref(),
                )?;
            }
            _ => return Err(Error::InvalidBlockType),
        }
        if final_block {
            break;
        }
    }
    bits.finish()?;
    Ok(output)
}

fn decode_stored_block(
    bits: &mut BitReader<'_>,
    output: &mut Vec<u8>,
    maximum_output_bytes: usize,
) -> Result<(), Error> {
    bits.align_to_byte();
    let length = bits.read_u16()?;
    let inverse = bits.read_u16()?;
    if length ^ inverse != u16::MAX {
        return Err(Error::InvalidStoredLength);
    }
    let bytes = bits.read_bytes(usize::from(length))?;
    extend_bounded(output, bytes, maximum_output_bytes)
}

fn decode_huffman_block(
    bits: &mut BitReader<'_>,
    output: &mut Vec<u8>,
    maximum_output_bytes: usize,
    literal: &Huffman,
    distance: Option<&Huffman>,
) -> Result<(), Error> {
    loop {
        let symbol = literal.decode(bits)?;
        match symbol {
            0..=255 => push_bounded(output, symbol as u8, maximum_output_bytes)?,
            256 => return Ok(()),
            257..=285 => {
                let index = symbol - 257;
                let length = LENGTH_BASE[index]
                    + bits.read_bits(LENGTH_EXTRA[index])? as usize;
                let distance_symbol = distance.ok_or(Error::InvalidDistance)?.decode(bits)?;
                if distance_symbol >= DISTANCE_BASE.len() {
                    return Err(Error::InvalidDistance);
                }
                let distance_value = DISTANCE_BASE[distance_symbol]
                    + bits.read_bits(DISTANCE_EXTRA[distance_symbol])? as usize;
                copy_match(output, distance_value, length, maximum_output_bytes)?;
            }
            _ => return Err(Error::InvalidLength),
        }
    }
}

fn fixed_trees() -> Result<(Huffman, Huffman), Error> {
    let mut literal_lengths = vec![0u8; 288];
    literal_lengths[..=143].fill(8);
    literal_lengths[144..=255].fill(9);
    literal_lengths[256..=279].fill(7);
    literal_lengths[280..].fill(8);
    let distance_lengths = vec![5u8; 32];
    Ok((
        Huffman::new(&literal_lengths)?,
        Huffman::new(&distance_lengths)?,
    ))
}

fn dynamic_trees(bits: &mut BitReader<'_>) -> Result<(Huffman, Option<Huffman>), Error> {
    let literal_count = bits.read_bits(5)? as usize + 257;
    let distance_count = bits.read_bits(5)? as usize + 1;
    let code_count = bits.read_bits(4)? as usize + 4;
    if literal_count > 286 || distance_count > 32 {
        return Err(Error::InvalidHuffmanTree);
    }
    const ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut code_lengths = [0u8; 19];
    for symbol in ORDER.iter().take(code_count) {
        code_lengths[*symbol] = bits.read_bits(3)? as u8;
    }
    let code_tree = Huffman::new(&code_lengths)?;
    let total = literal_count + distance_count;
    let mut lengths = Vec::with_capacity(total);
    while lengths.len() < total {
        match code_tree.decode(bits)? {
            value @ 0..=15 => lengths.push(value as u8),
            16 => {
                let previous = *lengths.last().ok_or(Error::InvalidHuffmanTree)?;
                let repeat = bits.read_bits(2)? as usize + 3;
                append_repeated(&mut lengths, total, previous, repeat)?;
            }
            17 => {
                let repeat = bits.read_bits(3)? as usize + 3;
                append_repeated(&mut lengths, total, 0, repeat)?;
            }
            18 => {
                let repeat = bits.read_bits(7)? as usize + 11;
                append_repeated(&mut lengths, total, 0, repeat)?;
            }
            _ => return Err(Error::InvalidHuffmanTree),
        }
    }
    let literal = Huffman::new(&lengths[..literal_count])?;
    if lengths[256] == 0 {
        return Err(Error::InvalidHuffmanTree);
    }
    let distance_lengths = &lengths[literal_count..];
    let distance = if distance_lengths.iter().all(|length| *length == 0) {
        if distance_count != 1 {
            return Err(Error::InvalidHuffmanTree);
        }
        None
    } else {
        Some(Huffman::new(distance_lengths)?)
    };
    Ok((literal, distance))
}

fn append_repeated(
    output: &mut Vec<u8>,
    maximum: usize,
    value: u8,
    count: usize,
) -> Result<(), Error> {
    if output.len().checked_add(count).is_none_or(|end| end > maximum) {
        return Err(Error::InvalidHuffmanTree);
    }
    output.resize(output.len() + count, value);
    Ok(())
}

#[derive(Clone, Copy)]
struct Node {
    children: [usize; 2],
    symbol: Option<usize>,
}

impl Node {
    fn empty() -> Self {
        Self {
            children: [NO_CHILD; 2],
            symbol: None,
        }
    }
}

struct Huffman {
    nodes: Vec<Node>,
    maximum_depth: usize,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Result<Self, Error> {
        let mut counts = [0usize; MAX_BITS + 1];
        let mut populated = 0usize;
        for length in lengths {
            if usize::from(*length) > MAX_BITS {
                return Err(Error::InvalidHuffmanTree);
            }
            if *length != 0 {
                counts[usize::from(*length)] += 1;
                populated += 1;
            }
        }
        if populated == 0 {
            return Err(Error::InvalidHuffmanTree);
        }
        let mut next_code = [0usize; MAX_BITS + 1];
        let mut code = 0usize;
        for bit_length in 1..=MAX_BITS {
            code = (code + counts[bit_length - 1]) << 1;
            if code + counts[bit_length] > (1usize << bit_length) {
                return Err(Error::InvalidHuffmanTree);
            }
            next_code[bit_length] = code;
        }

        let mut tree = Self {
            nodes: vec![Node::empty()],
            maximum_depth: 0,
        };
        for (symbol, length) in lengths.iter().copied().enumerate() {
            let length = usize::from(length);
            if length == 0 {
                continue;
            }
            let symbol_code = next_code[length];
            next_code[length] += 1;
            tree.insert(symbol_code, length, symbol)?;
            tree.maximum_depth = tree.maximum_depth.max(length);
        }
        Ok(tree)
    }

    fn insert(&mut self, code: usize, length: usize, symbol: usize) -> Result<(), Error> {
        let mut node = 0usize;
        for depth in 0..length {
            if self.nodes[node].symbol.is_some() {
                return Err(Error::InvalidHuffmanTree);
            }
            let bit = (code >> (length - depth - 1)) & 1;
            let next = self.nodes[node].children[bit];
            node = if next == NO_CHILD {
                let created = self.nodes.len();
                self.nodes.push(Node::empty());
                self.nodes[node].children[bit] = created;
                created
            } else {
                next
            };
        }
        if self.nodes[node].symbol.is_some()
            || self.nodes[node].children.iter().any(|child| *child != NO_CHILD)
        {
            return Err(Error::InvalidHuffmanTree);
        }
        self.nodes[node].symbol = Some(symbol);
        Ok(())
    }

    fn decode(&self, bits: &mut BitReader<'_>) -> Result<usize, Error> {
        let mut node = 0usize;
        for _ in 0..self.maximum_depth {
            let bit = bits.read_bits(1)? as usize;
            node = self.nodes[node].children[bit];
            if node == NO_CHILD {
                return Err(Error::InvalidCode);
            }
            if let Some(symbol) = self.nodes[node].symbol {
                return Ok(symbol);
            }
        }
        Err(Error::InvalidCode)
    }
}

struct BitReader<'a> {
    input: &'a [u8],
    byte_offset: usize,
    bit_offset: u8,
}

impl<'a> BitReader<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            byte_offset: 0,
            bit_offset: 0,
        }
    }

    fn read_bits(&mut self, count: u8) -> Result<u32, Error> {
        let mut value = 0u32;
        for shift in 0..count {
            let byte = *self.input.get(self.byte_offset).ok_or(Error::Truncated)?;
            value |= u32::from((byte >> self.bit_offset) & 1) << shift;
            self.bit_offset += 1;
            if self.bit_offset == 8 {
                self.bit_offset = 0;
                self.byte_offset += 1;
            }
        }
        Ok(value)
    }

    fn align_to_byte(&mut self) {
        if self.bit_offset != 0 {
            self.bit_offset = 0;
            self.byte_offset += 1;
        }
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        if self.bit_offset != 0 {
            return Err(Error::InvalidStoredLength);
        }
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], Error> {
        if self.bit_offset != 0 {
            return Err(Error::InvalidStoredLength);
        }
        let end = self.byte_offset.checked_add(count).ok_or(Error::Truncated)?;
        let bytes = self
            .input
            .get(self.byte_offset..end)
            .ok_or(Error::Truncated)?;
        self.byte_offset = end;
        Ok(bytes)
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.align_to_byte();
        if self.byte_offset != self.input.len() {
            return Err(Error::TrailingDeflateData);
        }
        Ok(())
    }
}

fn push_bounded(output: &mut Vec<u8>, byte: u8, maximum: usize) -> Result<(), Error> {
    if output.len() >= maximum {
        return Err(Error::OutputTooLarge);
    }
    output.push(byte);
    Ok(())
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

fn copy_match(
    output: &mut Vec<u8>,
    distance: usize,
    length: usize,
    maximum: usize,
) -> Result<(), Error> {
    if distance == 0 || distance > output.len() {
        return Err(Error::InvalidDistance);
    }
    if output
        .len()
        .checked_add(length)
        .is_none_or(|total| total > maximum)
    {
        return Err(Error::OutputTooLarge);
    }
    for _ in 0..length {
        let byte = output[output.len() - distance];
        output.push(byte);
    }
    Ok(())
}

fn read_le_u32(bytes: &[u8]) -> Result<u32, Error> {
    let bytes: [u8; 4] = bytes.try_into().map_err(|_| Error::Truncated)?;
    Ok(u32::from_le_bytes(bytes))
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb88320 & mask);
        }
    }
    !crc
}

const LENGTH_BASE: [usize; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99,
    115, 131, 163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5,
    5, 0,
];
const DISTANCE_BASE: [usize; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025,
    1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
    12, 13, 13,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_stored_fixed_and_dynamic_blocks() {
        let stored = gzip_member(&stored_deflate(b"stored"), b"stored");
        assert_eq!(decode_gzip(&stored, 6).unwrap(), b"stored");

        let fixed = gzip_member(&fixed_deflate(b"fixed"), b"fixed");
        assert_eq!(decode_gzip(&fixed, 5).unwrap(), b"fixed");

        let dynamic = gzip_member(&dynamic_a_deflate(), b"A");
        assert_eq!(decode_gzip(&dynamic, 1).unwrap(), b"A");
    }

    #[test]
    fn decodes_matches_extra_bits_and_many_empty_fixed_blocks() {
        let matches = gzip_member(&fixed_match_deflate(), b"abcdeabcdeabcdea");
        assert_eq!(
            decode_gzip(&matches, 16).unwrap(),
            b"abcdeabcdeabcdea"
        );

        let empty = gzip_member(&many_empty_fixed_blocks(512), b"");
        assert_eq!(decode_gzip(&empty, 0).unwrap(), b"");
    }

    #[test]
    fn enforces_deflate_block_work_limit() {
        for deflate in [
            many_empty_fixed_blocks(MAX_DEFLATE_BLOCKS),
            many_empty_stored_blocks(MAX_DEFLATE_BLOCKS),
        ] {
            let member = gzip_member(&deflate, b"");
            assert_eq!(decode_gzip(&member, 0).unwrap(), b"");
        }

        for deflate in [
            many_empty_fixed_blocks(MAX_DEFLATE_BLOCKS + 1),
            many_empty_stored_blocks(MAX_DEFLATE_BLOCKS + 1),
        ] {
            let member = gzip_member(&deflate, b"");
            assert_eq!(decode_gzip(&member, 0), Err(Error::TooManyBlocks));
        }
    }

    #[test]
    fn accepts_optional_headers_and_rejects_every_truncation() {
        let deflate = stored_deflate(b"header");
        let mut member = optional_header_member(&deflate, b"header");
        assert_eq!(decode_gzip(&member, 6).unwrap(), b"header");

        let header_crc = 10 + 2 + 3 + 5 + 8;
        member[header_crc] ^= 1;
        assert_eq!(decode_gzip(&member, 6), Err(Error::HeaderChecksumMismatch));

        let valid = gzip_member(&fixed_match_deflate(), b"abcdeabcdeabcdea");
        for end in 0..valid.len() {
            assert!(decode_gzip(&valid[..end], 16).is_err());
        }
    }

    #[test]
    fn enforces_output_crc_size_and_single_member_bounds() {
        let member = gzip_member(&stored_deflate(b"bounded"), b"bounded");
        assert_eq!(decode_gzip(&member, 6), Err(Error::OutputTooLarge));

        let mut bad_crc = member.clone();
        let trailer = bad_crc.len() - 8;
        bad_crc[trailer] ^= 1;
        assert_eq!(decode_gzip(&bad_crc, 7), Err(Error::DataChecksumMismatch));

        let mut bad_size = member.clone();
        let size = bad_size.len() - 4;
        bad_size[size] ^= 1;
        assert_eq!(decode_gzip(&bad_size, 7), Err(Error::DataSizeMismatch));

        let mut trailing = member.clone();
        trailing.insert(trailer, 0);
        assert_eq!(decode_gzip(&trailing, 7), Err(Error::TrailingDeflateData));
    }

    #[test]
    fn rejects_bad_headers_stored_lengths_and_huffman_trees() {
        let mut header = gzip_member(&stored_deflate(b"x"), b"x");
        header[3] = 0xe0;
        assert_eq!(decode_gzip(&header, 1), Err(Error::InvalidFlags));

        let mut stored = gzip_member(&stored_deflate(b"x"), b"x");
        stored[13] ^= 1;
        assert_eq!(decode_gzip(&stored, 1), Err(Error::InvalidStoredLength));

        let mut invalid = BitWriter::default();
        invalid.write_bits(1, 1);
        invalid.write_bits(2, 2);
        invalid.write_bits(0, 5);
        invalid.write_bits(0, 5);
        invalid.write_bits(0, 4);
        for _ in 0..4 {
            invalid.write_bits(0, 3);
        }
        let invalid = gzip_member(&invalid.finish(), b"");
        assert_eq!(decode_gzip(&invalid, 0), Err(Error::InvalidHuffmanTree));
    }

    fn gzip_member(deflate: &[u8], output: &[u8]) -> Vec<u8> {
        let mut gzip = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 255];
        gzip.extend_from_slice(deflate);
        gzip.extend_from_slice(&crc32(output).to_le_bytes());
        gzip.extend_from_slice(&(output.len() as u32).to_le_bytes());
        gzip
    }

    fn optional_header_member(deflate: &[u8], output: &[u8]) -> Vec<u8> {
        let mut gzip = vec![0x1f, 0x8b, 8, 0x1e, 0, 0, 0, 0, 0, 255];
        gzip.extend_from_slice(&3u16.to_le_bytes());
        gzip.extend_from_slice(b"ext");
        gzip.extend_from_slice(b"name\0");
        gzip.extend_from_slice(b"comment\0");
        let header_crc = crc32(&gzip) as u16;
        gzip.extend_from_slice(&header_crc.to_le_bytes());
        gzip.extend_from_slice(deflate);
        gzip.extend_from_slice(&crc32(output).to_le_bytes());
        gzip.extend_from_slice(&(output.len() as u32).to_le_bytes());
        gzip
    }

    fn stored_deflate(bytes: &[u8]) -> Vec<u8> {
        let length = u16::try_from(bytes.len()).unwrap();
        let mut output = vec![1];
        output.extend_from_slice(&length.to_le_bytes());
        output.extend_from_slice(&(!length).to_le_bytes());
        output.extend_from_slice(bytes);
        output
    }

    fn fixed_deflate(bytes: &[u8]) -> Vec<u8> {
        let mut writer = BitWriter::default();
        writer.write_bits(1, 1);
        writer.write_bits(1, 2);
        for byte in bytes {
            let (code, length) = fixed_literal_code(usize::from(*byte));
            writer.write_huffman(code, length);
        }
        let (end, length) = fixed_literal_code(256);
        writer.write_huffman(end, length);
        writer.finish()
    }

    fn fixed_match_deflate() -> Vec<u8> {
        let mut writer = BitWriter::default();
        writer.write_bits(1, 1);
        writer.write_bits(1, 2);
        for byte in b"abcde" {
            let (code, length) = fixed_literal_code(usize::from(*byte));
            writer.write_huffman(code, length);
        }
        let (length_code, length_bits) = fixed_literal_code(265);
        writer.write_huffman(length_code, length_bits);
        writer.write_bits(0, 1);
        writer.write_huffman(4, 5);
        writer.write_bits(0, 1);
        let (end, end_bits) = fixed_literal_code(256);
        writer.write_huffman(end, end_bits);
        writer.finish()
    }

    fn many_empty_fixed_blocks(count: usize) -> Vec<u8> {
        let mut writer = BitWriter::default();
        for block in 0..count {
            writer.write_bits(if block + 1 == count { 1 } else { 0 }, 1);
            writer.write_bits(1, 2);
            let (end, length) = fixed_literal_code(256);
            writer.write_huffman(end, length);
        }
        writer.finish()
    }

    fn many_empty_stored_blocks(count: usize) -> Vec<u8> {
        let mut output = Vec::with_capacity(count * 5);
        for block in 0..count {
            output.push(u8::from(block + 1 == count));
            output.extend_from_slice(&0u16.to_le_bytes());
            output.extend_from_slice(&u16::MAX.to_le_bytes());
        }
        output
    }

    fn fixed_literal_code(symbol: usize) -> (u16, u8) {
        match symbol {
            0..=143 => (0x30 + symbol as u16, 8),
            144..=255 => (0x190 + (symbol - 144) as u16, 9),
            256..=279 => ((symbol - 256) as u16, 7),
            280..=287 => (0xc0 + (symbol - 280) as u16, 8),
            _ => unreachable!(),
        }
    }

    fn dynamic_a_deflate() -> Vec<u8> {
        let mut writer = BitWriter::default();
        writer.write_bits(1, 1);
        writer.write_bits(2, 2);
        writer.write_bits(0, 5);
        writer.write_bits(0, 5);
        writer.write_bits(14, 4);
        for symbol in [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1] {
            writer.write_bits(
                match symbol {
                    18 => 1,
                    0 | 1 => 2,
                    _ => 0,
                },
                3,
            );
        }
        writer.write_huffman(0, 1);
        writer.write_bits(54, 7);
        writer.write_huffman(3, 2);
        writer.write_huffman(0, 1);
        writer.write_bits(127, 7);
        writer.write_huffman(0, 1);
        writer.write_bits(41, 7);
        writer.write_huffman(3, 2);
        writer.write_huffman(2, 2);
        writer.write_huffman(0, 1);
        writer.write_huffman(1, 1);
        writer.finish()
    }

    #[derive(Default)]
    struct BitWriter {
        bytes: Vec<u8>,
        bit_offset: u8,
    }

    impl BitWriter {
        fn write_bits(&mut self, value: u32, count: u8) {
            for bit in 0..count {
                self.write_bit(((value >> bit) & 1) as u8);
            }
        }

        fn write_huffman(&mut self, value: u16, count: u8) {
            for bit in (0..count).rev() {
                self.write_bit(((value >> bit) & 1) as u8);
            }
        }

        fn write_bit(&mut self, bit: u8) {
            if self.bit_offset == 0 {
                self.bytes.push(0);
            }
            let last = self.bytes.len() - 1;
            self.bytes[last] |= bit << self.bit_offset;
            self.bit_offset = (self.bit_offset + 1) % 8;
        }

        fn finish(self) -> Vec<u8> {
            self.bytes
        }
    }
}
