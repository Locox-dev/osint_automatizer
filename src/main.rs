use inquire::{
    validator::Validation,
    Text, error::InquireResult,
};
use std::process::Command;
use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};


static TITLE: &str = r"
                _    ____   _____ _____ _   _ _______ 
     /\        | |  / __ \ / ____|_   _| \ | |__   __|
    /  \  _   _| |_| |  | | (___   | | |  \| |  | |   
   / /\ \| | | | __| |  | |\___ \  | | | . ` |  | |   
  / ____ \ |_| | |_| |__| |____) |_| |_| |\  |  | |   
 /_/    \_\__,_|\__|\____/|_____/|_____|_| \_|  |_|                                  
";

fn main() -> InquireResult<()> {
    clear_terminal_screen();
    println!("{}", TITLE);

    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let validator = |input: &str| {
        if input.chars().count() > 140 {
            Ok(Validation::Invalid(
                "You're only allowed 140 characters.".into(),
            ))
        } else {
            Ok(Validation::Valid)
        }
    };
    let age_validator = |input: &str| {
        if input == "" {
            Ok(Validation::Valid)
        } else {
            let input_string = input.to_string();
            let input_checker = input_string.parse::<i32>().is_ok();
            if input_checker == true {
                let input_num = input_string.parse::<i32>().unwrap();
                if input_num > 150 || input_num < 1{
                    Ok(Validation::Invalid(
                        "Please enter a valid age".into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            } else {

                Ok(Validation::Invalid(
                    "Please enter a valid age".into(),
                ))
            }
        }
    };

    println!("Please enter the known target informations below.");
    println!("If a value is unkown, press enter to skip it. More values = better result.");
    // Asking initial known values of the target.
    let first_name = Text::new("First name:")
        .with_validator(validator)
        .prompt_skippable()?;
    let last_name = Text::new("Last name:")
        .with_validator(validator)
        .prompt_skippable()?;
    let username = Text::new("Username:")
        .with_validator(validator)
        .prompt_skippable()?;
    let other_names = Text::new("Other known names:")
        .with_validator(validator)
        .with_help_message("You can use ',' to separete multiple names.")
        .prompt_skippable()?;
    let age = Text::new("Age:")
        .with_validator(age_validator)
        .prompt_skippable()?;
    let job_name = Text::new("Job name:")
        .with_validator(validator)
        .prompt_skippable()?;
    let studies = Text::new("Studies:")
        .with_validator(validator)
        .prompt_skippable()?;
    let emails = Text::new("Emails:")
        .with_validator(validator)
        .with_help_message("You can use ',' to separate multiple emails.")
        .prompt_skippable()?;
    let phone_number = Text::new("Phone number:")
        .with_validator(validator)
        .prompt_skippable()?;
    let location = Text::new("Location:")
        .with_validator(validator)
        .prompt_skippable()?;

    
    // Saving initial known values of the target.
    let mut target = Target::new();
    if let Some(v) = first_name {
        target.set("first_name", Value::Str(v));
    }
    if let Some(v) = last_name {
        target.set("last_name", Value::Str(v));
    }
    if let Some(v) = username {
        target.set("username", Value::Str(v));
    }
    if let Some(v) = other_names {
        let other_names_bad_vec: Vec<&str> = v.split(',').collect();
        let other_names_vec: Vec<String> = other_names_bad_vec.iter().map(|&s| s.trim().to_string()).collect();
        target.set("other_names", Value::VecStr(other_names_vec));
    }
    if let Some(v) = age {
        if v != "" {
            let age_num = v.parse::<i32>().unwrap();
            target.set("age", Value::Int(age_num));
        }
    }
    if let Some(v) = job_name {
        target.set("job_name", Value::Str(v));
    }
    if let Some(v) = studies {
        target.set("studies", Value::Str(v));
    }
    if let Some(v) = emails {
        let emails_bad_vec: Vec<&str> = v.split(',').collect();
        let emails_vec: Vec<String> = emails_bad_vec.iter().map(|&s| s.trim().to_string()).collect();
        target.set("emails", Value::VecStr(emails_vec));
    }
    if let Some(v) = phone_number {
        target.set("phone_number", Value::Str(v));
    }
    if let Some(v) = location {
        target.set("location", Value::Str(v));
    }

    //////////////////////////////////   Start getting some information!   //////////////////////////////////
    let m = MultiProgress::new();
    let gathered_infos = Arc::new(Mutex::new(TargetGatheringResult::new()));

    if !target.username.is_empty() {

        let gathered_infos_clone = gathered_infos.clone();

        let handle = thread::spawn(move || {

            let pb = m.add(ProgressBar::new(0));
            pb.set_style(spinner_style.clone());
            pb.set_message("Checking for accounts using Sherlock.");
            
            match tool_sherlock(target.username) {
                Ok(info) => {
                    gathered_infos_clone.lock().unwrap().set("accounts", Value::HMap(info));
                    pb.finish_with_message("Done checking for accounts!");
                },
                Err(why) => println!("Something went wrong {}", why)
            }
        });

        let _ = handle.join();
        
    }

    //println!("{:?}", gathered_infos.lock().unwrap().accounts);
    


    Ok(())
}

fn clear_terminal_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "cls"])
            .spawn()
            .expect("cls command failed to start")
            .wait()
            .expect("failed to wait");
    } else {
        Command::new("clear")
            .spawn()
            .expect("clear command failed to start")
            .wait()
            .expect("failed to wait");
    };
}

fn tool_sherlock(data: String) -> Result<HashMap<String, String>, std::io::Error>{
    let mut gathered_infos = HashMap::new();
    let sherlock = Command::new("cmd")
        .args(["/c", "python", "tools\\sherlock\\sherlock", data.as_str()])
        .output();

    match sherlock {
        Ok(res) => {
            let output = res.stdout;
            let output_string = String::from_utf8_lossy(&output);

            for line in output_string.lines() {
                if let Some(start) = line.find("[+] ") {
                    let content = &line[start + 4..];
                    if let Some(separator) = content.find(": ") {
                        let site_name = content[..separator].trim().to_string();
                        let url = content[separator + 2..].trim().to_string();
                        gathered_infos.insert(site_name, url);
                    }
                }
            }
        },
        Err(err) => {
            eprintln!("Failed to execute script: {}", err);
        }
    }

    Ok(gathered_infos)

}

enum Value {
    Str(String),
    VecStr(Vec<String>),
    Int(i32),
    HMap(HashMap<String, String>),
}

#[derive(Debug)]
struct Target {
    first_name: String,
    last_name: String,
    username: String,
    other_names: Vec<String>,
    age: i32,
    job_name: String,
    studies: String,
    emails: Vec<String>,
    phone_number: String,
    location: String,
}

impl Target {
    fn new() -> Self {
        Target {
            first_name: String::from(""),
            last_name: String::from(""),
            username: String::from(""),
            other_names: Vec::new(),
            age: 0,
            job_name: String::from(""),
            studies: String::from(""),
            emails: Vec::new(),
            phone_number: String::from(""),
            location: String::from(""),
        }
    }

    fn set(&mut self, key: &str, value: Value) {
        match value {
            Value::Str(v) => match key {
                "first_name" => self.first_name = v,
                "last_name" => self.last_name = v,
                "username" => self.username = v,
                "job_name" => self.job_name = v,
                "studies" => self.studies = v,
                "phone_number" => self.phone_number = v,
                "location" => self.location = v,
                _ => println!("Invalid key"),
            },
            Value::VecStr(v) => match key {
                "other_names" => self.other_names = v,
                "emails" => self.emails = v,
                _ => println!("Invalid key"),
            },
            Value::Int(v) => match key {
                "age" => self.age = v,
                _ => println!("Invalid key"),
            },
            Value::HMap(_v) => match key {
                _ => println!("Invalid key"),
            }
        }
    }

    /*fn get(&self, key: &str) -> Result<Value, std::io::Error> {
        let mut result = Value::Str("".to_string());
        match key {
            "first_name" => result = Value::Str(self.first_name.clone()),
            "last_name" => result = Value::Str(self.last_name.clone()),
            "username" => result = Value::Str(self.username.clone()),
            "other_names" => result = Value::VecStr(self.other_names.clone()),
            "age" => result = Value::Int(self.age.clone()),
            "job_name" => result = Value::Str(self.job_name.clone()),
            "studies" => result = Value::Str(self.studies.clone()),
            "emails" => result = Value::VecStr(self.emails.clone()),
            "phone_number" => result = Value::Str(self.phone_number.clone()),
            "location" => result = Value::Str(self.location.clone()),
            _ => println!("Invalid key")
        };

        Ok(result)
    }*/
}

struct TargetGatheringResult {
    accounts: HashMap<String, String>,
}

impl TargetGatheringResult {
    fn new() -> Self {
        TargetGatheringResult {
            accounts: HashMap::new(),
        }
    }

    fn set(&mut self, key: &str, value: Value) {
        match value {
            Value::Str(_v) => match key {
                _ => println!("Invalid key"),
            },
            Value::VecStr(_v) => match key {
                _ => println!("Invalid key"),
            },
            Value::Int(_v) => match key {
                _ => println!("Invalid key"),
            },
            Value::HMap(v) => match key {
                "accounts" => self.accounts = v,
                _ => println!("Invalid key"),
            }
        }
    }

    /*fn get(&self, key: &str) -> Option<&String> {
        self.accounts.get(key)
    }*/
}