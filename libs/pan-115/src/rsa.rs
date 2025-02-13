use num::BigUint;
use rand::Rng;

const N_HEX: &str = "8686980c0f5a24c4b9d43020cd2c22703ff3f450756529058b1cf88f09b8602136477198a6e2683149659bd122c33592fdb5ad47944ad1ea4d36c6b172aad6338c3bb6ac6227502d010993ac967d1aef00f0c8e038de2e4d3bc2ec368af2e9f10a6f1eda4f7262f136420c07c331b871bf139f74f3010e3c4fe57df3afb71683";
const E_HEX: &str = "10001";

lazy_static::lazy_static! {
    static ref N: BigUint = BigUint::parse_bytes(N_HEX.as_bytes(), 16).unwrap();
    static ref E: BigUint = BigUint::parse_bytes(E_HEX.as_bytes(), 16).unwrap();
    static ref KEY_LENGTH: usize = N.bits() as usize / 8usize;
}

pub fn rsa_encrypt(input: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();

    for chunk in input.chunks(*KEY_LENGTH - 11) {
        rsa_encrypt_slice(chunk, &mut buf);
    }

    buf
}

fn rsa_encrypt_slice(input: &[u8], buf: &mut Vec<u8>) {
    // Padding
    let pad_size = *KEY_LENGTH - input.len() - 3;
    let mut pad_data = vec![0u8; pad_size];
    rand::rng().fill(&mut pad_data[..]);

    // Prepare message
    let mut msg_buf = vec![0u8; *KEY_LENGTH];
    msg_buf[0] = 0;
    msg_buf[1] = 2;
    for (i, &b) in pad_data.iter().enumerate() {
        msg_buf[2 + i] = (b % 0xff) + 1;
    }
    msg_buf[pad_size + 2] = 0;
    msg_buf[pad_size + 3..].copy_from_slice(input);

    let msg = BigUint::from_bytes_be(&msg_buf);
    let ret = (msg.modpow(&E, &N)).to_bytes_be();

    // Fill zeros at beginning
    let fill_size = *KEY_LENGTH - ret.len();
    if fill_size > 0 {
        let zeros = vec![0u8; fill_size];
        buf.extend_from_slice(&zeros);
    }
    buf.extend_from_slice(&ret);
}

pub fn rsa_decrypt(input: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();

    for chunk in input.chunks(*KEY_LENGTH) {
        rsa_decrypt_slice(chunk, &mut buf);
    }

    buf
}

fn rsa_decrypt_slice(input: &[u8], buf: &mut Vec<u8>) {
    // RSA Decrypt
    let msg = BigUint::from_bytes_be(input);
    let ret = (msg.modpow(&E, &N)).to_bytes_be();

    // Un-padding
    for (i, &b) in ret.iter().enumerate() {
        // Find the beginning of plaintext
        if b == 0 && i != 0 {
            buf.extend_from_slice(&ret[i + 1..]);
            break;
        }
    }
}
