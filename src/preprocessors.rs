use std::{path::Path, fs::File};

use subprocess::{Exec, Redirection};



pub struct Preprocessor {
    extension: String,
    command: String,
    method: String,
    priority: u16,
}

#[derive(PartialEq, Debug)]
pub enum PreprocessorErr {
    NoProcessor,
    ProcessorFailed,
    FileError,
}

impl Preprocessor {
    pub fn new(extension: String, command: String, method: String, priority: u16) -> Preprocessor {
        Preprocessor {extension,command,method,priority}
    }

    pub fn process(&self, file_path: String) -> Result<String,PreprocessorErr> {
        let file = File::open(&file_path);
        let file_handle;
        match file {
            Ok(handle) =>  file_handle = handle,
            Err(r) => {
                println!("{}",r);
                return Err(PreprocessorErr::FileError)
            }
        }
        let out;
        match self.method.as_str() {
            "file" => out = Exec::shell(format!("{} {}",&self.command,&file_path)).capture(),
            "pipe" => out = Exec::shell(&self.command).stdin(Redirection::File(file_handle)).capture(),
            _ => out = Exec::shell(&self.command).stdin(Redirection::File(file_handle)).capture(),
        }
        match out {
            Ok(x) => {
                if &x.stderr.len() > &0 {
                    println!("{}", String::from_utf8_lossy(&x.stderr).to_string());
                }
                Ok(String::from_utf8_lossy(&x.stdout).to_string())
            },
            Err(r) => {
                println!("{}", r);
                Err(PreprocessorErr::ProcessorFailed)
            }
        }
    }

    pub fn does_apply(&self, file_path: String) -> bool {
        file_path.ends_with(&self.extension)
    }
}

pub struct PreprocessorList {
    processors: Vec<Preprocessor>,
}



impl PreprocessorList {
    pub fn new() -> PreprocessorList {
        let processors = Vec::new();
        PreprocessorList { processors }
    }

    pub fn add(&mut self, extension: String, command: String, method: String, priority: u16) {
        let processor = Preprocessor::new(extension, command, method, priority);
        self.processors.push(processor);
        self.processors.sort_by_key(|k| k.priority);
    }

    pub fn check_file(&self, file_path: &String) -> Option<String> {
        for proc in self.processors.iter() {
            let new_path = format!("{}{}",file_path,proc.extension);
            if Path::new(&new_path).is_file() {
                return Some(new_path);
            }
        }
        None
    }

    pub fn process(&self, file_path: String) -> Result<String,PreprocessorErr>{
        for proc in self.processors.iter() {
            if proc.does_apply(file_path.clone()) {
                return proc.process(file_path);
            }
        }
        return Err(PreprocessorErr::NoProcessor);
    }
}