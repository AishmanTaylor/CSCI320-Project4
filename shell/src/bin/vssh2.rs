use std::{
    env, ffi::CString, fs::File, io::{Read, Write}, path::Path, process::exit,
};
use nix::{
    sys::wait::waitpid,
    unistd::{execvp, fork, ForkResult},
};

fn main() -> anyhow::Result<()> {
    loop {
        let path = env::current_dir()?;
        println!("{}", path.display());
        match process_line() {
            Ok(mut user_input) => {
                let background = user_input.trim().ends_with('&');
                if background {
                    user_input.truncate(user_input.trim().len() - 1);
                }
                let words: Vec<&str> = user_input.split_whitespace().collect();

                if words.is_empty() {
                    continue;
                }

                if words[0] == "exit" {
                    return Ok(());
                } else if words[0] == "cd" {
                    let root = Path::new(words[1]);
                    assert!(env::set_current_dir(&root).is_ok());
                } else {
                    let mut cmd = externalize(words[0]);
                    if words.len() > 1 && words[1] == "<" {
                        if words.len() == 2 {
                            println!("Error: no input file specified");
                            continue;
                        }
                        let input_file = Path::new(words[2]);
                        if !input_file.exists() {
                            println!("Error: input file does not exist");
                            continue;
                        }
                        let mut input = String::new();
                        File::open(input_file)?.read_to_string(&mut input)?;
                        cmd[0] = CString::new(input.trim())?;
                    }
                    match unsafe { fork() } {
                        Ok(ForkResult::Parent { child, .. }) => {
                            if background {
                                println!("Background process: {}", child);
                                continue;
                            }
                            waitpid(child, None).unwrap();
                        }
                        Ok(ForkResult::Child) => {
                            match execvp::<CString>(cmd[0].as_c_str(), &cmd) {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("Could not execute: {}", e);
                                    exit(1);
                                }
                            }
                        }
                        Err(_) => println!("Fork failed"),
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn externalize(command: &str) -> Vec<CString> {
    vec![CString::new(command).unwrap()]
}

fn process_line() -> anyhow::Result<String> {
    let mut user_input = String::new();
    print!("$ ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut user_input)?;
    Ok(user_input)
}
