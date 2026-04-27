use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};
use async_ssh2_lite::{AsyncSession, TcpStream};
use std::net::SocketAddr;
use rand::Rng;
use reqwest::Client;
use reqwest::Proxy;
use regex::Regex;
use url::Url;
use std::collections::HashMap;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[derive(Parser, Debug)]
#[command(name = "Mxm-vyper", version = "0.1.0", about = "Async brute-force auditor", long_about = None)]
struct Args {
    #[arg(short, long)]
    target: String,

    #[arg(short, long)]
    protocol: String,  // ssh, http, https, ftp, etc

    #[arg(short, long)]
    user: Option<String>,  // Para SSH, FTP

    #[arg(short, long)]
    wordlist: String,

    #[arg(short, long, default_value_t = 100)]
    threads: usize,

    #[arg(long)]
    proxy: Option<String>,  // proxy://host:port

    #[arg(long)]
    tor: bool,  // Usar Tor network

    #[arg(long)]
    tor_port: Option<u16>,  // Puerto Tor (default 9050)

    #[arg(long, default_value_t = 5)]
    timeout: u64,

    #[arg(long)]
    http_path: Option<String>,  // Ruta HTTP (default "/")

    #[arg(long)]
    http_method: Option<String>,  // GET, POST (default "GET")

    #[arg(long)]
    http_post_data: Option<String>,  // Datos POST (user=admin&pass=test)

    #[arg(long)]
    success_string: Option<String>,  // String en respuesta para éxito

    #[arg(long)]
    python_module: Option<String>,  // Módulo Python custom
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", r#"
    ╔══════════════════════════════════════════════════════════╗
    ║   Mxm-vyper 🔥 v0.1 - Async Multi-Protocol Brute-Forcer ║
    ║   Educational purposes only / Authorized testing only   ║
    ╚══════════════════════════════════════════════════════════╝
    "#.bright_red());

    let args = Args::parse();
    
    // Inicializar Python si se usa módulo custom
    if args.python_module.is_some() {
        pyo3::prepare_freethreaded_python();
        println!("{}", "[+] Python interpreter initialized".bright_green());
    }
    
    // Cargar wordlist
    let passwords = std::fs::read_to_string(&args.wordlist)?
        .lines()
        .map(String::from)
        .collect::<Vec<_>>();
    
    let total = passwords.len();
    let pb = ProgressBar::new(total as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/red}] {pos}/{len} ({eta}) | {msg}")
        .unwrap()
        .progress_chars("#>-"));
    
    let semaphore = Arc::new(Semaphore::new(args.threads));
    let mut handles = vec![];

    // Configurar cliente HTTP con Tor si es necesario
    let http_client = if args.tor || args.proxy.is_some() {
        let tor_port = args.tor_port.unwrap_or(9050);
        let proxy_url = if args.tor {
            format!("socks5h://127.0.0.1:{}", tor_port)
        } else {
            args.proxy.clone().unwrap()
        };
        
        match Proxy::all(proxy_url) {
            Ok(proxy) => match Client::builder().proxy(proxy).timeout(Duration::from_secs(args.timeout)).build() {
                Ok(client) => Some(client),
                Err(e) => {
                    eprintln!("{} Error building HTTP client: {}", "[!]".bright_red(), e);
                    None
                }
            },
            Err(e) => {
                eprintln!("{} Invalid proxy: {}", "[!]".bright_red(), e);
                None
            }
        }
    } else {
        Some(Client::builder().timeout(Duration::from_secs(args.timeout)).build().unwrap())
    };

    for (i, password) in passwords.into_iter().enumerate() {
        let permit = semaphore.clone().acquire_owned().await?;
        let target = args.target.clone();
        let user = args.user.clone();
        let protocol = args.protocol.clone();
        let proxy = args.proxy.clone();
        let timeout_secs = args.timeout;
        let pb_clone = pb.clone();
        let http_client_clone = http_client.clone();
        let http_path = args.http_path.clone();
        let http_method = args.http_method.clone();
        let http_post_data = args.http_post_data.clone();
        let success_string = args.success_string.clone();
        let python_module = args.python_module.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = permit;
            let mut success = false;
            let mut message = String::new();
            
            match protocol.as_str() {
                "ssh" => {
                    if let Some(user) = &user {
                        if ssh_bruteforce(&target, user, &password, timeout_secs, proxy.as_deref()).await {
                            success = true;
                            message = format!("SSH: {}@{}:{}", user, target, password);
                        }
                    }
                },
                "http" | "https" => {
                    if let Some(client) = http_client_clone {
                        if http_bruteforce(&client, &target, &password, &user, &http_path, &http_method, &http_post_data, &success_string).await {
                            success = true;
                            message = format!("HTTP: {}@{}", password, target);
                        }
                    }
                },
                "python" => {
                    if let Some(module) = python_module {
                        if let Ok(result) = python_custom_module(&module, &target, &password, &user).await {
                            success = result;
                            message = format!("Python plugin: {}@{}", password, target);
                        }
                    }
                },
                _ => {
                    pb_clone.println(format!("{} Unsupported protocol: {}", "[!]".bright_yellow(), protocol));
                }
            }
            
            pb_clone.inc(1);
            if success {
                pb_clone.println(format!("\n{} {} ✅\n", "[FOUND]".bright_green().bold(), message));
                pb_clone.set_message("CREDENTIAL FOUND!");
            } else if i % 100 == 0 {
                pb_clone.set_message(format!("Testing: {}", password.chars().take(20).collect::<String>()));
            }
            success
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let _ = handle.await;
    }
    
    pb.finish_with_message("✅ Scan completed");
    Ok(())
}

async fn ssh_bruteforce(
    target: &str, 
    user: &str, 
    pass: &str, 
    timeout_secs: u64,
    proxy: Option<&str>
) -> bool {
    let addr: SocketAddr = format!("{}:22", target).parse().unwrap_or_else(|_| "127.0.0.1:22".parse().unwrap());
    
    let stream = match timeout(Duration::from_secs(timeout_secs), TcpStream::connect(addr)).await {
        Ok(Ok(s)) => s,
        _ => return false,
    };
    
    let mut session = AsyncSession::new(stream, None).ok()?;
    let _ = timeout(Duration::from_secs(timeout_secs), session.handshake()).await;
    
    match timeout(Duration::from_secs(timeout_secs), session.userauth_password(user, pass)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

async fn http_bruteforce(
    client: &Client,
    target: &str,
    password: &str,
    username: &Option<String>,
    path: &Option<String>,
    method: &Option<String>,
    post_data_template: &Option<String>,
    success_string: &Option<String>,
) -> bool {
    let url = format!("{}{}", target, path.as_deref().unwrap_or("/"));
    let method_upper = method.as_deref().unwrap_or("GET").to_uppercase();
    
    let response = match method_upper.as_str() {
        "POST" => {
            let mut post_data = post_data_template.clone().unwrap_or_else(|| "".to_string());
            // Reemplazar placeholders
            post_data = post_data.replace("{PASS}", password);
            if let Some(user) = username {
                post_data = post_data.replace("{USER}", user);
            }
            
            client.post(&url)
                .body(post_data)
                .header("User-Agent", "Mxm-vyper/0.1")
                .send()
                .await
        },
        "GET" => {
            let mut url_with_params = url.clone();
            if let Some(user) = username {
                url_with_params = format!("{}?user={}&pass={}", url, user, password);
            }
            client.get(&url_with_params)
                .header("User-Agent", "Mxm-vyper/0.1")
                .send()
                .await
        },
        _ => return false,
    };
    
    match response {
        Ok(resp) => {
            if let Ok(body) = resp.text().await {
                if let Some(success_str) = success_string {
                    return body.contains(success_str);
                }
                // Si no hay string de éxito, asumimos código 200
                return true;
            }
            false
        },
        Err(_) => false,
    }
}

async fn python_custom_module(
    module_name: &str,
    target: &str,
    password: &str,
    username: &Option<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    Python::with_gil(|py| {
        let module = py.import(module_name)?;
        let func = module.getattr("bruteforce")?;
        
        let kwargs = PyDict::new(py);
        kwargs.set_item("target", target)?;
        kwargs.set_item("password", password)?;
        if let Some(user) = username {
            kwargs.set_item("username", user)?;
        }
        
        let result = func.call((), Some(kwargs))?;
        let bool_result = result.extract::<bool>()?;
        
        Ok(bool_result)
    })
}
