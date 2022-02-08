use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use config::Source;
use config::Value;
use rgate::ThreadPool;
use rgate::preprocessors::PreprocessorErr;
use rgate::preprocessors::PreprocessorList;

fn main() {
    let mut server_settings = config::Config::default();
    server_settings.merge(config::File::with_name("Config")).unwrap();
    let server_port = server_settings.get_str("Server.port").unwrap_or("8080".to_string());
    let server_ip = server_settings.get_str("Server.ip").unwrap_or("127.0.0.1".to_string());
    let server_threads: usize = server_settings.get_int("Server.threads").unwrap_or(1).try_into().unwrap_or(1);
    let listener = TcpListener::bind(format!("{server_ip}:{server_port}")).unwrap();
    println!("listening on {server_ip}:{server_port}");
    
    let mut preproc_setttings = config::Config::default();
    preproc_setttings.merge(config::File::with_name("Preproc")).unwrap();
    let preproc_setttings_keys = preproc_setttings.collect().expect("invalid config");
    let mut interpeters = PreprocessorList::new();
    for (key,_) in preproc_setttings_keys.iter() {
        let element = preproc_setttings.get_table(key);
        match element {
            Ok(dat) => {
                let ext = dat.get("extension").unwrap().clone().into_str().unwrap();
                let cmd = dat.get("command").unwrap_or(&Value::from("cat".to_string())).clone().into_str().unwrap_or("cat".to_string());
                let typ = dat.get("input_type").unwrap_or(&Value::from("pipe".to_string())).clone().into_str().unwrap_or("pipe".to_string());
                let prio: u16 = preproc_setttings.get_int(format!("{}.priority",key).as_str()).unwrap_or(-1).try_into().unwrap_or(u16::MAX);
                println!("added preprocessor for {ext} using \"{cmd}\" using {typ}, priority {prio}");
                interpeters.add(ext,cmd,typ, prio);
            },
            _ => println!("funky error"),
        }
    }
    let final_interpeters = Arc::new(interpeters);
    let pool = ThreadPool::new(server_threads);
    println!("initiated {server_threads} thread(s) for processing pool");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let shared_interp = Arc::clone(&final_interpeters);
        pool.execute(|| {
            handle_connection(stream,shared_interp);
        });
    }
}

fn handle_connection(mut stream: TcpStream, interpeters: Arc<PreprocessorList>) {
    let mut buffer = [0;1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET /";
    let (mut status_line,filename) = if buffer.starts_with(get) {
        handle_get_request(buffer,&interpeters)
    } else {
        ("HTTP/1.1 404 NOT FOUND".to_string(),"errors/404.html".to_string())
    };
    let contents = interpeters.process(filename.clone());
    let final_content: String;
    match contents {
        Ok(data) => {
            final_content = data;
        }
        Err(PreprocessorErr::NoProcessor) => {
            final_content = fs::read_to_string(filename.to_string()).unwrap();
            status_line = "HTTP/1.1 200 OK".to_string();
        },
        Err(PreprocessorErr::ProcessorFailed) => {
            final_content = fs::read_to_string("errors/500.html".to_string()).unwrap();
            status_line = "HTTP/1.1 500 INTERNAL SERVER ERROR".to_string();
        },
        Err(PreprocessorErr::FileError) => {
            final_content = fs::read_to_string("errors/500.html".to_string()).unwrap();
            status_line = "HTTP/1.1 500 INTERNAL SERVER ERROR".to_string();
        },
    }

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        final_content.len(),
        final_content
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_get_request(buffer: [u8; 1024], interpeters: &Arc<PreprocessorList>) -> (String,String) {
    let mut request_arr = buffer.split(|c| c == &b" "[0]);
    request_arr.next();
    let path: String;
    match request_arr.next() {
        Some(x) => {
            if x == b"/" {
                path = "/index".to_string();
            } else {
                path = String::from_utf8_lossy(x).to_string()
            }
        },
        None => return ("HTTP/1.1 404 NOT FOUND".to_string(),"errors/404.html".to_string())
    }
    let file_path = format!("public{}",path);
    let file_path_htm = format!("public{}.htm",path);
    let file_path_html = format!("public{}.html",path);
    let interped_file;
    match interpeters.check_file(&file_path) {
        Some(dat) => interped_file = dat.to_string(),
        _ => interped_file = "n/a".to_string(),
    }
    if path == "/".to_string() {
        println!("200: {}", path);
        ("HTTP/1.1 200 OK".to_string(),"public/index.html".to_string())
    } else if path.contains("..") {
        println!("403: {}", path);
        ("HTTP/1.1 403 FORBIDDEN".to_string(),"errors/403.html".to_string())
    } else if Path::new(&file_path).is_file() {
        println!("200: {}", path);
        ("HTTP/1.1 200 OK".to_string(),file_path)
    } else if interped_file != "n/a".to_string() {
        println!("200: {}", path);
        ("HTTP/1.1 200 OK".to_string(),format!("{}",interped_file))
    } else if Path::new(&file_path_htm).is_file() {
        println!("200: {}", path);
        ("HTTP/1.1 200 OK".to_string(),file_path_htm)
    } else if Path::new(&file_path_html).is_file() {
        println!("200: {}", path);
        ("HTTP/1.1 200 OK".to_string(),file_path_html)
    } else if path.contains("/coffee"){
        println!("418: {}", path);
        ("HTTP/1.1 418 IM A TEAPOT".to_string(),"errors/418.html".to_string())
    } else {
        println!("404: {}", path);
        ("HTTP/1.1 404 NOT FOUND".to_string(),"errors/404.html".to_string())
    }
}