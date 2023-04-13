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

async fn connect_rabbitmq() -> Connection {
    //this is for demo and teaching purposes, you would fetch this information from a config of course
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    )).await.unwrap();

    connection.register_callback(DefaultConnectionCallback).await.unwrap();
    return connection;
}

async fn channel_rabbitmq(connection: amqprs::connection::Connection)-> Channel{
    let channel = connection.open_channel(None).await.unwrap();
    channel.register_callback(DefaultChannelCallback).await.unwrap();
    return  channel;
}

async fn get_sys_info(){
    let mut sys = System::new_all();
    let mut details = SystemInfo::default();

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

        let connection = connect_rabbitmq();
        let result = serde_json::to_string(&details.to_owned()).expect("{}}").to_string();
        let channel = channel_rabbitmq(connection.await);
        // create arguments for basic_publish
        let args = BasicPublishArguments::new("systemmonitor", "");
        channel.await.basic_publish(BasicProperties::default(), result.into(), args).await.unwrap();
        thread::sleep(time::Duration::from_millis(1000));
    }
    
}

#[tokio::main]
async fn main() {
    get_sys_info().await;
}