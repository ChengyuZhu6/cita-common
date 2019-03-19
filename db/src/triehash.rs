// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// This software is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This software is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Generetes trie root.
//!
//! This module should be used to generate trie root hash.

use hashable::Hashable;
use rlp;
use rlp::RlpStream;
use types::H256;
use util::SharedPrefix;

use std::cmp;
use std::collections::BTreeMap;

/// Generates a trie root hash for a vector of values
///
/// ```rust
/// extern crate db;
/// extern crate cita_types;
///
/// use std::str::FromStr;
/// use db::triehash::*;
/// use cita_types::H256;
///
/// fn main() {
///     let v = vec![From::from("doe"), From::from("reindeer")];
///
///     #[cfg(feature = "sha3hash")]
///     let root = "e766d5d51b89dc39d981b41bda63248d7abce4f0225eefd023792a540bcffee3";
///     #[cfg(feature = "blake2bhash")]
///     let root = "2e23216dd9a4b1bfa60a274b3420871d6560d55f45abb578aeaa547c79f7948b";
///     #[cfg(feature = "sm3hash")]
///     let root = "6eb00305e52ac9631000b13c27894bd34bd0dcd7d5e034a287f36d9429d7a7fd";
///
///     assert_eq!(ordered_trie_root(v), H256::from_str(root).unwrap());
/// }
/// ```
pub fn ordered_trie_root<I>(input: I) -> H256
where
    I: IntoIterator<Item = Vec<u8>>,
{
    let gen_input = input
        // first put elements into btree to sort them by nibbles
        // optimize it later
        .into_iter()
        .enumerate()
        .map(|(i, vec)| (rlp::encode(&i).into_vec(), vec))
        .collect::<BTreeMap<_, _>>()
        // then move them to a vector
        .into_iter()
        .map(|(k, v)| (as_nibbles(&k), v))
        .collect();

    gen_trie_root(gen_input)
}

/// Generates a trie root hash for a vector of key-values
///
/// ```rust
/// extern crate db;
/// extern crate cita_types;
///
/// use std::str::FromStr;
/// use db::triehash::*;
/// use cita_types::H256;
///
/// fn main() {
///     let v = vec![
///         (From::from("doe"), From::from("reindeer")),
///         (From::from("dog"), From::from("puppy")),
///         (From::from("dogglesworth"), From::from("cat")),
///     ];
///
///     #[cfg(feature = "sha3hash")]
///     let root = "8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3";
///     #[cfg(feature = "blake2bhash")]
///     let root = "82df7576318e4ab41bbe9b4e5c664c1d5e6d2558d4872ebdcce824ea91e004cb";
///     #[cfg(feature = "sm3hash")]
///     let root = "ac0c2b00e9f978a86713cc6dddea3972925f0d29243a2b51a3b597afaf1c7451";
///
///     assert_eq!(trie_root(v), H256::from_str(root).unwrap());
/// }
/// ```
pub fn trie_root<I>(input: I) -> H256
where
    I: IntoIterator<Item = (Vec<u8>, Vec<u8>)>,
{
    let gen_input = input
        // first put elements into btree to sort them and to remove duplicates
        .into_iter()
        .collect::<BTreeMap<_, _>>()
        // then move them to a vector
        .into_iter()
        .map(|(k, v)| (as_nibbles(&k), v))
        .collect();

    gen_trie_root(gen_input)
}

/// Generates a key-hashed (secure) trie root hash for a vector of key-values.
///
/// ```rust
/// extern crate db;
/// extern crate cita_types;
///
/// use std::str::FromStr;
/// use db::triehash::*;
/// use cita_types::H256;
///
/// fn main() {
///     let v = vec![
///         (From::from("doe"), From::from("reindeer")),
///         (From::from("dog"), From::from("puppy")),
///         (From::from("dogglesworth"), From::from("cat")),
///     ];
///
///     #[cfg(feature = "sha3hash")]
///     let root = "d4cd937e4a4368d7931a9cf51686b7e10abb3dce38a39000fd7902a092b64585";
///     #[cfg(feature = "blake2bhash")]
///     let root = "4fc4a4c6a187355054c79faace7e06619955ea670470c8d27a23ff59892c8ec6";
///     #[cfg(feature = "sm3hash")]
///     let root = "fc7b49b3492fec20bfd915c412a0c68c6d9110ddbf7c6606750f771f63f5f336";
///
///     assert_eq!(sec_trie_root(v), H256::from_str(root).unwrap());
/// }
/// ```
pub fn sec_trie_root(input: Vec<(Vec<u8>, Vec<u8>)>) -> H256 {
    let gen_input = input
        // first put elements into btree to sort them and to remove duplicates
        .into_iter()
        .map(|(k, v)| (k.crypt_hash().to_vec(), v))
        .collect::<BTreeMap<_, _>>()
        // then move them to a vector
        .into_iter()
        .map(|(k, v)| (as_nibbles(&k), v))
        .collect();

    gen_trie_root(gen_input)
}

fn gen_trie_root(input: Vec<(Vec<u8>, Vec<u8>)>) -> H256 {
    let mut stream = RlpStream::new();
    hash256rlp(&input, 0, &mut stream);
    stream.out().crypt_hash()
}

/// Hex-prefix Notation. First nibble has flags: oddness = 2^0 & termination = 2^1.
///
/// The "termination marker" and "leaf-node" specifier are completely equivalent.
///
/// Input values are in range `[0, 0xf]`.
///
/// ```markdown
///  [0,0,1,2,3,4,5]   0x10012345 // 7 > 4
///  [0,1,2,3,4,5]     0x00012345 // 6 > 4
///  [1,2,3,4,5]       0x112345   // 5 > 3
///  [0,0,1,2,3,4]     0x00001234 // 6 > 3
///  [0,1,2,3,4]       0x101234   // 5 > 3
///  [1,2,3,4]         0x001234   // 4 > 3
///  [0,0,1,2,3,4,5,T] 0x30012345 // 7 > 4
///  [0,0,1,2,3,4,T]   0x20001234 // 6 > 4
///  [0,1,2,3,4,5,T]   0x20012345 // 6 > 4
///  [1,2,3,4,5,T]     0x312345   // 5 > 3
///  [1,2,3,4,T]       0x201234   // 4 > 3
/// ```
fn hex_prefix_encode(nibbles: &[u8], leaf: bool) -> Vec<u8> {
    let inlen = nibbles.len();
    let oddness_factor = inlen % 2;
    // next even number divided by two
    let reslen = (inlen + 2) >> 1;
    let mut res = vec![];
    res.reserve(reslen);

    let first_byte = {
        let mut bits = ((inlen as u8 & 1) + (2 * leaf as u8)) << 4;
        if oddness_factor == 1 {
            bits += nibbles[0];
        }
        bits
    };

    res.push(first_byte);

    let mut offset = oddness_factor;
    while offset < inlen {
        let byte = (nibbles[offset] << 4) + nibbles[offset + 1];
        res.push(byte);
        offset += 2;
    }

    res
}

/// Converts slice of bytes to nibbles.
fn as_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut res = vec![];
    res.reserve(bytes.len() * 2);
    for i in 0..bytes.len() {
        res.push(bytes[i] >> 4);
        res.push((bytes[i] << 4) >> 4);
    }
    res
}

fn hash256rlp(input: &[(Vec<u8>, Vec<u8>)], pre_len: usize, stream: &mut RlpStream) {
    let inlen = input.len();

    // in case of empty slice, just append empty data
    if inlen == 0 {
        stream.append_empty_data();
        return;
    }

    // take slices
    let key: &[u8] = &input[0].0;
    let value: &[u8] = &input[0].1;

    // if the slice contains just one item, append the suffix of the key
    // and then append value
    if inlen == 1 {
        stream.begin_list(2);
        stream.append(&hex_prefix_encode(&key[pre_len..], true));
        stream.append(&value);
        return;
    }

    // get length of the longest shared prefix in slice keys
    let shared_prefix = input
        .iter()
        // skip first element
        .skip(1)
        // get minimum number of shared nibbles between first and each successive
        .fold(key.len(), |acc, &(ref k, _)| {
            cmp::min(key.shared_prefix_len(k), acc)
        });

    // if shared prefix is higher than current prefix append its
    // new part of the key to the stream
    // then recursively append suffixes of all items who had this key
    if shared_prefix > pre_len {
        stream.begin_list(2);
        stream.append(&hex_prefix_encode(&key[pre_len..shared_prefix], false));
        hash256aux(input, shared_prefix, stream);
        return;
    }

    // an item for every possible nibble/suffix
    // + 1 for data
    stream.begin_list(17);

    // if first key len is equal to prefix_len, move to next element
    let mut begin = match pre_len == key.len() {
        true => 1,
        false => 0,
    };

    // iterate over all possible nibbles
    for i in 0..16 {
        // cout how many successive elements have same next nibble
        let len = match begin < input.len() {
            true => input[begin..]
                .iter()
                .take_while(|pair| pair.0[pre_len] == i)
                .count(),
            false => 0,
        };

        // if at least 1 successive element has the same nibble
        // append their suffixes
        match len {
            0 => {
                stream.append_empty_data();
            }
            _ => hash256aux(&input[begin..(begin + len)], pre_len + 1, stream),
        }
        begin += len;
    }

    // if fist key len is equal prefix, append its value
    match pre_len == key.len() {
        true => {
            stream.append(&value);
        }
        false => {
            stream.append_empty_data();
        }
    };
}

fn hash256aux(input: &[(Vec<u8>, Vec<u8>)], pre_len: usize, stream: &mut RlpStream) {
    let mut s = RlpStream::new();
    hash256rlp(input, pre_len, &mut s);
    let out = s.out();
    match out.len() {
        0...31 => stream.append_raw(&out, 1),
        _ => stream.append(&out.crypt_hash()),
    };
}

#[test]
fn test_nibbles() {
    let v = vec![0x31, 0x23, 0x45];
    let e = vec![3, 1, 2, 3, 4, 5];
    assert_eq!(as_nibbles(&v), e);

    // A => 65 => 0x41 => [4, 1]
    let v: Vec<u8> = From::from("A");
    let e = vec![4, 1];
    assert_eq!(as_nibbles(&v), e);
}

#[test]
fn test_hex_prefix_encode() {
    let v = vec![0, 0, 1, 2, 3, 4, 5];
    let e = vec![0x10, 0x01, 0x23, 0x45];
    let h = hex_prefix_encode(&v, false);
    assert_eq!(h, e);

    let v = vec![0, 1, 2, 3, 4, 5];
    let e = vec![0x00, 0x01, 0x23, 0x45];
    let h = hex_prefix_encode(&v, false);
    assert_eq!(h, e);

    let v = vec![0, 1, 2, 3, 4, 5];
    let e = vec![0x20, 0x01, 0x23, 0x45];
    let h = hex_prefix_encode(&v, true);
    assert_eq!(h, e);

    let v = vec![1, 2, 3, 4, 5];
    let e = vec![0x31, 0x23, 0x45];
    let h = hex_prefix_encode(&v, true);
    assert_eq!(h, e);

    let v = vec![1, 2, 3, 4];
    let e = vec![0x00, 0x12, 0x34];
    let h = hex_prefix_encode(&v, false);
    assert_eq!(h, e);

    let v = vec![4, 1];
    let e = vec![0x20, 0x41];
    let h = hex_prefix_encode(&v, true);
    assert_eq!(h, e);
}

#[cfg(test)]
mod tests {
    use super::trie_root;
    use std::str::FromStr;
    use types::H256;

    #[test]
    fn simple_test() {
        let data = trie_root(vec![(
            b"A".to_vec(),
            b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_vec(),
        )]);

        #[cfg(feature = "sha3hash")]
        let hex_str = "d23786fb4a010da3ce639d66d5e904a11dbc02746d1ce25029e53290cabf28ab";

        #[cfg(feature = "blake2bhash")]
        let hex_str = "8901a2291955fc6eb443a0175ce2ab218157e571e29b09aaf3dc2da3946b2dfa";

        #[cfg(feature = "sm3hash")]
        let hex_str = "3bf73fbd4b9efb0896a40701aebbbb6d40bb1d14f4421d8c3f60bd522f5fd0fb";

        assert_eq!(data, H256::from_str(hex_str).unwrap());
    }

    #[test]
    fn test_triehash_out_of_order() {
        assert!(
            trie_root(vec![
                (vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
                (vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
                (vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
            ]) == trie_root(vec![
                (vec![0x01u8, 0x23], vec![0x01u8, 0x23]),
                (vec![0xf1u8, 0x23], vec![0xf1u8, 0x23]),
                (vec![0x81u8, 0x23], vec![0x81u8, 0x23]),
            ])
        );
    }

}