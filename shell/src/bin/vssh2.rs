//Used https://doc.rust-lang.org/std/env/fn.current_dir.html, https://doc.rust-lang.org/std/env/fn.set_current_dir.html
// https://github.com/gjf2a/shell/blob/master/src/bin/typing_demo.rs use this with split white space

use std::{env, process::exit, path::Path, ffi::CString, io::Write};
use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult, execvp},
};
use anyhow::{Result, anyhow};

struct CommandLine {
    background: bool,
    output_file: Option<String>,
    input_file: Option<String>,
    commands: Vec<String>,
}

impl CommandLine {
    fn new(command_line: &str) -> CommandLine {
        let mut background = false;
        let mut output_file = None;
        let mut input_file = None;
        let mut commands = Vec::new();

        let parts: Vec<&str> = command_line.split('|').collect();
        if let Some(last) = parts.last() {
            if last.contains('>') {
                let last_parts: Vec<&str> = last.split('>').collect();
                if let Some(output) = last_parts.get(1) {
                    output_file = Some(output.trim().to_string());
                    commands.push(last_parts[0].trim().to_string());
                } else {
                    commands.push(last.trim().to_string());
                }
            } else {
                commands.push(last.trim().to_string());
            }
        }

        if let Some(first) = parts.first() {
            if first.contains('<') {
                let first_parts: Vec<&str> = first.split('<').collect();
                if let Some(input) = first_parts.get(1) {
                    input_file = Some(input.trim().to_string());
                    commands[0] = first_parts[0].trim().to_string();
                }
            } else {
                commands[0] = first.trim().to_string();
            }
        }

        if let Some(last_command) = commands.last_mut() {
            if last_command.ends_with('&') {
                background = true;
                last_command.pop();
            }
        }

        CommandLine {
            background,
            output_file,
            input_file,
            commands,
        }
    }
}

fn main() -> Result<()> {
    loop {
        let path = env::current_dir()?;
        println!("{}", path.display());
        match process_line() {
            Ok(mut user_input) => {
                let mut words: Vec<&str> = user_input.split_whitespace().collect();
                if words[0] == "exit" {
                    return Ok(());
                } else if words[0] == "cd" {
                    let root = Path::new(words[1]);
                    assert!(env::set_current_dir(&root).is_ok());
                } else {
                    let command_line = CommandLine::new(&user_input);
                    execute_command_line(&command_line)?;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn execute_command_line(command_line: &CommandLine) -> Result<()> {
    let mut input_fd = None;
    let mut output_fd = None;
    // Redirect input from file, if specified
    if let Some(input_file) = &command_line.input_file {
        input_fd = Some(nix::fcntl::open(input_file, nix::fcntl::O_RDONLY, nix::sys::stat::Mode::empty())?);
        nix::unistd::dup2(input_fd.unwrap(), nix::libc::STDIN_FILENO)?;
    }

    // Redirect output to file, if specified
    if let Some(output_file) = &command_line.output_file {
        output_fd = Some(nix::fcntl::open(output_file, nix::fcntl::O_WRONLY | nix::fcntl::O_CREAT | nix::fcntl::O_TRUNC, nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR | nix::sys::stat::Mode::S_IRGRP | nix::sys::stat::Mode::S_IROTH)?);
        nix::unistd::dup2(output_fd.unwrap(), nix::libc::STDOUT_FILENO)?;
    }

    // Execute commands in the pipeline
    let mut previous_output_fd = None;
    for command in &command_line.commands {
        let args: Vec<CString> = command.split_whitespace().map(|arg| CString::new(arg.to_string()).unwrap()).collect();
        let argv: Vec<*const i8> = args.iter().map(|arg| arg.as_ptr()).collect();
        let command_fd = match previous_output_fd {
            Some(fd) => fd,
            None => nix::libc::STDOUT_FILENO,
        };
        let result = unsafe {
            fork().expect("Failed to fork")
        };
        match result {
            ForkResult::Parent { child } => {
                if !command_line.background {
                    waitpid(child, None).expect("Failed to wait for child process");
                }
                previous_output_fd = None;
            }
            ForkResult::Child => {
                // Redirect output to the previous command's input, if not the last command
                if !command_line.background && !command.eq(&command_line.commands[command_line.commands.len() - 1]) {
                    let pipe_fd = nix::unistd::pipe().expect("Failed to create pipe");
                    nix::unistd::dup2(pipe_fd.1, nix::libc::STDOUT_FILENO).expect("Failed to redirect output to pipe");
                    previous_output_fd = Some(pipe_fd.0);
                }

                // Redirect input from previous command's output, if not the first command
                if !command.eq(&command_line.commands[0]) {
                    nix::unistd::dup2(command_fd, nix::libc::STDIN_FILENO).expect("Failed to redirect input from pipe");
                }

                // Execute command
                execvp(&args[0], &argv).expect("Failed to execute command");
            }
        }
    }

    // Close file descriptors
    if let Some(fd) = input_fd {
        nix::unistd::close(fd).expect("Failed to close input file descriptor");
    }
    if let Some(fd) = output_fd {
        nix::unistd::close(fd).expect("Failed to close output file descriptor");
    }

    Ok(())
}

fn process_line() -> Result<String> {
    let mut user_input = String::new();
    print!("$ ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut user_input)?;
    user_input.pop(); // Remove trailing newline
    Ok(user_input)
}
