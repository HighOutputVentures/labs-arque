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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId {
    data: [u8;12],
}

impl Default for EventId {
    fn default() -> Self { EventId::new() }
}

impl EventId {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
            % 0xFFFFFFFF;

        let increment = unsafe {
            let mut index = INDEX.lock().unwrap();
            let next = (*index + 1u32) % 0xFFFFFF;
            *index = next;
            next
        };

        Self {
            data: [
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

    pub fn from_bytes(data: [u8;12]) -> Self {
        EventId { data }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use regex::Regex;
    use rstest::*;

    #[rstest]
    fn test_uniqueness() {
        let mut set = BTreeSet::new();

        for _n in 0..1000 {
            set.insert(EventId::new());
        }

        assert_eq!(set.len(), 1000, "ids should be unique");
    }

    #[rstest]
    fn test_to_str() {
        let id = EventId::new();

        let regex = Regex::new(r"^[0-9a-fA-F]{24}$").unwrap();
        assert!(
            regex.is_match(id.to_string().as_str()),
            "id string should match the regex pattern"
        );
    }

    #[rstest]
    fn test_to_bytes() {
        let id = EventId::new();

        assert_eq!(
            id.to_bytes().len(),
            12,
            "id bytes length should be equal to 12"
        );
    }
}
