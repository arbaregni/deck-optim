use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug,Error)]
pub enum ArgumentReadError {
    #[error("failed to open {} for reading: {source}", file_name.display())]
    FailedToOpenFile { file_name: PathBuf, source: io::Error },
    #[error("error while reading file {}: {source}", file_name.display())]
    ErrorWhileReadingFile {
        file_name: PathBuf,
        source: io::Error,
    },
    #[allow(dead_code)]
    #[error("invalid json at {}: {source}", file_name.display())]
    InvalidJson {
        file_name: PathBuf,
        source: serde_json::Error,
        report: String,
    }
}

pub fn read_json_from_path<T>(path: &PathBuf) -> Result<T, ArgumentReadError>
where T: serde::de::DeserializeOwned
{
    let mut file = File::open(path)
        .map_err(|source| ArgumentReadError::FailedToOpenFile { 
            file_name: path.clone(), 
            source
        })?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|source| ArgumentReadError::ErrorWhileReadingFile { 
            file_name: path.clone(),
            source
        })?;
    let out = serde_json::from_str(&buf)
        .map_err(|source| {
            let report = build_report_for_json_error(path, &source, &buf);
            ArgumentReadError::InvalidJson {
                file_name: path.clone(),
                source,
                report
            }
        })?;
    Ok(out)

}

pub fn write_json_to_path<T: serde::ser::Serialize>(path: &PathBuf,  data: &T) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        log::info!("creating parent directory at {}", parent.display());
        std::fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    let bytes = serde_json::to_vec(data)?;
    file.write_all(&bytes)?;
    Ok(())
}

fn build_report_for_json_error(file_name: &PathBuf, cause: &serde_json::Error, source: &str) -> String {
    use ariadne::Report;
    use ariadne::ReportKind;
    use ariadne::Source;
    use ariadne::Label;
    use ariadne::Color;

    let line = source.lines()
        .nth(cause.line() - 1)
        .expect("go to nth line");

    let line_start = unsafe {
        // SAFETY: these pointers are from the same allocation
        (line.as_ptr() as *const u8).offset_from(source.as_ptr() as *const u8)
    };
    let start = line_start as usize + cause.column(); // maybe not accurate. depends on if serde_json::Error
                                             // counts chars as multiple columns or not

    let span = start..start+1;
    let display_name = file_name.display().to_string();
    let span = (display_name.as_str(), span);
    
    let cache = (display_name.as_str(), Source::from(source));

    let mut buf = Vec::<u8>::new();
    Report::build(ReportKind::Error, span.clone()) 
        .with_message(format!("invalid json: {cause}"))
        .with_label(
            Label::new(span)
                .with_message("this is invalid")
                .with_color(Color::Red)
        )
        .finish()
        .write(cache, &mut buf)
        .expect("write report to buffer");

    String::from_utf8(buf)
        .expect("not valid utf8")
}
