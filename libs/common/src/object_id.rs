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
    use rstest::*;

    #[rstest]
    fn test_uniqueness() {
        let p = ObjectId::new();
        let x = ObjectId::new();

        assert_ne!(p.id, x.id, "the two object ids should not be equal");
    }

    #[rstest]
    fn test_increment() {
        let p = ObjectId::new();
        let x = ObjectId::new();

        assert!(
            p.id.last().unwrap() < x.id.last().unwrap(),
            "the second object id should be greater than the first object id"
        );
    }
}
