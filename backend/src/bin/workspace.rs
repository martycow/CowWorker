use cowworker_core::{backup, Store};
use std::path::Path;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result=match args.get(1).map(String::as_str){
  Some("verify") if args.len()==3=>backup::verify(Path::new(&args[2])),
  Some("restore") if args.len()==4=>backup::restore(Path::new(&args[2]),Path::new(&args[3])),
  Some("backup") if args.len()==4=>Store::open(&args[2]).and_then(|s|s.backup(Path::new(&args[3]))),
  _=>Err("Usage: workspace backup <workspace> <new-directory> | verify <backup> | restore <backup> <new-directory>".into())
 };
    match result {
        Ok(manifest) => println!(
            "{}",
            serde_json::to_string_pretty(&manifest).expect("manifest")
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
