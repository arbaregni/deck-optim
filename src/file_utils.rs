use std::fs::File;
use std::io::{self, BufWriter};
use std::io::Read;
use std::path::PathBuf;


#[derive(Debug)]
pub enum ArgumentReadError {
    FailedToOpenFile {
        file_name: PathBuf,
        cause: io::Error,
    },
    ErrorWhileReadingFile {
        file_name: PathBuf,
        cause: io::Error,
    },
    InvalidJson {
        file_name: PathBuf,
        cause: serde_json::Error,
        report: String,
    }
}

pub fn read_json_from_path<T>(path: &PathBuf) -> Result<T, ArgumentReadError>
where T: serde::de::DeserializeOwned
{
    let mut file = File::open(path)
        .map_err(|cause| ArgumentReadError::FailedToOpenFile { 
            file_name: path.clone(), 
            cause
        })?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|cause| ArgumentReadError::ErrorWhileReadingFile { 
            file_name: path.clone(),
            cause
        })?;
    let out = serde_json::from_str(&buf)
        .map_err(|cause| {
            let report = build_report_for_json_error(path, &cause, &buf);
            ArgumentReadError::InvalidJson {
                file_name: path.clone(),
                cause,
                report
            }
        })?;
    Ok(out)

}

impl std::error::Error for ArgumentReadError { }

impl std::fmt::Display for ArgumentReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArgumentReadError::*;
        match self {
            FailedToOpenFile { file_name, cause } => write!(f, "failed to open file {} for reading: {cause}", file_name.display()),
            ErrorWhileReadingFile { file_name, cause } => write!(f, "failed to read file {}: {cause}", file_name.display()),
            InvalidJson { file_name, cause, report } => {
                write!(f, "{report}")
                // write!(f, "failed to parse json from {}: {cause}", file_name.display())
            }
        }
    }
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
