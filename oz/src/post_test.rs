use {
    glob::glob,
    std::{
        fs::{self, File,}, io::{Read, Write}, path::Path, env,
    },
    crate::{RESULTS, ALLURE_RESULTS, ALLURE_ENV},
};

pub fn merge_time_logs() {
    let res_dir = Path::new(RESULTS);
    let times_path = res_dir.join("time.log");
    let mut times = File::create(&times_path).unwrap();

    for item in fs::read_dir(res_dir).unwrap() {
        let path = item.unwrap().path();
        if path.is_dir() {
            let log = path.join("time.log");
            if log.exists() {
                let mut file = File::open(log).unwrap();
                let mut time  = String::new();
                file.read_to_string(&mut time).unwrap();
                time += "\n";

                times.write_all(time.as_bytes()).unwrap();
            }
        }
    }

    let dst = Path::new(ALLURE_RESULTS).join("time_consolidated.log");
    fs::copy(times_path, dst).unwrap();
}

pub fn create_allure_env() {
    let path = Path::new(ALLURE_ENV);
    let mut file = File::create(path).unwrap();

    let rome_apps_tag = env::var("ROME_APPS_TAG").expect("expected ROME_APPS_TAG");
    let rome_evm_tag = env::var("ROME_EVM_TAG").expect("expected ROME_EVM_TAG:");
    let ref_name = env::var("REF_NAME").expect("expected REF_NAME:");

    file.write_all(format!("rome-apps tag: {}\n", rome_apps_tag).as_bytes()).unwrap();
    file.write_all(format!("rome-evm  tag: {}\n", rome_evm_tag).as_bytes()).unwrap();
    file.write_all(format!("tests branch: {}\n", ref_name).as_bytes()).unwrap();
}

pub fn fix_allure_results() {
    let files = glob(&format!("{}/*-result.json", ALLURE_RESULTS))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .iter()
        .map(|file| {
            format!("{}", file.as_os_str().to_str().unwrap())
        })
        .collect::<Vec<_>>();

    println!("Fix allure reports: {}", files.len());

    for file in files {
        let mut f = File::open(&file).unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();
        let mut json: serde_json::Value = serde_json::from_str(&content).unwrap();

        if let serde_json::Value::Object(obj) = &mut json {
            if let Some(serde_json::Value::Array(labels)) = obj.get_mut("labels") {
                let value = serde_json::json!({"name": "epic", "value": "OpenZeppelin contracts"});
                labels.push(value);
            }
        }

        fs::write(&file, serde_json::to_string(&json).unwrap()).unwrap();
    }
}

#[allow(dead_code)]
pub fn report() {
    let mut pass = 0_u64;
    let mut pend = 0_u64;
    let mut fail = 0_u64;
    let mut skip = vec![];

    let files = glob(&format!("{}/**/stdout.log", RESULTS))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    println!("`stdout` files found: {}. Processing .. ", files.len());
    for file in &files {
        let mut f = File::open(file).unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();

        let mut found = false;
        if content.contains("passing") {
            pass += 1;
            found = true;
        }
        if content.contains("pending") {
            pend += 1;
            found = true;
        }
        if content.contains("failing") {
            fail += 1;
            found = true;
        }
        if !found {
            skip.push(file.as_os_str().to_str().unwrap());
        }
    }

    println!("Summarize result");
    println!("  Passing - {}", pass);
    println!("  Pending - {}", pend);
    println!("  Failing - {}", fail);
    println!("Test files without test result - {}",  skip.len());

    for file in skip {
        println!("  {}", file)
    }
}
