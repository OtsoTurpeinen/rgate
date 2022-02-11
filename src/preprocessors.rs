use std::{path::Path, fs::File, collections::HashMap};

use subprocess::{Exec, Redirection};



pub struct Preprocessor {
    extension: String,
    command: String,
    method: String,
    argpass: String,
    priority: u16,
    strict_error: u8,
}

#[derive(PartialEq, Debug)]
pub enum PreprocessorErr {
    NoProcessor,
    ProcessorFailed,
    FileError,
}

impl Preprocessor {
    pub fn new(extension: String, command: String, method: String, argpass: String, priority: u16, strict_error: u8) -> Preprocessor {
        Preprocessor {extension,command,method,priority,argpass, strict_error}
    }

    pub fn process(&self, file_path: String, _args: &HashMap<String,String> ) -> Result<String,PreprocessorErr> {
        let file = File::open(&file_path);
        let file_handle;
        match file {
            Ok(handle) =>  file_handle = handle,
            Err(r) => {
                println!("{}",r);
                return Err(PreprocessorErr::FileError)
            }
        }

        let args_io;
        match self.argpass.as_str() {
            "= " => args_io = self.arg_parse(_args),
            _ => args_io = "".to_string(),
        }

        let out;
        match self.method.as_str() {
            "file" => out = Exec::shell(format!("{} {} {}",&self.command,&file_path,args_io)).stdin(args_io.as_str()).capture(),
            "pipe" => out = Exec::shell(&self.command).stdin(Redirection::File(file_handle)).capture(),
            _ => out = Exec::shell(&self.command).stdin(Redirection::File(file_handle)).stdin(args_io.as_str()).capture(),
        }
        match out {
            Ok(x) => {
                println!("{:?}",x);
                if &x.stderr.len() > &0 {
                    println!("{}", String::from_utf8_lossy(&x.stderr).to_string());
                    if self.strict_error == 1 {
                        println!("strict 500");
                        return Err(PreprocessorErr::ProcessorFailed)
                    }
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

    fn arg_parse(&self, _args: &HashMap<String,String>) -> String {
        let mut r: String = "".to_string();
        let mut fix = self.argpass.chars();
        let a = fix.next().unwrap();
        let s = fix.next().unwrap();
        for (key,value) in _args.iter() {
            r = format!("{r}{key}{a}{value}{s}");
        }
        r
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

    pub fn add(&mut self, extension: String, command: String, method: String, argpass: String, priority: u16, strict_error: u8) {
        let processor = Preprocessor::new(extension, command, method,argpass, priority, strict_error);
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

    pub fn process(&self, file_path: String, args: &HashMap<String,String> ) -> Result<String,PreprocessorErr>{
        for proc in self.processors.iter() {
            if proc.does_apply(file_path.clone()) {
                return proc.process(file_path,args);
            }
        }
        return Err(PreprocessorErr::NoProcessor);
    }
}