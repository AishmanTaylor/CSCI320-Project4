//Used https://doc.rust-lang.org/std/env/fn.current_dir.html, https://doc.rust-lang.org/std/env/fn.set_current_dir.html
// https://github.com/gjf2a/shell/blob/master/src/bin/typing_demo.rs use this with split white space

use std::{env, process::exit, path::Path};
use nix::{sys::wait::waitpid,unistd::{fork, ForkResult, write}};

fn main() {
    let path = env::current_dir()?;
    let root = Path::new("/");
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    loop {
        println!("{}", path.to_str().unwrap());
        if arguments[0] == "exit" {
            return;
        } else if arguments[0] == "cd" {
            let root = Path::new(&("{}", arguments[1]));
            assert!(env::set_current_dir(&root).is_ok());
        } else {
            match unsafe{fork()} {
                Ok(ForkResult::Parent { child, .. }) => {
                    pub fn waitpid<P: Into<Option<Pid>>>(
                        pid: P,
                        options: Option<WaitPidFlag>
                    ) -> Result<WaitStatus>;
                }
                Ok(ForkResult::Child) => {
                    pub fn execvp<S: AsRef<CStr>>(filename: &CStr, args: &[S]) -> Result<Infallible>;
                }
                Err(_) => println!("Fork failed"),
             }
        }
        //FORK GET HELP
        //PANIC GET HELP
    }
}