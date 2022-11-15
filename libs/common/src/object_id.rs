use once_cell::sync::Lazy;
use std::{
    process,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

static MACHINE_ID: Lazy<u32> = Lazy::new(|| (fastrand::f32() * 0xFFFFFF as f32).floor() as u32);

static PROCESS_ID: Lazy<u32> = Lazy::new(|| (process::id() % 0xFFFF) as u32);

static mut INDEX: Lazy<Mutex<u32>> =
    Lazy::new(|| Mutex::new((fastrand::f32() * 0xFFFFFF as f32).floor() as u32));

pub struct ObjectId {
    data: Vec<u8>,
}

impl ObjectId {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % 0xFFFFFFFF;

        let increment = unsafe {
            let mut index = INDEX.lock().unwrap();
            let next = (*index + 1u32) % 0xFFFFFF;
            *index = next;
            next
        };

        Self {
            data: vec![
                ((time >> 24) & 0xFF) as u8,
                ((time >> 16) & 0xFF) as u8,
                ((time >> 8) & 0xFF) as u8,
                (time & 0xFF) as u8,
                ((*MACHINE_ID >> 16) & 0xFF) as u8,
                ((*MACHINE_ID >> 8) & 0xFF) as u8,
                (*MACHINE_ID & 0xFF) as u8,
                ((*PROCESS_ID >> 8) & 0xFF) as u8,
                (*PROCESS_ID & 0xFF) as u8,
                ((increment >> 16) & 0xFF) as u8,
                ((increment >> 8) & 0xFF) as u8,
                (increment & 0xFF) as u8,
            ],
        }
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn to_string(&self) -> String {
        hex::encode(&self.data)
    }

    pub fn timestamp() -> u32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use rstest::*;

    #[rstest]
    fn test_uniqueness() {
        let object_id = ObjectId::new();
        let dummy_object_id = ObjectId::new();

        assert_ne!(
            object_id.data, dummy_object_id.data,
            "the two object ids should not be equal"
        );
    }

    #[rstest]
    fn test_to_str() {
        let id = ObjectId::new();

        let re = Regex::new(r"^[0-9a-fA-F]{24}$").unwrap();
        assert!(
            re.is_match(id.to_string().as_str()),
            "the object id string should match to the regex pattern"
        );
    }

    #[rstest]
    fn test_to_bytes() {
        let id = ObjectId::new();

        assert_eq!(
            id.to_bytes().len(),
            12,
            "the object id bytes length should be equal to 12"
        );
    }
}
