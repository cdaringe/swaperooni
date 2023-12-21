use std::{process::Command, thread, time::Duration};

pub fn main() -> () {
    let mut cmd_base_0 = Command::new("bash");
    let script_path = "./swap/examples/swap_on_file_change/worker-app.sh";
    let mut count = 0;

    let start_cmd = cmd_base_0.args(vec![script_path, &count.to_string()]);
    let mut swap = swaperooni::SwapBuilder::start(start_cmd).unwrap();
    loop {
        println!("@info: sleeping for 5 seconds");
        thread::sleep(Duration::from_secs(5));
        count += 1;
        let mut cmd_base_i = Command::new("bash");
        let ith_cmd = cmd_base_i.args(vec![script_path, &count.to_string()]);
        println!("@info: swapping process");
        swap.swap(ith_cmd).unwrap();
    }
}
