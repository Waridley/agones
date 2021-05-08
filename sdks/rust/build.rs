use std::{path::{PathBuf}, env::var};
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	println!("cargo:rerun-if-changed=../../proto");
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed={}", var("OUT_DIR").unwrap());
	
	let mut builder = tonic_build::configure();
	
	builder = match var("CARGO_FEATURE_CLIENT") {
		Ok(s) if s == "1" => builder.build_client(true),
		_ => { builder.build_client(false) }
	};
	
	builder = match var("CARGO_FEATURE_SERVER") {
		Ok(s) if s == "1" => builder.build_server(true),
		_ => { builder.build_server(false) }
	};
	
	let include_path = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap())
		.parent().unwrap() // sdks
		.parent().unwrap() // agones
		.join("proto");
	
	let protos = vec![
		var("CARGO_FEATURE_ALLOCATION").ok()
			.and_then(|s| (s == "1").then(|| include_path.join("allocation/allocation.proto"))),
		var("CARGO_FEATURE_SDK").ok()
			.and_then(|s| (s == "1").then(|| include_path.join("sdk/sdk.proto"))),
		var("CARGO_FEATURE_ALPHA").ok()
			.and_then(|s| (s == "1").then(|| include_path.join("sdk/alpha/alpha.proto"))),
		var("CARGO_FEATURE_BETA").ok()
			.and_then(|s| (s == "1").then(|| include_path.join("sdk/beta/beta.proto"))),
	].into_iter()
		.filter(|opt| opt.is_some())
		.map(|opt| opt.unwrap())
		.collect::<Vec<_>>();
	
	dbg!(builder.compile(
		&*protos,
		&[
			include_path.join("googleapis"),
			include_path,
		],
	))?;
	
	//TODO: There's probably a better way to parse the proto files to find the includes
	let rs_incl_dir = PathBuf::from(var("OUT_DIR").unwrap()).join("includes");
	std::fs::create_dir_all(&rs_incl_dir)?;
	for path in protos {
		File::open(&path)
			.and_then(|proto_file| {
				let reader = BufReader::new(proto_file);
				let mut includes = vec![];
				for line in reader.lines() {
					if let Ok(line) = line {
						if line.starts_with("message ") {
							let line = line.trim_start_matches("message ");
							let non_alpha = line.find(|c: char| !c.is_ascii_alphabetic()).unwrap();
							includes.push(line[..non_alpha].trim().to_string());
						}
					}
				}
				File::create(rs_incl_dir
					.join(&*path
						.file_name().unwrap()
						.to_str().unwrap()
						.replace(".proto", ".rs")
					)
				)
					.and_then(|mut out_file| {
						out_file.write(b"pub use crate::proto::{")?;
						out_file.write(includes.join(",").as_bytes())?;
						out_file.write(b"};")?;
						Ok(())
					})
			})?;
	}
	
	Ok(())
}