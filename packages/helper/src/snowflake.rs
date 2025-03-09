use std::sync::OnceLock;

use snowflaker::generator::{Generator, SnowflakeGenerator};

static GENERATOR: OnceLock<SnowflakeGenerator> = OnceLock::new();

pub fn generate_snowflake_id() -> u64 {
    let generator = GENERATOR.get_or_init(|| {
        SnowflakeGenerator::dynamic().expect("could not create snowflake generator")
    });

    generator
        .next_id()
        .expect("could not generate snowflake id")
}
