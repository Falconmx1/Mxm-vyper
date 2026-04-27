use std::process::Command;
use std::net::TcpStream;
use std::io::{Read, Write};
use rand::Rng;

pub struct TorController {
    control_port: u16,
    password: Option<String>,
}

impl TorController {
    pub fn new(control_port: u16, password: Option<String>) -> Self {
        Self {
            control_port,
            password,
        }
    }
    
    pub fn renew_identity(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.control_port))?;
        
        // Autenticación
        if let Some(pass) = &self.password {
            let auth_cmd = format!("AUTHENTICATE \"{}\"\r\n", pass);
            stream.write_all(auth_cmd.as_bytes())?;
        } else {
            stream.write_all(b"AUTHENTICATE \"\"\r\n")?;
        }
        
        let mut buf = [0; 1024];
        stream.read(&mut buf)?;
        
        // Señal NEWNYM para nueva identidad
        stream.write_all(b"SIGNAL NEWNYM\r\n")?;
        stream.read(&mut buf)?;
        
        Ok(())
    }
    
    pub fn check_tor_running(&self) -> bool {
        TcpStream::connect(format!("127.0.0.1:{}", self.control_port)).is_ok()
    }
}

// Función para iniciar Tor automáticamente si no está corriendo
pub fn start_tor_if_needed() -> Result<(), Box<dyn std::error::Error>> {
    // Verificar si Tor ya está corriendo
    if TcpStream::connect("127.0.0.1:9050").is_ok() {
        println!("{} Tor already running on port 9050", "[✓]".bright_green());
        return Ok(());
    }
    
    println!("{} Starting Tor...", "[!]".bright_yellow());
    
    #[cfg(target_os = "linux")]
    {
        Command::new("systemctl")
            .args(&["start", "tor"])
            .output()?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("brew")
            .args(&["services", "start", "tor"])
            .output()?;
    }
    
    // Esperar a que Tor inicie
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    Ok(())
}
