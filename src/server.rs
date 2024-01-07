use std::{
    fs,
    fmt,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::threads::ThreadPool;

#[derive(Clone, Copy, PartialEq)]
enum ItemType {
    TextFile = 0,
    Submenu = 1
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ItemType::TextFile => write!(f, "TextFile"),
            ItemType::Submenu => write!(f, "Submenu"),
        }
    }
}

#[derive(Clone)]
pub struct ServerConfig {
    pub addr: String,
    pub port: i32,
    pub root: String
}

impl ServerConfig {
    pub fn new(addr: &str, port: i32, root: &str) -> ServerConfig {
        let addr = String::from(addr);
        let root = String::from(root);
        ServerConfig { addr, port, root }
    }

    pub fn full_address(&self) -> String {
        return format!("{}:{}", self.addr, self.port);
    }
}

struct MenuItem {
    pub item_type: ItemType,
    pub display_name: String,
    pub selector: String,
    pub addr: String,
    pub port: i32

}

impl MenuItem {
    pub fn new(item_type: ItemType, display_name: &str, selector: &str, addr: &str, port: i32) -> MenuItem {
        let display_name = String::from(display_name);
        let selector = String::from(selector);
        let addr = String::from(addr);
        MenuItem { item_type, display_name, selector, addr, port }
    }

    pub fn to_response_string(&self) -> String {
        return format!("{}{}	{}	{}	{}", self.item_type as i32, self.display_name, self.selector, self.addr, self.port);
    }
}

struct ConnectionHandler {
    cfg: ServerConfig
}

impl ConnectionHandler {
    fn new(cfg: ServerConfig) -> ConnectionHandler {
        ConnectionHandler { cfg }
    }

    // fn build_menu(&self, path: String) -> String {
    //     let full_path = format!("{0}{1}", self.cfg.root, path);
    //     let gophermap_path = format!("{0}/gophermap", full_path);
    
    //     let file_contents = fs::read_to_string(gophermap_path);
    //     if file_contents.is_ok() {
    //         return file_contents.unwrap();
    //     } else {
    //         return self.output_menu(self.build_menu_items(path));
    //     }
    // }

    fn output_menu(&self, items: Vec<MenuItem>) -> String {
        let mut menu = String::new();
    
        for item in items {
            menu.push_str(&item.to_response_string());
            menu.push_str("\r\n");
        }
        
        return menu;
    }
    
    fn get_item_type(&self, path: &str) -> Option<ItemType> {
        let md = fs::metadata(path).unwrap();
        
        match md.file_type().is_dir() {
            true => Some(ItemType::Submenu),
            _ => Some(ItemType::TextFile)
        }
    }

    fn build_menu_items(&self, path: String) -> Vec<MenuItem> {
        let mut items = vec![];
    
        for file in fs::read_dir(format!("{}/{}", self.cfg.root, path)).unwrap() {
            let f = file.unwrap();
            let name = &f.file_name().into_string().unwrap();
            let item_type = self.get_item_type(&format!("{}", f.path().to_str().unwrap().to_string()));

            if let Some(t) = item_type {
                let mut item = MenuItem::new(t, name, &format!("{}{}", path, name), &self.cfg.addr, self.cfg.port);
                if t == ItemType::Submenu {
                    item.selector = item.selector + "/";
                }
                items.push(item);
            }
        }

        return items;
    }
    
    fn handle_connection(&self, mut stream: TcpStream) {
    
        let buf_reader = BufReader::new(&mut stream);
        let request_line = buf_reader.lines().next().unwrap().unwrap();
    
        let request = &request_line[..];
        let item_local_path = format!("{}/{}", self.cfg.root, request);
        let item_type = self.get_item_type(&item_local_path);
        println!("Request:'{request}'");
    
        let response = match item_type {
            Some(ItemType::Submenu) => self.output_menu(self.build_menu_items(String::from(request))),
            Some(ItemType::TextFile) => fs::read_to_string(&item_local_path).unwrap(),
            _ => self.output_menu(self.build_menu_items(String::from("")))
        };

        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(String::from("\r\n.\r\n").as_bytes()).unwrap();
    }
}

pub fn run(cfg: ServerConfig) {
    let listener = TcpListener::bind(cfg.full_address()).unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let cfg = cfg.clone();
        let stream = stream.unwrap();
        pool.execute(move || {
            let handler = ConnectionHandler::new(cfg);
            handler.handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

