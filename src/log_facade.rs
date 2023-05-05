use chrono::prelude::*;

pub fn setup_logs() -> Result<(), fern::InitError>{
    let chrono_now: DateTime<Local> = Local::now();
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
                    format!("logs\\output-{0}.log",
                    chrono_now.format("%Y-%m-%d_%H.%M.%S").to_string()
                    ))?),
        )
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Warn)
                .chain(std::io::stdout())
        )
        .apply()?;
    Ok(())
}
