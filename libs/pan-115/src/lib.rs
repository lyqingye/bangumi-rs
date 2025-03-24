#![allow(dead_code)]
pub mod client;
pub mod dir;
mod download;
pub mod errors;
mod file;
mod iter;
pub mod model;
mod offline;
mod rsa;
mod xor;

use base64::{Engine as _, engine::general_purpose};
use errors::Pan115Error;
use rand::Rng;

use anyhow::Result;
use rsa::{rsa_decrypt, rsa_encrypt};
use xor::*;

// 16 bytes
type Key = [u8; 16];

pub fn gen_key() -> Key {
    rand::rng().random::<Key>()
}

pub fn encode(input: &[u8], key: &[u8; 16]) -> String {
    let mut buf: Vec<u8> = [key, input].concat();
    xor_transform(&mut buf[16..], &xor_derive_key(key, 4));
    buf[16..].reverse();
    xor_transform(&mut buf[16..], &XOR_CLIENT_KEY);
    general_purpose::STANDARD.encode(rsa_encrypt(&buf))
}

pub fn decode(input: &str, key: &[u8; 16]) -> Result<Vec<u8>, Pan115Error> {
    let buf = general_purpose::STANDARD
        .decode(input)
        .map_err(|_| Pan115Error::DecodeFailed(input.to_string()))?;
    let buf = rsa_decrypt(&buf);
    let mut output = buf[16..].to_vec();
    xor_transform(&mut output, &xor_derive_key(&buf[..16], 12));
    output.reverse();
    xor_transform(&mut output, &xor_derive_key(key, 4));
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        let key_bytes = hex::decode("eeee8b2bae1fd9b6d22adbb9e4256819").unwrap();
        let mut key: [u8; 16] = [0; 16];
        key.copy_from_slice(&key_bytes);

        let input = hex::decode("74657374").unwrap();
        println!("{}", hex::encode(input.as_slice()));
        let mut buf: Vec<u8> = [key.as_slice(), input.as_slice()].concat();
        xor_transform(&mut buf[16..], &xor_derive_key(&key, 4));
        buf[16..].reverse();
        xor_transform(&mut buf[16..], &XOR_CLIENT_KEY);

        println!("{}", hex::encode(buf));
    }

    #[test]
    fn test_decode2() {
        let key_bytes = hex::decode("679db9b23d59d677a0b79a9e2b840b4c").unwrap();
        let mut key: [u8; 16] = [0; 16];
        key.copy_from_slice(&key_bytes);
        let decoded = "gjAKvuA4/sABqZ/Mtsi61F+eosNQ5yqxnyYfUV+CkJhc0+Au4aNIFdhg5XQD6LKHO3Db1HWUgEFnR8+0XXwGzv4oKvGJexSoI2mcnLX5n3VAjaZKpqhGx7nr/fF0Iz4pPIhNNIYNNYUMa8prGWKVn1Z9MC51T0oY2zgloqzAynxZokQYy6Z5ubQxtHA78zr+pHM8TzS2np/Gt8LBXx9sZtnGOrpjIDwJhJstMKflppjDGxbxiS23H0rHaPFCXaY2/1bJVO9KuPGGkap+Je1HElisyePj9Vt2SxuAH9Q+afGU7Pmiyjj6LL/ZF9Y3uodlAqGtSYp0MU3XMFqaeLfz+jrpaCuS4kO2R2yBgW5TbzblEtVFO3vm/Xa5uK9wAD/IMnardVadW+B3jRXtMslvsvwqxuXm7Ryv61W2NPM/tV6gLYEfE8oacblEOm5p42Hg+VIEltlQHVp/MXSISNJCzalFRdmkkPNnRY7pOsjCS2ii4ToaKSNtNEsE1sccQAun";
        let decoded = decode(decoded, &key).unwrap();
        println!("{}", String::from_utf8(decoded).unwrap());
    }
}
