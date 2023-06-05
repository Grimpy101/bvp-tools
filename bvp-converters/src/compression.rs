pub fn read_u32(src: &Vec<u8>, i: usize) -> u32 {
    let b1 = src[i] as u32;
    let b2 = src[i+1] as u32;
    let b3 = src[i+2] as u32;
    let b4 = src[i+3] as u32;

    return b1 << 0 | b2 << 8 | b3 << 16 | b4 << 24;
}

// Bob Jenkins 4-byte integer hashing
pub fn hash_u32(x: u32) -> u32 {
    let mut x = (x + 0x7ed55d16) + (x <<  12);
    x = (x ^ 0xc761c23c) ^ (x >> 19);
    x = (x + 0x165667b1) + (x << 5);
    x = (x + 0xd3a2646c) ^ (x << 9);
    x = (x + 0xfd7046c5) + (x << 3);
    x = (x ^ 0xb55a4f09) ^ (x >> 16);
    return x;
}

pub fn create_hash_table() -> Vec<u32> {
    let l = 1 << 16;
    let mut hash_table = Vec::with_capacity(l);
    for _ in 0..l {
        hash_table.push(0u32);
    }
    return hash_table;
}

pub fn compress_lz4(src: &Vec<u8>) -> Result<Vec<u8>, String> {
    let mut hash_table = create_hash_table(); // Table for looking up already written data
    let src_len_f64 = src.len() as f64;
    let dest_len = (src_len_f64 + (src_len_f64 / 255.0) + 16.0).floor() as usize;
    let mut dest: Vec<u8> = Vec::with_capacity(dest_len);

    let mut src_index = 0;
    let search_step = 1;
    let mut literal_start = src_index;

    while src_index + 4 < src.len() {
        // Find a match for read source data
        let data = read_u32(src, src_index);
        let hash32 = hash_u32(data);
        let hash16 = (((hash32 >> 16) ^ hash32) & 0xffff) as usize;
        // Increment/decrement by 1 so we can later check for negative (invalid) indices
        let match_index = hash_table[hash16] as i32 - 1;
        hash_table[hash16] = (src_index + 1) as u32;

        let match_found = match_index >= 0; // The valid index check
        let mut match_correct = false;
        let match_offset = src_index - match_index as usize;
        let match_near_enough = match_offset < (1 << 16);
        if match_found {
            match_correct = read_u32(src, match_index as usize) == data;
        }

        if !match_found || !match_correct || !match_near_enough {
            src_index += search_step;
            continue;
        }

        // Determine the length of duplicated data
        let mut match_index = match_index as usize;
        let literal_count = src_index - literal_start;
        let match_start = match_index;
        while src_index < src.len() && src[src_index] == src[match_index] {
            src_index += 1;
            match_index += 1;
        }
        let match_length = match_index - match_start;
        
        // Write token
        let token_literal_count = literal_count.min(0xf);
        let token_match_length = (match_length).min(0xf);
        let token = (token_literal_count << 4) | token_match_length;
        dest.push(token as u8);

        // Write possible additional data length bytes
        if literal_count >= 0xf {
            let mut remaining = literal_count - 0xf;
            while remaining >= 0xff {
                dest.push(0xff);
                remaining -= 0xff;
            }
            dest.push(remaining as u8);
        }

        // Write source data
        for _ in 0..literal_count {
            dest.push(src[literal_start]);
            literal_start += 1;
        }

        // Write match offset
        dest.push((match_offset as u8 >> 0) & 0xff);
        dest.push((match_offset as u8 >> 8) & 0xff);

        // Write possible additional match length bytes
        if match_length >= 0xf {
            let mut remaining = match_length - 0xf;
            while remaining >= 0xff {
                dest.push(0xff);
                remaining -= 0xff;
            }
            dest.push(remaining as u8);
        }

        literal_start = src_index;
    }

    // Write remaining data as before

    let literal_count = src.len() - literal_start;
    let match_length = 0;
    let token_literal_count = literal_count.min(0xf);
    let token_match_length = match_length.min(0xf);
    let token = (token_literal_count << 4) | token_match_length;
    dest.push(token as u8);

    if literal_count >= 0xf {
        let mut remaining = literal_count - 0xf;
        while remaining >= 0xff {
            dest.push(0xff);
            remaining -= 0xff;
        }
        dest.push(remaining as u8);
    }

    for _ in 0..literal_count {
        dest.push(src[literal_start]);
        literal_start += 1;
    }

    // Write end token
    if literal_count > 0 {
        dest.push(0);
    }

    return Ok(dest);
}