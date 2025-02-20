use ipipe::Pipe;
use std::{i32, io::{Read, Write}, path::PathBuf, thread, time::Duration, vec};
mod common_utils;
use common_utils::*;
use std::env;
use std::fs::File;
use uuid::Uuid;

struct System
{
    EntityId: Uuid,
    IsActive: bool
}

fn main()
{
    println!("config service started");
    let args:Vec<_> = env::args().collect();
    //виндовая реализация, для линухов, разумеется, будет иначе
    let mut path_to_platform:PathBuf = PathBuf::new();
    if args.len() > 1 && args[1] != ""
    {
        let dev_json = load_json_from_file(&args[1]);
        let path = dev_json.get("developerDataFolder").unwrap().as_str().unwrap();
        path_to_platform.push(path);
    }
    else {
        path_to_platform.push("C:\\ProgramData\\Datareon\\Platform");
    }
    let mut path_to_local_node_config = path_to_platform.clone();
    path_to_local_node_config.push("LocalNodeConfig.json");
    path_to_platform.push("Cache");
    //тут грузим конфиг
    path_to_platform.push("DatareonPlatformNode");
    path_to_platform.push("ClusterCache.json");


    let cluster_cache = load_json_from_file(path_to_platform.to_str().unwrap());
    let systems = cluster_cache.get("Systems").unwrap().as_array().unwrap();
    let mut guids:Vec<Uuid> = vec![];
    for i in systems
    {
        let next :serde_json::Value = serde_json::from_value(i.clone()).unwrap();
        if next.get("IsActive").unwrap() == true
        {
            let val = next.get("EntityId").unwrap().as_str().unwrap();
            guids.push(Uuid::parse_str(val).unwrap());
        }
    }
    let local_node_config = load_json_from_file(path_to_local_node_config.to_str().unwrap());
    let node_id = local_node_config.get("NodeId").unwrap().clone();
    

    //закончили - отправляем обратно инфо, что всё ок
    let t = thread::spawn(move ||answer_handler(&guids, &(Uuid::parse_str(node_id.as_str().unwrap()).unwrap())));
    t.join();
}

fn answer_handler(guids: &Vec<Uuid>, node: &Uuid)
{
    let mut to_config_pipe = Pipe::with_name("to_config_pipe").unwrap();
    let mut from_config_pipe: Pipe = Pipe::with_name("from_config_pipe").unwrap();
    send_load_done(&mut from_config_pipe);
    let mut next_command:[u8;4] =[0;4];
    loop {
        let mut buf = read_to_end(&mut to_config_pipe);
        if buf.len()>0
        {
            next_command = [buf[0], buf[1], buf[2], buf[3]];
            let mut command = i32::from_le_bytes(next_command);
            match Commands::get_command(command)
            {
                Commands::GetAdaptersConfig => {println!("отправляем конфиг");
                                            send_services_to_start(&mut from_config_pipe, &guids);},
                Commands::GetNodeConfig =>send_node_to_start(&mut from_config_pipe, &node),
                _=> do_nothing(),
            }
            
        }
    }
}

fn do_nothing()
{
    println!("сидим, пердим");
}

fn send_load_done(pipe: &mut Pipe)
{
    let command:i32= Commands::LoadDone as i32;
    let size: i64 = 192834;
    let mut datagram:Vec<u8>= vec![];
    push_bytes_i32(&mut datagram, command);
    push_bytes_i64(&mut datagram, size);
    println!("Пишем обратку");
    pipe.write(&datagram).unwrap();
}

 fn send_services_to_start(pipe: &mut Pipe, guids: &Vec<Uuid>)
 {
    let command:i32= Commands::AdaptersStart as i32;
    let size = guids.len()*16;
    let mut datagram:Vec<u8>= vec![];
    push_bytes_i32(&mut datagram, command);
    push_bytes_usize(&mut datagram, size);
    push_guids(&mut datagram, guids);
    pipe.write(&datagram).unwrap();
 }

 fn send_node_to_start(pipe: &mut Pipe, node: &Uuid)
 {
    let command:i32= Commands::NodeStart as i32;
    let size = 16;
    let mut datagram:Vec<u8>= vec![];
    push_bytes_i32(&mut datagram, command);
    push_bytes_usize(&mut datagram, size);
    push_guid(&mut datagram, node);
    pipe.write(&datagram).unwrap();
 }

 pub fn push_bytes_i32(vector: &mut Vec<u8>, val: i32)
 {
    for i in val.to_le_bytes()
    {
        vector.push(i);
    }
 
}


pub fn push_bytes_i64(vector: &mut Vec<u8>, val: i64)
{
   for i in val.to_le_bytes()
   {
       vector.push(i);
   }

}

pub fn push_bytes_usize(vector: &mut Vec<u8>, val: usize)
{
   for i in val.to_le_bytes()
   {
       vector.push(i);
   }

}

fn load_json_from_file(path: &str)->serde_json::Value
{
    let mut result:serde_json::Value;
    let file = File::open(&path);
        let mut buffer = String::new();
        let mut opened_file = match file
        {
            Ok(file)=> file,
            Err(e)=>{
                println!("Не удалось открыть файл по пути {:?}, {}",path, e);
                return serde_json::from_str("{}").unwrap();
            }
        };
    let mut json_str = String::new();
    opened_file.read_to_string(&mut json_str);
    result = serde_json::from_str(&json_str).unwrap();
    result
}

fn push_guids(vector: &mut Vec<u8>, guids: &Vec<Uuid>)
{
    for item in guids
    {
        for byte in item.to_bytes_le()
        {
        vector.push(byte);
        }
    }
}

fn push_guid(vector: &mut Vec<u8>,guid: &Uuid)
{
    for byte in guid.to_bytes_le()
        {
        vector.push(byte);
        }
}


fn read_to_end(pipe: &mut Pipe)->Vec<u8>
{
    
    let mut buf:[u8;8]= [0;8];
    let mut result:Vec<u8>=vec![];
    let mut read = false;
    'looping: loop{
        let size = pipe.read(&mut buf).unwrap();
        if size > 0
        {
            to_vec(size, &mut buf, &mut result);
            read = true;
        }
        if read && (size == 0 || size<8)
        {
            break 'looping;
        }
    }
    pipe.flush();
    result
}

fn to_vec(size: usize, buf: &mut[u8], dest: &mut Vec<u8>)
{
    for i in 0..size
    {
        dest.push(buf[i]);
    }
}