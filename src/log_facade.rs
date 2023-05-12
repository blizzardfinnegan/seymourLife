use chrono::prelude::*;
use std::path::Path;
use std::fs;

pub fn setup_logs() -> Result<(), fern::InitError>{
    let chrono_now: DateTime<Local> = Local::now();
    if ! Path::new("logs").is_dir(){
        _ = fs::create_dir("logs");
    };
    fern::Dispatch::new()
        .format(|out,message,record|{
            out.finish(format_args!(
                "{} - [{}, {}] - {}",
                Local::now().to_rfc3339(),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Trace)
                .chain(fern::log_file(
                    format!("logs/{0}.log",
                    chrono_now.format("%Y-%m-%d_%H.%M").to_string()
                    ))?),
        )
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Info)
                .chain(std::io::stdout())
        )
        .apply()?;
    Ok(())
}
