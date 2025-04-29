use std::error::Error;
use sysinfo::{System, Disks};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize system information
    let mut sys = System::new_all();
    let disks = Disks::new_with_refreshed_list();
    
    // Refresh system information
    sys.refresh_all();

    // Print system information
    println!("=== SYSTEM INFORMATION ===");
    println!("System name: {}", System::name().unwrap_or_default());
    println!("Kernel version: {}", System::kernel_version().unwrap_or_default());
    println!("OS version: {}", System::os_version().unwrap_or_default());
    println!("Host name: {}", System::host_name().unwrap_or_default());
    
    // Print CPU information
    println!("\n=== CPU INFORMATION ===");
    println!("CPU Count: {}", sys.cpus().len());
    println!("CPU Brand: {}", sys.global_cpu_info().brand());
    println!("CPU Frequency: {} MHz", sys.global_cpu_info().frequency());
    
    // Print individual CPU cores
    for (i, cpu) in sys.cpus().iter().enumerate() {
        println!("CPU {}: {}% usage", i, cpu.cpu_usage());
    }

    // Print RAM information
    println!("\n=== MEMORY INFORMATION ===");
    println!("Total Memory: {} MB", sys.total_memory() / 1024 / 1024);
    println!("Used Memory: {} MB", sys.used_memory() / 1024 / 1024);
    println!("Free Memory: {} MB", (sys.total_memory() - sys.used_memory()) / 1024 / 1024);
    println!("Total Swap: {} MB", sys.total_swap() / 1024 / 1024);
    println!("Used Swap: {} MB", sys.used_swap() / 1024 / 1024);

    // Print disk information
    println!("\n=== DISK INFORMATION ===");
    for disk in disks.list() {
        println!("Disk name: {}", disk.name().to_string_lossy());
        println!("  Mount point: {}", disk.mount_point().to_string_lossy());
        println!("  File system: {}", disk.file_system().to_string_lossy());
        println!("  Total space: {} GB", disk.total_space() / 1024 / 1024 / 1024);
        println!("  Available space: {} GB", disk.available_space() / 1024 / 1024 / 1024);
        let used_space = disk.total_space() - disk.available_space();
        println!("  Used space: {} GB", used_space / 1024 / 1024 / 1024);
        if disk.total_space() > 0 {
            let usage_percent = (used_space as f64 / disk.total_space() as f64) * 100.0;
            println!("  Usage: {:.2}%", usage_percent);
        }
        println!();
    }

    // Fetch external IP
    println!("\n=== EXTERNAL IP ===");
    println!("Fetching your external IP address...");
    
    // Use a public API to get the external IP address
    let response = reqwest::get("https://api.ipify.org").await?;
    
    if response.status().is_success() {
        let ip = response.text().await?;
        println!("Your external IP address is: {}", ip);
    } else {
        println!("Failed to get IP address. Status: {}", response.status());
    }
    
    // Test network speed
    println!("\n=== NETWORK SPEED TEST ===");
    println!("Running network speed test (this may take a moment)...");
    
    test_internet_speed().await;
    
    Ok(())
}

// Custom implementation for network speed testing
async fn test_internet_speed() {
    // Download speed test
    println!("Testing download speed...");
    let download_speed = test_download_speed().await;
    println!("Download speed: {:.2} Mbps", download_speed);
    
    // Upload speed test
    println!("Testing upload speed...");
    let upload_speed = test_upload_speed().await;
    println!("Upload speed: {:.2} Mbps", upload_speed);
    
    // Ping test
    println!("Testing ping...");
    let ping = test_ping().await;
    println!("Ping: {:.2} ms", ping);
}

async fn test_download_speed() -> f64 {
    // Use a large file from a reliable server for testing
    let url = "https://speed.cloudflare.com/__down?bytes=100000000"; // 100MB file
    
    let start = Instant::now();
    
    match reqwest::get(url).await {
        Ok(response) => {
            if let Ok(bytes) = response.bytes().await {
                let duration = start.elapsed();
                let seconds = duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1_000_000_000.0;
                let bits = bytes.len() as f64 * 8.0;
                
                // Calculate Mbps
                return bits / seconds / 1_000_000.0;
            }
        },
        Err(_) => {}
    }
    
    0.0 // Return 0 if the test failed
}

async fn test_upload_speed() -> f64 {
    // Generate random data for upload test
    let data_size = 10_000_000; // 10MB
    let data = vec![0u8; data_size];
    
    let url = "https://speed.cloudflare.com/__up";
    let client = reqwest::Client::new();
    
    let start = Instant::now();
    
    match client.post(url)
        .body(data)
        .send()
        .await {
        Ok(_) => {
            let duration = start.elapsed();
            let seconds = duration.as_secs() as f64 + duration.subsec_nanos() as f64 / 1_000_000_000.0;
            let bits = data_size as f64 * 8.0;
            
            // Calculate Mbps
            return bits / seconds / 1_000_000.0;
        },
        Err(_) => {}
    }
    
    0.0 // Return 0 if the test failed
}

async fn test_ping() -> f64 {
    let url = "https://www.cloudflare.com";
    let client = reqwest::Client::new();
    
    let mut total_ms = 0.0;
    let attempts = 4;
    let mut successful_attempts = 0;
    
    for _ in 0..attempts {
        let start = Instant::now();
        
        match client.get(url).send().await {
            Ok(_) => {
                let duration = start.elapsed();
                let ms = duration.as_millis() as f64;
                total_ms += ms;
                successful_attempts += 1;
            },
            Err(_) => {}
        }
        
        // Wait a bit between ping attempts
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    if successful_attempts > 0 {
        total_ms / successful_attempts as f64
    } else {
        0.0
    }
}
