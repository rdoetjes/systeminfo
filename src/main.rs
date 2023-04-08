use std::{thread, time::{self}, sync::{Arc, Mutex}};

use rocket::{get, State, routes, serde::Serialize};
use ::sysinfo::{System, SystemExt, CpuExt};

#[derive(Default, Debug)]
struct SharedData{
    system_info: Arc<std::sync::Mutex<SystemInfo>>,
}

impl SharedData{
    fn new(s:  Arc<std::sync::Mutex<SystemInfo>>) -> SharedData{
        SharedData{
            system_info: s,
        }
    }
}

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

#[get("/sysinfo")]
async fn sysinfo(state: &State<SharedData>) -> String {
    let result;

    let details = &state.system_info.lock().unwrap().to_owned();
    result = rocket::serde::json::to_string(&details).expect("{}}").to_string();
    
    drop(details);
    result
}

fn get_sys_info(info: &mut Arc<std::sync::Mutex<SystemInfo>>){
    let mut sys = System::new_all();

    loop{
        sys.refresh_all();
        let mut details = info.lock().unwrap();
    
        details.tot_memory = sys.total_memory();
        details.used_memory = sys.used_memory();
        details.tot_swap = sys.total_swap();
        details.used_swap = sys.used_swap();

        details.cpu_util.clear();
        for cpu in sys.cpus() {
           details.cpu_util.push(cpu.cpu_usage()); 
        }
      
        drop(details);
        
        thread::sleep(time::Duration::from_millis(1000));
    }
}


#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let detail = Arc::new(Mutex::new(SystemInfo::default()));

    //thread that gathers all system data and puts it in Arc Mutex called detail
    let mut s = detail.clone();
    thread::spawn(move || {
        get_sys_info(&mut s);
    });


   let shared_data = SharedData::new(detail.clone());

    const ROOTV1: &str = "/api/v1";
    let rocket = rocket::build()
                    .mount(ROOTV1, routes![sysinfo])
                    .manage(shared_data)
                    .ignite().await?;

    let rocket = rocket.launch().await?;

    Ok(())
}
