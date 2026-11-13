use std::{borrow::Cow, collections::HashMap, io::Read, process::ExitCode};

use device_driver_diagnostics::{DynError, ResultExt};
use device_driver_trace::decoder::{DecodeError, StreamingDecoder};

use crate::{DecodeArgs, DecodeIngest, EofBehavior};

pub fn decode(args: DecodeArgs) -> Result<ExitCode, DynError> {
    let manifest_paths = match args.manifests {
        Some(manifests) => manifests,
        None => std::fs::read_dir(
            std::env::current_dir().with_message(|| "determining current directory")?,
        )
        .with_message(|| "reading current dir")?
        .filter_map(|entry| {
            entry
                .map(|entry| {
                    let path = entry.path();
                    (path.extension().map(|ext| ext.to_string_lossy())
                        == Some(Cow::Borrowed("ddsl")))
                    .then_some(path)
                })
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()
        .with_message(|| "collecting ddsl files from current dir")?,
    };

    let mut manifests = HashMap::new();
    for path in manifest_paths {
        let source = std::fs::read_to_string(&path)
            .with_message(|| format!("reading ddsl file: {}", path.display()))?;

        let mut diagnostics = Default::default();
        let tokens = device_driver_lexer::lex(&source);
        let ast = device_driver_parser::parse(&tokens, &mut diagnostics);
        let (manifest, _) =
            device_driver_mir::lower_ast(&ast, &Default::default(), &mut diagnostics).unwrap();
        if diagnostics.has_error() {
            return Err(DynError::new(format!(
                "manifest at {} compiles with {} errors",
                path.display(),
                diagnostics.summary().errors
            )));
        }

        if manifests
            .insert(device_driver_trace::crc32(source.as_bytes()), manifest)
            .is_some()
        {
            return Err(DynError::new(format!(
                "manifest {} is included multiple times",
                path.display()
            )));
        }
    }

    let mut source: Box<dyn Read> = match args.input {
        DecodeIngest::Stdin => Box::new(std::io::stdin().lock()),
        DecodeIngest::File { path } => Box::new(
            std::fs::File::open(&path)
                .with_message(|| format!("opening trace file {}", path.display()))?,
        ),
    };

    let mut decoder = StreamingDecoder::new();

    let mut buffer = [0; 1024];
    loop {
        let buf_len = source
            .read(&mut buffer)
            .with_message(|| "reading data source")?;

        if matches!(args.eof, EofBehavior::Exit) && buf_len == 0 {
            break;
        }

        for event in decoder.push_bytes(&buffer[..buf_len]) {
            match event {
                Ok(event) => match device_driver_trace::correlator::correlate(&manifests, event) {
                    Ok(e) => {
                        println!("{e}");
                    }
                    Err(e) => println!("event correlation error: {e}"),
                },
                Err(DecodeError::Incomplete) => println!("received incomplete event"),
                Err(DecodeError::Malformed) => println!("received malformed event"),
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
