use std::{thread, time};
use amqprs::{connection::{Connection, OpenConnectionArguments}, callbacks::{DefaultConnectionCallback, DefaultChannelCallback}, channel::{Channel, BasicPublishArguments}, BasicProperties};
use serde::Serialize;
use ::sysinfo::{System, SystemExt, CpuExt};
use serde_json;

#[derive(Default, Debug, Clone, Serialize)]
struct SystemInfo{
    tot_memory: u64,
    used_memory: u64,
    tot_swap: u64,
    used_swap: u64,
    cpu_util: Vec<f32>,
}

impl SystemInfo{
    fn default() -> SystemInfo{
        let mut sys = System::new_all();
        // First we update all information of our `System` struct.
        sys.refresh_all();
        SystemInfo {
            cpu_util: Vec::with_capacity(sys.cpus().len()),
            ..Default::default()
        }
    }
}

async fn connect_rabbitmq(host: &str, port: u16, username: &str, password: &str) -> Connection {
    //this is for demo and teaching purposes, you would fetch this information from a config of course
    let mut res = Connection::open(&OpenConnectionArguments::new(
        host,
        port,
        username,
        password,
    )).await;

    while res.is_err(){
        println!("trying to connect after error");
        std::thread::sleep(time::Duration::from_millis(2000));
        res = Connection::open(&OpenConnectionArguments::new(
            host,
            port,
            username,
            password,
        )).await;
    }

    let connection = res.unwrap();
    connection.register_callback(DefaultConnectionCallback).await.unwrap();
    connection
}

async fn channel_rabbitmq(connection: &amqprs::connection::Connection)-> Channel{
    let channel = connection.open_channel(None).await.unwrap();
    channel.register_callback(DefaultChannelCallback).await.unwrap();
    return  channel;
}

async fn get_sys_info(host: &str, port: u16, username: &str, password: &str){
    let mut sys = System::new_all();
    let mut details = SystemInfo::default();
   
    let mut connection = connect_rabbitmq(host, port, username, password).await;
    let mut channel = channel_rabbitmq(&connection).await;

    loop{
        sys.refresh_cpu();
        sys.refresh_memory();
        
        details.tot_memory = sys.total_memory();
        details.used_memory = sys.used_memory();
        details.tot_swap = sys.total_swap();
        details.used_swap = sys.used_swap();

        details.cpu_util.clear();
        for cpu in sys.cpus() {
            details.cpu_util.push(cpu.cpu_usage()); 
        }

        if !connection.is_open(){
            println!("Connection not open");
            connection = connect_rabbitmq(host, port, username, password).await;
            channel = channel_rabbitmq(&connection).await;
            println!("{}", connection);
        }

        if !channel.is_open() {
            println!("{}", channel.is_open());
            channel = channel_rabbitmq(&connection).await;
        }

        let args = BasicPublishArguments::new("systemmonitor", "");
        let result = serde_json::to_string(&details.to_owned()).expect("{}}").to_string();
        channel.basic_publish(BasicProperties::default(), result.into(), args).await.unwrap();

        thread::sleep(time::Duration::from_millis(1)); 
    }
    
}

#[tokio::main]
async fn main() {
    get_sys_info("localhost", 5672, "guest","herpies").await;
}