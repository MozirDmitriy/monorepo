#![no_main]

use arbitrary::Arbitrary;
use blake3::Hasher as RefBlake3;
use commonware_codec::{DecodeExt, Encode};
use commonware_cryptography::{
    blake3::{hash as our_hash, Blake3 as OurBlake3, Digest},
    Hasher,
};
use libfuzzer_sys::fuzz_target;

#[derive(Debug, Arbitrary)]
pub struct FuzzInput {
    pub chunks: Vec<Vec<u8>>,
    pub data: Vec<u8>,
    pub case_selector: u8,
}

// Basic hashing comparison with chunks
fn fuzz_basic_hashing(chunks: &[Vec<u8>]) {
    let mut our_hasher = OurBlake3::new();
    let mut ref_hasher = RefBlake3::new();

    for chunk in chunks {
        our_hasher.update(chunk);
        ref_hasher.update(chunk);
    }

    let our_result = our_hasher.finalize();
    let ref_result = ref_hasher.finalize();
    assert_eq!(our_result.as_ref(), ref_result.as_bytes());
}

// Reset functionality
fn fuzz_reset_functionality(chunks: &[Vec<u8>]) {
    let mut our_hasher = OurBlake3::new();
    let mut ref_hasher = RefBlake3::new();

    // First round
    for chunk in chunks {
        our_hasher.update(chunk);
        ref_hasher.update(chunk);
    }
    let our_result = our_hasher.finalize();
    let ref_result = ref_hasher.finalize();
    assert_eq!(our_result.as_ref(), ref_result.as_bytes());

    // Reset and second round
    our_hasher.reset();
    let mut ref_hasher = RefBlake3::new();

    for chunk in chunks {
        our_hasher.update(chunk);
        ref_hasher.update(chunk);
    }

    let our_result_after_reset = our_hasher.finalize();
    let ref_result_after_reset = ref_hasher.finalize();
    assert_eq!(our_result, our_result_after_reset);
    assert_eq!(
        our_result_after_reset.as_ref(),
        ref_result_after_reset.as_bytes()
    );
}

// Chunked vs all-at-once hashing
fn fuzz_chunked_vs_whole(chunks: &[Vec<u8>]) {
    let mut our_hasher = OurBlake3::new();
    let mut ref_hasher = RefBlake3::new();
    let mut all_data = Vec::new();

    for chunk in chunks {
        all_data.extend_from_slice(chunk);
        our_hasher.update(chunk);
    }

    let our_final = our_hasher.finalize();

    let ref_final = ref_hasher.update(&all_data).finalize();
    assert_eq!(our_final.as_ref(), ref_final.as_bytes());
}

// Differential fuzzing
fn fuzz_diff_hash(data: &[u8]) {
    let our_hash_result = our_hash(data);
    let mut ref_hasher = RefBlake3::new();
    assert_eq!(
        our_hash_result.as_ref(),
        ref_hasher.update(data).finalize().as_bytes()
    );
}

// Encode/decode functionality
fn fuzz_encode_decode(data: &[u8]) {
    let mut hasher = OurBlake3::new();
    hasher.update(data);
    let digest = hasher.finalize();

    let encoded = digest.encode();
    assert_eq!(encoded.len(), 32); // DIGEST_LENGTH = 32
    assert_eq!(encoded, digest.as_ref());

    let decoded = Digest::decode(encoded).unwrap();
    assert_eq!(digest, decoded);
}

fn fuzz(input: FuzzInput) {
    match input.case_selector % 5 {
        0 => fuzz_basic_hashing(&input.chunks),
        1 => fuzz_reset_functionality(&input.chunks),
        2 => fuzz_chunked_vs_whole(&input.chunks),
        3 => fuzz_diff_hash(&input.data),
        4 => fuzz_encode_decode(&input.data),
        _ => unreachable!(),
    }
}

fuzz_target!(|input: FuzzInput| {
    fuzz(input);
});
