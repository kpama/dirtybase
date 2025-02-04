pub use uuid::Uuid;
pub use uuid25::Uuid25;

pub fn uuid_v4() -> Uuid {
    Uuid::new_v4()
}

pub fn uuid_v4_string() -> String {
    uuid_v4().to_string()
}

pub fn uuid_v4_from_str(input: &str) -> Option<Uuid> {
    if let Ok(x) = Uuid::parse_str(input) {
        if x.get_version_num() == 4 {
            return Some(x);
        }
    }
    None
}

pub fn uuid_v7() -> Uuid {
    Uuid::now_v7()
}

pub fn uuid_v7_string() -> String {
    uuid_v7().to_string()
}

pub fn uuid25_v4() -> Uuid25 {
    uuid25::gen_v4()
}

pub fn uuid25_v4_string() -> String {
    uuid25_v4().to_string()
}

pub fn uuid25_from_str(input: &str) -> Option<Uuid25> {
    uuid25::Uuid25::parse(input).ok()
}

pub fn uuid25_v7() -> Uuid25 {
    uuid25::gen_v7()
}

pub fn uuid25_v7_string() -> String {
    uuid25_v7().to_string()
}

pub fn uuid_v7_from_str(input: &str) -> Option<Uuid> {
    if let Ok(x) = Uuid::parse_str(input) {
        if x.get_version_num() == 7 {
            return Some(x);
        }
    }
    None
}
