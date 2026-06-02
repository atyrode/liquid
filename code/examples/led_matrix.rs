#[cfg(feature = "led-matrix")]
fn main() -> Result<(), String> {
    fluid_sim::led_matrix::run_from_env()
}

#[cfg(not(feature = "led-matrix"))]
fn main() -> Result<(), String> {
    Err(
        "The LED matrix renderer is disabled. Rebuild with --no-default-features --features led-matrix."
            .to_string(),
    )
}
