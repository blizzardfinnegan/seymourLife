use log::{trace,debug,info,warn,error};
use std::time::SystemTime;

fn setup_logger() -> Result<(), fern::InitError>{
    fern::Dispatch::new()
        .format(|out,message,record|{
            out.finish(format_args!(
                "{} - [{}, {}] - {}",
                humantime::format_rfc3339_millis(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(fern::log_file("output.log")?)
        .level(log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;
    trace!("Test trace message");
    debug!("Test trace message");
    info!("Test trace message");
    warn!("Test trace message");
    error!("Test trace message");
    Ok(())
}
