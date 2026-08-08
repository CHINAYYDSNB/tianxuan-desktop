use serde::Serialize;

use crate::models::Host;
use crate::services::ssh_client::exec;

#[derive(Debug, Clone, Serialize, Default)]
pub struct HostMetrics {
    pub cpu_percent: f64,
    pub mem_total_mb: u64,
    pub mem_used_mb: u64,
    pub mem_percent: f64,
    pub disk_total_gb: f64,
    pub disk_used_gb: f64,
    pub disk_percent: f64,
    pub load_1: f64,
    pub load_5: f64,
    pub load_15: f64,
    pub online: bool,
}

/// Parse CPU usage from a single `/proc/stat` first-line sample.
/// Returns (total, idle) jiffies.
pub fn parse_cpu(line: &str) -> Option<(u64, u64)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 || parts[0] != "cpu" {
        return None;
    }
    let mut total = 0u64;
    let mut idle = 0u64;
    for (i, p) in parts.iter().enumerate().skip(1) {
        let v: u64 = p.parse().ok()?;
        total += v;
        if i == 4 || i == 5 {
            idle += v;
        }
    }
    Some((total, idle))
}

pub fn cpu_percent(sample1: (u64, u64), sample2: (u64, u64)) -> f64 {
    let (t1, i1) = sample1;
    let (t2, i2) = sample2;
    let d_total = t2.saturating_sub(t1);
    if d_total == 0 {
        return 0.0;
    }
    let d_idle = i2.saturating_sub(i1);
    ((d_total - d_idle) as f64 / d_total as f64) * 100.0
}

/// Parse `free -m` output into (total_mb, used_mb).
pub fn parse_mem(free_out: &str) -> Option<(u64, u64)> {
    for line in free_out.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let total: u64 = parts[1].parse().ok()?;
                let used: u64 = parts[2].parse().ok()?;
                return Some((total, used));
            }
        }
    }
    None
}

/// Parse `df -h` output for a mount point.
pub fn parse_df(df_out: &str, mount: &str) -> Option<(f64, f64, f64)> {
    for line in df_out.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 && parts[5] == mount {
            let total = parse_size(&parts[1])?;
            let used = parse_size(&parts[2])?;
            let percent = parts[4].trim_end_matches('%').parse::<f64>().ok()?;
            return Some((total, used, percent));
        }
    }
    None
}

fn parse_size(s: &str) -> Option<f64> {
    let s = s.trim();
    let (num, mult) = if let Some(v) = s.strip_suffix('G') {
        (v, 1024.0)
    } else if let Some(v) = s.strip_suffix('M') {
        (v, 1.0)
    } else if let Some(v) = s.strip_suffix('T') {
        (v, 1024.0 * 1024.0)
    } else {
        (s, 1.0)
    };
    num.parse::<f64>().ok().map(|n| n * mult / 1024.0)
}

/// Parse `uptime` output into load averages (1, 5, 15).
pub fn parse_load(uptime_out: &str) -> Option<(f64, f64, f64)> {
    let last_part = uptime_out.split("load average:").nth(1)?.trim();
    let parts: Vec<&str> = last_part
        .split(',')
        .map(|s| s.trim())
        .collect();
    if parts.len() < 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

pub async fn collect(host: &Host, password: &str) -> Result<HostMetrics, String> {
    // two CPU samples ~200ms apart for usage
    let s1 = exec(host, password, "cat /proc/stat | head -1").await?;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let s2 = exec(host, password, "cat /proc/stat | head -1").await?;
    let mem = exec(host, password, "free -m").await?;
    let disk = exec(host, password, "df -h /").await?;
    let up = exec(host, password, "uptime").await?;

    let cpu1 = parse_cpu(s1.stdout.trim());
    let cpu2 = parse_cpu(s2.stdout.trim());
    let mem_parsed = parse_mem(&mem.stdout);
    let disk_parsed = parse_df(&disk.stdout, "/");
    let load = parse_load(&up.stdout);

    let mut m = HostMetrics {
        online: true,
        ..Default::default()
    };

    if let (Some(c1), Some(c2)) = (cpu1, cpu2) {
        m.cpu_percent = cpu_percent(c1, c2);
    }
    if let Some((total, used)) = mem_parsed {
        m.mem_total_mb = total;
        m.mem_used_mb = used;
        m.mem_percent = if total > 0 {
            used as f64 / total as f64 * 100.0
        } else {
            0.0
        };
    }
    if let Some((total, used, percent)) = disk_parsed {
        m.disk_total_gb = total;
        m.disk_used_gb = used;
        m.disk_percent = percent;
    }
    if let Some((l1, l5, l15)) = load {
        m.load_1 = l1;
        m.load_5 = l5;
        m.load_15 = l15;
    }

    Ok(m)
}

pub fn err_offline(_e: &str) -> HostMetrics {
    HostMetrics {
        online: false,
        cpu_percent: 0.0,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROC_STAT: &str = "cpu  292889 0 346812 1882344 13250 0 1359 0 0 0";

    #[test]
    fn test_parse_cpu() {
        let (total, idle) = parse_cpu(PROC_STAT).unwrap();
        assert_eq!(total, 292889 + 346812 + 1882344 + 13250 + 1359);
        assert_eq!(idle, 1882344 + 13250);
    }

    #[test]
    fn test_parse_cpu_invalid() {
        assert!(parse_cpu("cpu 123").is_none());
        assert!(parse_cpu("cpux 1 2 3 4 5").is_none());
    }

    #[test]
    fn test_cpu_percent() {
        // first sample: idle heavy, second sample: busy
        let s1 = (1000, 900);
        let s2 = (2000, 1400);
        let pct = cpu_percent(s1, s2);
        // d_total=1000, d_idle=500 -> busy 50%
        assert!((pct - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_cpu_percent_zero_delta() {
        assert_eq!(cpu_percent((100, 50), (100, 50)), 0.0);
    }

    const FREE_M: &str = "              total        used        free      shared  buff/cache   available
Mem:           3948        1456        1240          18        1251        2102
Swap:             0           0           0";

    #[test]
    fn test_parse_mem() {
        let (total, used) = parse_mem(FREE_M).unwrap();
        assert_eq!(total, 3948);
        assert_eq!(used, 1456);
    }

    #[test]
    fn test_parse_mem_invalid() {
        assert!(parse_mem("not mem data").is_none());
    }

    const DF_H: &str = "Filesystem      Size  Used Avail Use% Mounted on
/dev/vda1        40G  8.2G   30G  22% /";

    #[test]
    fn test_parse_df() {
        let (total, used, percent) = parse_df(DF_H, "/").unwrap();
        assert!((total - 40.0).abs() < 0.01);
        assert!((used - 8.2).abs() < 0.01);
        assert!((percent - 22.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_df_no_match() {
        assert!(parse_df(DF_H, "/mnt").is_none());
    }

    const UPTIME: &str = " 01:24:15 up 5 days, 10:50,  0 user,  load average: 0.10, 0.04, 0.01";

    #[test]
    fn test_parse_load() {
        let (l1, l5, l15) = parse_load(UPTIME).unwrap();
        assert!((l1 - 0.10).abs() < 0.001);
        assert!((l5 - 0.04).abs() < 0.001);
        assert!((l15 - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_parse_size() {
        assert!((parse_size("40G").unwrap() - 40.0).abs() < 0.01);
        assert!((parse_size("512M").unwrap() - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_collect_real_server() {
        let Some(pw) = std::env::var("TX_TEST_PASSWORD").ok() else {
            eprintln!("SKIP: TX_TEST_PASSWORD not set");
            return;
        };
        let host = crate::models::Host::new(
            "Metrics CI".to_string(),
            std::env::var("TX_TEST_HOST").unwrap_or_else(|_| "47.100.33.169".to_string()),
            std::env::var("TX_TEST_PORT")
                .unwrap_or_else(|_| "22".to_string())
                .parse()
                .unwrap_or(22),
            std::env::var("TX_TEST_USER").unwrap_or_else(|_| "root".to_string()),
            crate::models::AuthType::Password,
            "ci".to_string(),
            "默认".to_string(),
            vec![],
            None,
            None,
        );
        let m = collect(&host, &pw).await.expect("collect metrics");
        assert!(m.online, "host should be online");
        assert!(m.mem_total_mb > 0, "mem total should be positive");
        assert!(m.cpu_percent >= 0.0 && m.cpu_percent <= 100.0);
        assert!(m.disk_total_gb > 0.0, "disk total should be positive");
        eprintln!(
            "metrics: cpu={:.1}% mem={}/{}MB ({:.1}%) disk={:.1}G/{:.1}G ({:.1}%) load={:.2}/{:.2}/{:.2}",
            m.cpu_percent, m.mem_used_mb, m.mem_total_mb, m.mem_percent,
            m.disk_used_gb, m.disk_total_gb, m.disk_percent, m.load_1, m.load_5, m.load_15
        );
    }
}
