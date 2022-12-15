use std::iter::repeat_with;

pub fn random_bytes(len: usize) -> Vec<u8> {
    repeat_with(|| fastrand::u8(..)).take(len).collect()
}
