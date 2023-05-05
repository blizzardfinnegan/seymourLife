pub mod log_facade;
pub mod tty;
pub mod gpio_facade;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_facade::setup_logs()?;
    error!("Test log creation");
    Ok(())
}
