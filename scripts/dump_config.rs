use galileo::config::AppConfig;
use std::fs;

fn main() {
    let data = fs::read_to_string("galileo.yaml").unwrap();
    let cfg: AppConfig = serde_yaml::from_str(&data).unwrap();
    println!(
        "default? {} named={} keys={:?}",
        cfg.galileo.engine.jupiter.default_config().is_some(),
        cfg.galileo.engine.jupiter.named_configs().len(),
        cfg.galileo.engine.jupiter.named_configs().keys().collect::<Vec<_>>()
    );
}
