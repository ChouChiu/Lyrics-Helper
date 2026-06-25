// Simplified Triple-DES implementation for QRC decryption
// Note: This is a simplified version for demonstration purposes

const ENCRYPT: i32 = 1;
const DECRYPT: i32 = 0;

fn bitnum(a: &[u8], b: i32, c: i32) -> u8 {
    let pos = (b / 8) as usize;
    let bit = 7 - (b % 8);
    ((a[pos] >> bit) & 1) << c
}

fn ip(state: &mut [u8], input: &[u8]) {
    state[0] = bitnum(input, 57, 7)
        | bitnum(input, 49, 6)
        | bitnum(input, 41, 5)
        | bitnum(input, 33, 4)
        | bitnum(input, 25, 3)
        | bitnum(input, 17, 2)
        | bitnum(input, 9, 1)
        | bitnum(input, 1, 0);

    state[1] = bitnum(input, 59, 7)
        | bitnum(input, 51, 6)
        | bitnum(input, 43, 5)
        | bitnum(input, 35, 4)
        | bitnum(input, 27, 3)
        | bitnum(input, 19, 2)
        | bitnum(input, 11, 1)
        | bitnum(input, 3, 0);

    state[2] = bitnum(input, 61, 7)
        | bitnum(input, 53, 6)
        | bitnum(input, 45, 5)
        | bitnum(input, 37, 4)
        | bitnum(input, 29, 3)
        | bitnum(input, 21, 2)
        | bitnum(input, 13, 1)
        | bitnum(input, 5, 0);

    state[3] = bitnum(input, 63, 7)
        | bitnum(input, 55, 6)
        | bitnum(input, 47, 5)
        | bitnum(input, 39, 4)
        | bitnum(input, 31, 3)
        | bitnum(input, 23, 2)
        | bitnum(input, 15, 1)
        | bitnum(input, 7, 0);

    state[4] = bitnum(input, 56, 7)
        | bitnum(input, 48, 6)
        | bitnum(input, 40, 5)
        | bitnum(input, 32, 4)
        | bitnum(input, 24, 3)
        | bitnum(input, 16, 2)
        | bitnum(input, 8, 1)
        | bitnum(input, 0, 0);

    state[5] = bitnum(input, 58, 7)
        | bitnum(input, 50, 6)
        | bitnum(input, 42, 5)
        | bitnum(input, 34, 4)
        | bitnum(input, 26, 3)
        | bitnum(input, 18, 2)
        | bitnum(input, 10, 1)
        | bitnum(input, 2, 0);

    state[6] = bitnum(input, 60, 7)
        | bitnum(input, 52, 6)
        | bitnum(input, 44, 5)
        | bitnum(input, 36, 4)
        | bitnum(input, 28, 3)
        | bitnum(input, 20, 2)
        | bitnum(input, 12, 1)
        | bitnum(input, 4, 0);

    state[7] = bitnum(input, 62, 7)
        | bitnum(input, 54, 6)
        | bitnum(input, 46, 5)
        | bitnum(input, 38, 4)
        | bitnum(input, 30, 3)
        | bitnum(input, 22, 2)
        | bitnum(input, 14, 1)
        | bitnum(input, 6, 0);
}

fn inv_ip(state: &mut [u8], input: &[u8]) {
    state[0] = bitnum(input, 39, 7)
        | bitnum(input, 7, 6)
        | bitnum(input, 47, 5)
        | bitnum(input, 15, 4)
        | bitnum(input, 55, 3)
        | bitnum(input, 23, 2)
        | bitnum(input, 63, 1)
        | bitnum(input, 31, 0);

    state[1] = bitnum(input, 38, 7)
        | bitnum(input, 6, 6)
        | bitnum(input, 46, 5)
        | bitnum(input, 14, 4)
        | bitnum(input, 54, 3)
        | bitnum(input, 22, 2)
        | bitnum(input, 62, 1)
        | bitnum(input, 30, 0);

    state[2] = bitnum(input, 37, 7)
        | bitnum(input, 5, 6)
        | bitnum(input, 45, 5)
        | bitnum(input, 13, 4)
        | bitnum(input, 53, 3)
        | bitnum(input, 21, 2)
        | bitnum(input, 61, 1)
        | bitnum(input, 29, 0);

    state[3] = bitnum(input, 36, 7)
        | bitnum(input, 4, 6)
        | bitnum(input, 44, 5)
        | bitnum(input, 12, 4)
        | bitnum(input, 52, 3)
        | bitnum(input, 20, 2)
        | bitnum(input, 60, 1)
        | bitnum(input, 28, 0);

    state[4] = bitnum(input, 35, 7)
        | bitnum(input, 3, 6)
        | bitnum(input, 43, 5)
        | bitnum(input, 11, 4)
        | bitnum(input, 51, 3)
        | bitnum(input, 19, 2)
        | bitnum(input, 59, 1)
        | bitnum(input, 27, 0);

    state[5] = bitnum(input, 34, 7)
        | bitnum(input, 2, 6)
        | bitnum(input, 42, 5)
        | bitnum(input, 10, 4)
        | bitnum(input, 50, 3)
        | bitnum(input, 18, 2)
        | bitnum(input, 58, 1)
        | bitnum(input, 26, 0);

    state[6] = bitnum(input, 33, 7)
        | bitnum(input, 1, 6)
        | bitnum(input, 41, 5)
        | bitnum(input, 9, 4)
        | bitnum(input, 49, 3)
        | bitnum(input, 17, 2)
        | bitnum(input, 57, 1)
        | bitnum(input, 25, 0);

    state[7] = bitnum(input, 32, 7)
        | bitnum(input, 0, 6)
        | bitnum(input, 40, 5)
        | bitnum(input, 8, 4)
        | bitnum(input, 48, 3)
        | bitnum(input, 16, 2)
        | bitnum(input, 56, 1)
        | bitnum(input, 24, 0);
}

pub fn des_key_setup(key: &[u8], schedule: &mut [[u8; 8]; 16], mode: i32) {
    let mut key_bits = [0u8; 64];
    for i in 0..8 {
        for j in 0..8 {
            key_bits[i * 8 + j] = (key[i] >> (7 - j)) & 1;
        }
    }

    let pc1: [usize; 56] = [
        56, 48, 40, 32, 24, 16, 8, 0, 57, 49, 41, 33, 25, 17,
        9, 1, 58, 50, 42, 34, 26, 18, 10, 2, 59, 51, 43, 35,
        62, 54, 46, 38, 30, 22, 14, 6, 61, 53, 45, 37, 29, 21,
        13, 5, 60, 52, 44, 36, 28, 20, 12, 4, 27, 19, 11, 3,
    ];

    let mut c = [0u8; 28];
    let mut d = [0u8; 28];

    for i in 0..28 {
        c[i] = key_bits[pc1[i]];
        d[i] = key_bits[pc1[i + 28]];
    }

    let shifts = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];

    for i in 0..16 {
        let shift = shifts[i];
        for _ in 0..shift {
            let c0 = c[0];
            let d0 = d[0];
            for j in 0..27 {
                c[j] = c[j + 1];
                d[j] = d[j + 1];
            }
            c[27] = c0;
            d[27] = d0;
        }

        let pc2: [usize; 48] = [
            13, 16, 10, 23, 0, 4, 2, 27, 14, 5, 20, 9,
            22, 18, 11, 3, 25, 7, 15, 6, 26, 19, 12, 1,
            40, 51, 30, 36, 46, 54, 29, 39, 50, 44, 32, 47,
            43, 48, 38, 55, 33, 52, 45, 41, 49, 35, 28, 31,
        ];

        let mut cd = [0u8; 56];
        cd[..28].copy_from_slice(&c);
        cd[28..56].copy_from_slice(&d);

        for j in 0..48 {
            let idx = pc2[j];
            schedule[i][j / 8] |= cd[idx] << (7 - (j % 8));
        }
    }

    if mode == DECRYPT {
        let (first, rest) = schedule.split_at_mut(8);
        for (a, b) in first.iter_mut().zip(rest.iter_mut().rev()) {
            a.swap_with_slice(b);
        }
    }
}

pub fn des_crypt(block: &[u8], schedule: &[[u8; 8]; 16]) -> [u8; 8] {
    let mut state = [0u8; 8];
    let mut result = [0u8; 8];

    ip(&mut state, block);

    for round_keys in schedule.iter().take(16) {
        let mut new_state = [0u8; 8];
        for i in 0..4 {
            new_state[i] = state[i + 4];
            new_state[i + 4] = state[i] ^ round_keys[i];
        }
        state = new_state;
    }

    inv_ip(&mut result, &state);

    result
}

pub fn triple_des_key_setup(key: &[u8], schedule: &mut [[[u8; 8]; 16]; 3], mode: i32) {
    let mut key1 = [0u8; 8];
    let mut key2 = [0u8; 8];
    let mut key3 = [0u8; 8];

    key1.copy_from_slice(&key[0..8]);
    key2.copy_from_slice(&key[8..16]);
    key3.copy_from_slice(&key[16..24]);

    if mode == ENCRYPT {
        des_key_setup(&key1, &mut schedule[0], ENCRYPT);
        des_key_setup(&key2, &mut schedule[1], DECRYPT);
        des_key_setup(&key3, &mut schedule[2], ENCRYPT);
    } else {
        des_key_setup(&key3, &mut schedule[0], DECRYPT);
        des_key_setup(&key2, &mut schedule[1], ENCRYPT);
        des_key_setup(&key1, &mut schedule[2], DECRYPT);
    }
}

pub fn triple_des_crypt(block: &[u8], schedule: &[[[u8; 8]; 16]; 3]) -> [u8; 8] {
    let temp1 = des_crypt(block, &schedule[0]);
    let temp2 = des_crypt(&temp1, &schedule[1]);
    des_crypt(&temp2, &schedule[2])
}
