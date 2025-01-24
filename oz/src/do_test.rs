use {
    tokio::{
        task::JoinHandle, sync::Semaphore, process::Command,
    },
    std::{
        fs::{self, File,}, io::Write, path::Path, sync::Arc,
    },
    crate::RESULTS,
};

pub async fn spawn(
    tasks: usize,
    files: Vec<String>,
    mut private_keys: Vec<Vec<String>>,
) -> Vec<JoinHandle<()>> {
    let semaphore = Arc::new(Semaphore::new(tasks));

    let mut jh = vec![];

    for file in files {
        let semaphore = Arc::clone(&semaphore);
        let permit = semaphore.acquire_owned().await.unwrap();
        let task_keys = private_keys.pop().unwrap();

        let handle = tokio::spawn(async move {
            do_test(file, task_keys).await;
            drop(permit);
        });

        jh.push(handle);
    }
    jh
}

async fn do_test(file: String, keys: Vec<String>) {

    let keys_list = keys.join(",");
    let mut cmd = Command::new("sh");
    cmd.arg("-c");
    let shell = "cd /opt/oz/openzeppelin-contracts && npx hardhat test ".to_string() + &file;
    cmd.arg(&shell);
    cmd.env("PRIVATE_KEYS", keys_list);

    let start = tokio::time::Instant::now();
    let output = cmd.output().await.unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let time = start.elapsed().as_secs();
    let mut time_msg = format!("{} s", time);

    let time = time / 60;
    
    if time > 15 {
        time_msg += " (15m+)"
    } else if time > 10 {
        time_msg += " (10m+)"
    } else if time > 5 {
        time_msg += " (5m+)"
    }

    println!("test: {}", &file);
    println!("stdout: {}", &stdout);
    println!("stderr: {}", &stderr);
    println!("{}",output.status);
    println!("time: {}\n", time_msg);

    let name = file.replace(".", "_").replace("/", "_");
    let dir = Path::new(RESULTS).join(name);
    fs::create_dir(&dir).unwrap();

    let mut file = File::create(dir.join("stdout.log")).unwrap();
    file.write_all(stdout.as_bytes()).unwrap();

    let mut file = File::create(dir.join("stderr.log")).unwrap();
    file.write_all(stderr.as_bytes()).unwrap();

    let mut file = File::create(dir.join("time.log")).unwrap();
    file.write_all(time_msg.as_bytes()).unwrap();
}




