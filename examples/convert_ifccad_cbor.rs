//! Explicit CAD-projection conversion; original foreign IFCX nodes are not copied.
use ocdraw::ifccad::*;
use std::ffi::OsString;

fn transport_options(
    args: &[OsString],
    writing: bool,
) -> Result<(bool, i32), Box<dyn std::error::Error>> {
    let mut zstd = false;
    let mut level = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].to_str() {
            Some("--zstd") if !zstd => zstd = true,
            Some("--level") if writing && level.is_none() => {
                i += 1;
                level = Some(
                    args.get(i)
                        .and_then(|s| s.to_str())
                        .ok_or("--level requires an integer in 1..=22")?
                        .parse::<i32>()?,
                );
            }
            _ => return Err("unknown, duplicate or inapplicable option; use --help".into()),
        }
        i += 1;
    }
    if level.is_some() && !zstd {
        return Err("--level requires --zstd".into());
    }
    let level = level.unwrap_or(3);
    if !(1..=22).contains(&level) {
        return Err("Zstd level must be in 1..=22".into());
    }
    Ok((zstd, level))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() == 2 && matches!(args[1].to_str(), Some("--help" | "-h")) {
        println!("convert_ifccad_cbor to-cbor INPUT.ifcx OUTPUT.ifccad.cbor [--zstd [--level 1..22]]\nconvert_ifccad_cbor to-json INPUT.ifccad.cbor OUTPUT.ifcx [--zstd]\nRaw CBOR is the default. Experimental --zstd requires feature ifccad-cbor-zstd; default level 3. Use .ifccad.cbor.zst for compressed files. The standard Zstd header identifies the method; compression level is a writer setting.\nConverts the supported CAD projection only; foreign IFCX nodes/fragments are not retained. Outputs must be new files.");
        return Ok(());
    }
    if args.len() < 4 {
        return Err("expected: to-cbor|to-json INPUT OUTPUT; use --help".into());
    }
    let (compressed, level) = transport_options(&args[4..], args[1] == "to-cbor")?;
    #[cfg(not(feature = "ifccad-cbor-zstd"))]
    if compressed {
        return Err("--zstd requires building with --features ifccad-cbor-zstd".into());
    }
    #[cfg(not(feature = "ifccad-cbor-zstd"))]
    let _ = level;
    match args[1].to_str() {
        Some("to-cbor") => {
            let source = load_ifccad_file(&args[2], Default::default())?;
            if compressed {
                #[cfg(feature = "ifccad-cbor-zstd")]
                encode_ifccad_cbor_zstd_document(
                    source.document(),
                    IfccadCborZstdEncodeOptions {
                        compression_level: level,
                    },
                )?
                .write_file(&args[3])?;
            } else {
                encode_ifccad_cbor_document(source.document())?.write_file(&args[3])?;
            }
        }
        Some("to-json") => {
            let source = if compressed {
                #[cfg(feature = "ifccad-cbor-zstd")]
                {
                    load_ifccad_cbor_zstd_file(&args[2], Default::default())?
                }
                #[cfg(not(feature = "ifccad-cbor-zstd"))]
                {
                    unreachable!("feature check above rejects compressed input")
                }
            } else {
                load_ifccad_cbor_file(&args[2], Default::default())?
            };
            encode_ifccad_document(source.document())?.write_file(&args[3])?;
        }
        _ => return Err("unknown conversion direction; use --help".into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn options(args: &[&str], writing: bool) -> Result<(bool, i32), Box<dyn std::error::Error>> {
        transport_options(
            &args.iter().map(OsString::from).collect::<Vec<_>>(),
            writing,
        )
    }
    #[test]
    fn raw_default_and_explicit_experimental_options() {
        assert_eq!(options(&[], true).unwrap(), (false, 3));
        assert_eq!(options(&["--zstd"], true).unwrap(), (true, 3));
        assert_eq!(
            options(&["--zstd", "--level", "9"], true).unwrap(),
            (true, 9)
        );
        assert_eq!(
            options(&["--level", "1", "--zstd"], true).unwrap(),
            (true, 1)
        );
        assert_eq!(options(&["--zstd"], false).unwrap(), (true, 3));
        for args in [
            vec!["--level", "3"],
            vec!["--zstd", "--zstd"],
            vec!["--zstd", "--level"],
            vec!["--zstd", "--level", "0"],
            vec!["--zstd", "--level", "23"],
            vec!["--unknown"],
        ] {
            assert!(options(&args, true).is_err(), "{args:?}");
        }
        assert!(options(&["--zstd", "--level", "3"], false).is_err());
    }
}
