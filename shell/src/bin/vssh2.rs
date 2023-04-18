//Used https://doc.rust-lang.org/std/env/fn.current_dir.html, https://doc.rust-lang.org/std/env/fn.set_current_dir.html
// https://github.com/gjf2a/shell/blob/master/src/bin/typing_demo.rs use this with split white space

use std::{env, process::exit, path::Path, ffi::CString, io::Write};
use nix::{sys::wait::waitpid,unistd::{fork, ForkResult, execvp}};

fn main() -> anyhow::Result<()> {
    loop {
        let path = env::current_dir()?;
        println!("{}", path.display());
        match process_line() {
            Ok(user_input) => {
                let words:Vec<&str> = user_input.split_whitespace().collect();
                    if words[0] == "exit" {
                        return Ok(());
                    } else if words[0] == "cd" {
                        let root = Path::new(words[1]);
                        assert!(env::set_current_dir(&root).is_ok());
                    } else {
                        match unsafe{fork()} {
                            Ok(ForkResult::Parent { child, .. }) => {
                                waitpid(child, None).unwrap();
                            }
                            Ok(ForkResult::Child) => {
                                let cmd = externalize(user_input.as_str());
                                println!("{cmd:?}");
                                match execvp::<CString>(cmd[0].as_c_str(), &cmd) {
                                    Ok(_) => {println!("Child finished");},
                                    Err(e) => {
                                        println!("Could not execute: {e}");
                                        exit(1);
                                    },
                                }
                            }
                            Err(_) => println!("Fork failed"),
                        }
                    }
                }
            Err(e) => {
                println!("Error: {e}");
            }
        }
    }
}

fn externalize(command: &str) -> Vec<CString> {
    command.split_whitespace()
        .map(|s| CString::new(s).unwrap())
        .collect()
}

fn process_line() -> anyhow::Result<String> {
    let mut user_input = String::new();
    print!("$ ");
    std::io::stdout().flush()?; 
    std::io::stdin().read_line(&mut user_input)?;
    Ok(user_input)
}