#![feature(let_chains)]

use minimap_core::{GitRemote, Record, Workspace};
use std::{fs::Metadata, path::PathBuf};

#[derive(Debug, thiserror::Error)]
enum Error {
	#[error(transparent)]
	Minimap(#[from] minimap_core::Error),
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error("failed to parse .minimap file: {0}: {1}")]
	Toml(toml::de::Error, PathBuf),
	#[error("no .minimap file found (hit filesystem boundary)")]
	NoDotMinimap,
}

type Result<T> = std::result::Result<T, Error>;

fn main() {
	std::process::exit(pmain());
}

fn pmain() -> i32 {
	let mut args = std::env::args().fuse();
	let arg0 = args.next();

	// Here, we expand out `-abcd` into `-a -b -c -d`,
	// and `--foo=bar` into `--foo bar`.
	let mut args = args.flat_map(|arg| {
		let mut chars = arg.chars();
		let mut expanded = vec![];

		if chars.next() == Some('-') {
			match chars.next() {
				Some('-') => {
					arg.split_once('=')
						.map(|(key, value)| {
							expanded.push(key.into());
							expanded.push(value.into());
						})
						.unwrap_or_else(|| expanded.push(arg));
				}
				Some(c) => {
					expanded.push(format!("-{}", c));
					expanded.extend(chars.map(|c| format!("-{}", c)));
				}
				None => {
					expanded.push(arg);
				}
			}
		} else {
			expanded.push(arg);
		}

		expanded.into_iter()
	});

	let mut precommand_args = vec![];

	let subcommand = {
		let mut last = args.next();
		while last.as_ref().map(|s| s.starts_with('-')).unwrap_or(false) {
			let arg = last.unwrap();
			let should_break = arg == "--";
			precommand_args.push(arg);
			last = args.next();
			if should_break {
				break;
			}
		}
		last
	};

	let args = args.collect::<Vec<_>>();

	let mut precommand_args = precommand_args.into_iter();
	while let Some(arg) = precommand_args.next() {
		match arg.as_str() {
			"--help" => return show_usage(arg0),
			"--version" => {
				eprintln!("minimap {}", env!("CARGO_PKG_VERSION"));
				return 2;
			}
			"-C" => {
				if let Some(dir) = precommand_args.next() {
					if !std::env::set_current_dir(&dir).is_ok() {
						eprintln!("error: failed to change directory to `{}`", dir);
						return 1;
					}
				} else {
					eprintln!("error: missing argument to `-C`");
					return 1;
				}
			}
			unknown => {
				eprintln!("error: unknown argument `{}`\n", unknown);
				return show_usage(arg0);
			}
		};
	}

	let result = match subcommand.as_ref().map(|s| s.as_str()) {
		Some("workspace") => cmd_workspace(arg0.as_ref().map(|s| s.as_str()), &args),
		Some("project") => cmd_project(arg0.as_ref().map(|s| s.as_str()), &args),
		Some(unknown) => {
			eprintln!("error: unknown subcommand `{}`\n", unknown);
			Ok(show_usage(arg0))
		}
		None => Ok(show_usage(arg0)),
	};

	match result {
		Ok(code) => std::process::exit(code),
		Err(err) => {
			eprintln!("error: {}", err);
			std::process::exit(1);
		}
	}
}

fn show_usage(arg0: Option<String>) -> i32 {
	let arg0 = arg0.unwrap_or_else(|| "minimap".to_string());
	eprintln!(
		concat!(
			"minimap ",
			env!("CARGO_PKG_VERSION"),
			"\n",
			"\n",
			"usage: {arg0} [--version] [--help] <command> [<args>]\n",
			"\n",
			"Available commands:\n",
			"\n",
			"interacting with workspaces:\n",
			"workspace name     Gets or sets the workspace name\n"
		),
		arg0 = arg0
	);
	2
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum DotMinimapRemoteType {
	Git,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct DotMinimap {
	remote: String,
	#[serde(rename = "type")]
	remote_type: DotMinimapRemoteType,
}

#[cfg(unix)]
fn has_hit_filesystem_boundary(last: &Metadata, current: &Metadata) -> bool {
	use std::os::unix::fs::MetadataExt;
	last.dev() != current.dev()
}

#[cfg(not(unix))]
fn has_hit_filesystem_boundary(_last: &Metadata, _current: &Metadata) -> bool {
	false
}

fn open_workspace<'a>() -> Result<Workspace<'a, GitRemote>> {
	let minimap_file = {
		let mut current_dir = std::env::current_dir()?;
		let mut last_stats = std::fs::metadata(&current_dir)?;
		loop {
			let minimap_file = current_dir.join(".minimap");
			if minimap_file.is_file() {
				break minimap_file;
			}

			let give_up = if let Some(next_dir) = current_dir.parent() {
				if next_dir == current_dir {
					true
				} else {
					let stats = std::fs::metadata(&next_dir)?;

					if has_hit_filesystem_boundary(&last_stats, &stats) {
						true
					} else {
						current_dir = next_dir.to_path_buf();
						last_stats = stats;
						false
					}
				}
			} else {
				true
			};

			if give_up {
				return Err(Error::NoDotMinimap);
			}
		}
	};

	let minimap_file_contents = std::fs::read_to_string(&minimap_file)?;
	let minimap_file: DotMinimap =
		toml::from_str(&minimap_file_contents).map_err(|err| Error::Toml(err, minimap_file))?;

	let git_remote = GitRemote::open(&minimap_file.remote)?;
	let workspace = Workspace::open(git_remote);
	Ok(workspace)
}

fn cmd_workspace(arg0: Option<&str>, args: &[String]) -> Result<i32> {
	let subcommand = args.iter().next();

	match subcommand.as_ref().map(|s| s.as_str()) {
		Some("name") => cmd_workspace_name(arg0, &args[1..]),
		Some("description") => cmd_workspace_description(arg0, &args[1..]),
		Some("--help") | None => {
			eprintln!(
				concat!(
					"usage: {arg0} workspace <command> [<args>]\n",
					"\n",
					"Minimap workspace metadata commands.\n",
					"\n",
					"Available commands:\n",
					"    name           Gets or sets the workspace name\n",
					"    description    Gets or sets the workspace description\n",
					"    --help         Prints this help message",
				),
				arg0 = arg0.unwrap_or("minimap")
			);
			Ok(2)
		}
		Some(unknown) if unknown.starts_with('-') => {
			eprintln!("error: unknown 'workspace' argument `{}`\n", unknown);
			Ok(2)
		}
		Some(unknown) => {
			eprintln!("error: unknown 'workspace' subcommand `{}`\n", unknown);
			Ok(2)
		}
	}
}

fn cmd_workspace_name(arg0: Option<&str>, args: &[String]) -> Result<i32> {
	let mut write_name = None;
	let mut verbose = false;
	let mut idempotent = true;

	for arg in args {
		match arg.as_str() {
			"--help" => {
				eprintln!(
					concat!(
						"usage: {arg0} workspace name [-vf] [<new_name>]\n",
						"\n",
						"Gets or sets the workspace name.\n",
						"\n",
						"Returns non-zero if the workspace name is not set and no\n",
						"new name is provided.\n",
						"\n",
						"Options:\n",
						"    -v, --verbose     Prints all record information along with the name\n",
						"    -f, --force       Perform a commit even if the last committed name\n",
						"                      is the same as the new name\n",
						"    --help            Prints this help message",
					),
					arg0 = arg0.unwrap_or("minimap")
				);
				return Ok(2);
			}
			"--verbose" | "-v" => {
				verbose = true;
			}
			"--force" | "-f" => {
				idempotent = false;
			}
			arg if arg.starts_with('-') => {
				eprintln!("error: unknown argument `{}`\n", arg);
				return Ok(2);
			}
			name => {
				if write_name.is_some() {
					eprintln!("error: too many arguments\nusage: minimap workspace name --help");
					return Ok(2);
				}

				write_name = Some(name);
			}
		}
	}

	let workspace = open_workspace()?;

	if let Some(name) = write_name {
		let record = if idempotent {
			if let Some(record) = workspace.name()?
				&& record.message() == name
			{
				record
			} else {
				workspace.set_name(name)?
			}
		} else {
			workspace.set_name(name)?
		};

		if verbose {
			print_record(&record, true);
		}

		Ok(0)
	} else {
		if let Some(record) = workspace.name()? {
			print_record(&record, verbose);
			Ok(0)
		} else {
			Ok(1)
		}
	}
}

fn cmd_workspace_description(arg0: Option<&str>, args: &[String]) -> Result<i32> {
	let mut write_description = None;
	let mut verbose = false;
	let mut idempotent = true;

	for arg in args {
		match arg.as_str() {
			"--help" => {
				eprintln!(
					concat!(
						"usage: {arg0} workspace description [-vf] [<new_description>]\n",
						"\n",
						"Gets or sets the workspace description.\n",
						"\n",
						"Returns non-zero if the workspace description is not set and no\n",
						"new description is provided.\n",
						"\n",
						"Options:\n",
						"    -v, --verbose     Prints all record information along with the description\n",
						"    -f, --force       Perform a commit even if the last committed description\n",
						"                      is the same as the new description\n",
						"    --help            Prints this help message",
					),
					arg0 = arg0.unwrap_or("minimap")
				);
				return Ok(2);
			}
			"--verbose" | "-v" => {
				verbose = true;
			}
			"--force" | "-f" => {
				idempotent = false;
			}
			arg if arg.starts_with('-') => {
				eprintln!("error: unknown argument `{}`\n", arg);
				return Ok(2);
			}
			description => {
				if write_description.is_some() {
					eprintln!(
						"error: too many arguments\nusage: minimap workspace description --help"
					);
					return Ok(2);
				}

				write_description = Some(description);
			}
		}
	}

	let workspace = open_workspace()?;

	if let Some(description) = write_description {
		let record = if idempotent {
			if let Some(record) = workspace.description()?
				&& record.message() == description
			{
				record
			} else {
				workspace.set_description(description)?
			}
		} else {
			workspace.set_description(description)?
		};

		if verbose {
			print_record(&record, true);
		}

		Ok(0)
	} else {
		if let Some(record) = workspace.description()? {
			print_record(&record, verbose);
			Ok(0)
		} else {
			Ok(1)
		}
	}
}

fn cmd_project(arg0: Option<&str>, args: &[String]) -> Result<i32> {
	let subcommand = args.iter().next();

	match subcommand.as_ref().map(|s| s.as_str()) {
		Some("create") => cmd_project_create(arg0, &args[1..]),
		Some("--help") | None => {
			eprintln!(
				concat!(
					"usage: {arg0} project <command> [<args>]\n",
					"\n",
					"Minimap project commands.\n",
					"\n",
					"Available commands:\n",
					"    create    Creates a new project\n",
					"    --help    Prints this help message",
				),
				arg0 = arg0.unwrap_or("minimap")
			);
			Ok(2)
		}
		Some(unknown) if unknown.starts_with('-') => {
			eprintln!("error: unknown 'project' argument `{}`\n", unknown);
			Ok(2)
		}
		Some(unknown) => {
			eprintln!("error: unknown 'project' subcommand `{}`\n", unknown);
			Ok(2)
		}
	}
}

fn cmd_project_create(arg0: Option<&str>, args: &[String]) -> Result<i32> {
	let mut args = args.into_iter();
	let mut verbose = false; // -v or --verbose
	let mut allow_existing = false; // -e or --allow-existing
	let mut allow_lowercase = false; // -l or --allow-lowercase
	let mut slug = None; // one and only positional
	let mut description = None; // -d or --description <description>
	let mut name = None; // -n or --name <name>

	while let Some(arg) = args.next() {
		match arg.as_str() {
			"--help" => {
				eprintln!(
					concat!(
						"usage: {arg0} project create [-vel] [-d <description>] [-n <name>] <slug>\n",
						"\n",
						"Creates a new project.\n",
						"\n",
						"Options:\n",
						"    -v, --verbose         Prints all record information along with the project\n",
						"    -e, --allow-existing  Allow the project to be created if it already exists\n",
						"    -l, --allow-lowercase Allow the project slug to contain lowercase letters\n",
						"    -d, --description     Sets the project description\n",
						"    -n, --name            Sets the project name\n",
						"    --help                Prints this help message",
					),
					arg0 = arg0.unwrap_or("minimap")
				);
				return Ok(2);
			}
			"--verbose" | "-v" => {
				verbose = true;
			}
			"--allow-existing" | "-e" => {
				allow_existing = true;
			}
			"--allow-lowercase" | "-l" => {
				allow_lowercase = true;
			}
			"--description" | "-d" => {
				if description.is_some() {
					eprintln!(
						"error: `--description` may only be specified once\nusage: minimap project create --help"
					);
					return Ok(2);
				}

				if let Some(arg) = args.next()
					&& !arg.starts_with('-')
				{
					description = Some(arg.to_string());
				} else {
					eprintln!(
						"error: missing argument to `--description`\nusage: minimap project create --help"
					);
					return Ok(2);
				}
			}
			"--name" | "-n" => {
				if name.is_some() {
					eprintln!(
						"error: `--name` may only be specified once\nusage: minimap project create --help"
					);
					return Ok(2);
				}

				if let Some(arg) = args.next()
					&& !arg.starts_with('-')
				{
					name = Some(arg.to_string());
				} else {
					eprintln!(
						"error: missing argument to `--name`\nusage: minimap project create --help"
					);
					return Ok(2);
				}
			}
			arg if arg.starts_with('-') => {
				eprintln!("error: unknown argument `{}`\n", arg);
				return Ok(2);
			}
			arg => {
				if slug.is_some() {
					eprintln!("error: too many arguments\nusage: minimap project create --help");
					return Ok(2);
				}

				slug = Some(arg.to_string());
			}
		}
	}

	let slug = match slug {
		Some(slug) => slug,
		None => {
			eprintln!("error: missing argument `slug`\nusage: minimap project create --help");
			return Ok(2);
		}
	};

	if !allow_lowercase && slug.chars().any(|c| c.is_ascii_lowercase()) {
		eprintln!(
			"error: slug contains lowercase letters: {slug}\nusage: minimap project create --help"
		);
		return Ok(2);
	}

	if slug.chars().any(|c| !c.is_ascii_alphanumeric()) {
		eprintln!(
			"error: slug contains invalid characters: {slug}\nusage: minimap project create --help"
		);
		return Ok(2);
	}

	let workspace = open_workspace()?;
	let project = workspace.create_project(&slug)?;

	match project {
		Ok(project) => {
			if verbose {
				print_record(&project.record()?.unwrap(), true);
			}

			Ok(0)
		}
		Err(old_record) if allow_existing => {
			if verbose {
				print_record(&old_record, true);
			}

			Ok(0)
		}
		Err(old_record) => {
			eprintln!("error: project already exists: {slug}", slug = slug);

			if verbose {
				eprintln!("\n");
				print_record(&old_record, true);
			}

			Ok(1)
		}
	}
}

fn print_record<R: Record>(record: &R, verbose: bool) {
	if verbose {
		println!("id:     {}", record.id());
		println!("author: {}", record.author());
		println!("email:  {}", record.email());
		println!("date:   {}", timestamp_to_iso8601(record.timestamp()));
		println!("\n{}", record.message());
	} else {
		println!("{}", record.message());
	}
}

fn timestamp_to_iso8601(timestamp: i64) -> String {
	let naive_datetime = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
	let datetime: chrono::DateTime<chrono::Utc> =
		chrono::DateTime::from_naive_utc_and_offset(naive_datetime, chrono::Utc);
	datetime.to_rfc3339()
}
