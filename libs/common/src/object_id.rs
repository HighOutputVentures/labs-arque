use once_cell::sync::Lazy;
use std::{
    process, str,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

static MACHINE_ID: Lazy<u32> = Lazy::new(|| (fastrand::f32() * 0xFFFFFF as f32).floor() as u32);

static PROCESS_ID: Lazy<u32> = Lazy::new(|| (process::id() % 0xFFFF) as u32);

static mut INDEX: Lazy<Mutex<u32>> =
    Lazy::new(|| Mutex::new((fastrand::f32() * 0xFFFFFF as f32).floor() as u32));

pub struct ObjectId {
    id: Vec<u8>,
    string: String,
}

impl ObjectId {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % 0xFFFFFFFF;

        let increment = unsafe {
            let next = (*INDEX.lock().unwrap() + 1u32) % 0xFFFFFF;
            *INDEX.lock().unwrap() = next;
            next
        };

        Self {
            id: vec![
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
            string: String::new(),
        }
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.id
    }

    pub fn to_str(&mut self) -> &str {
        self.string = hex::encode(&self.id);

        &self.string
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use rstest::*;

    #[rstest]
    fn test_timestamp() {
        let object_id = ObjectId::new();

        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            % 0xFFFFFFFF;

        let timestamp = object_id.id[3] as u64
            | (object_id.id[2] as u64) << 8
            | (object_id.id[1] as u64) << 16
            | (object_id.id[0] as u64) << 24;

        assert_eq!(current_timestamp, timestamp, "the timestamp should be equal");
    }

    #[rstest]
    fn test_machine_id() {
        let object_id = ObjectId::new();

        let machine_id =
            object_id.id[6] as u32 | (object_id.id[5] as u32) << 8 | (object_id.id[4] as u32) << 16;

        assert_eq!(machine_id, *MACHINE_ID, "the machine id should be equal");
    }

    #[rstest]
    fn test_process_id() {
        let object_id = ObjectId::new();

        let process_id =
            object_id.id[8] as u32 | (object_id.id[7] as u32) << 8;

        assert_eq!(process_id, *PROCESS_ID, "the process id should be equal");
    }

    #[rstest]
    fn test_uniqueness() {
        let object_id = ObjectId::new();
        let dummy_object_id = ObjectId::new();

        assert_ne!(
            object_id.id, dummy_object_id.id,
            "the two object ids should not be equal"
        );
    }

    #[rstest]
    fn test_increment() {
        let object_id = ObjectId::new();
        let dummy_object_id = ObjectId::new();

        assert!(
            object_id.id.last().unwrap() < dummy_object_id.id.last().unwrap(),
            "the second object id should be greater than the first object id"
        );
    }

    #[rstest]
    fn test_to_str() {
        let mut object_id = ObjectId::new();

        let re = Regex::new(r"^[0-9a-fA-F]{24}$").unwrap();
        assert!(
            re.is_match(object_id.to_str()),
            "the object id string should match to the regex pattern"
        );
    }

    #[rstest]
    fn test_to_bytes() {
        let object_id = ObjectId::new();

        assert_eq!(
            object_id.to_bytes().len(),
            12,
            "the object id bytes length should be equal to 12"
        );
    }
}
