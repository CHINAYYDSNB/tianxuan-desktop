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
    pub io_read_kbps: f64,
    pub io_write_kbps: f64,
    pub net_rx_kbps: f64,
    pub net_tx_kbps: f64,
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

/// Aggregate disk read/write sectors from `/proc/diskstats` output.
/// Returns (read_sectors, write_sectors). Only counts real disks (skip loop/dm/ram).
pub fn parse_diskstats(disk_out: &str) -> Option<(u64, u64)> {
    let mut rd = 0u64;
    let mut wr = 0u64;
    let mut found = false;
    for line in disk_out.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 14 {
            continue;
        }
        let name = parts[2];
        if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("dm-") {
            continue;
        }
        let r_sectors: u64 = parts[5].parse().ok()?;
        let w_sectors: u64 = parts[9].parse().ok()?;
        rd += r_sectors;
        wr += w_sectors;
        found = true;
    }
    if found {
        Some((rd, wr))
    } else {
        None
    }
}

/// Aggregate network rx/tx bytes from `/proc/net/dev` output.
/// Returns (rx_bytes, tx_bytes) across all non-loopback interfaces.
pub fn parse_netdev(net_out: &str) -> Option<(u64, u64)> {
    let mut rx = 0u64;
    let mut tx = 0u64;
    let mut found = false;
    for line in net_out.lines().skip(2) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(idx) = line.find(':') else {
            continue;
        };
        let iface = line[..idx].trim();
        if iface == "lo" {
            continue;
        }
        let rest = line[idx + 1..].trim();
        let mut nums = rest.split_whitespace();
        let rx_b = nums.next()?;
        // skip 7 fields to reach tx_bytes (field index 9 overall: 1-based)
        // layout: rx_bytes(1) rx_packets(2) rx_errs(3) rx_drop(4) rx_fifo(5)
        //         rx_frame(6) rx_compressed(7) rx_multicast(8) tx_bytes(9)
        let mut tx_b = None;
        for (i, v) in nums.enumerate() {
            if i == 7 {
                tx_b = Some(v);
                break;
            }
        }
        let Some(tx_b) = tx_b else {
            continue;
        };
        rx += rx_b.parse::<u64>().ok()?;
        tx += tx_b.parse::<u64>().ok()?;
        found = true;
    }
    if found {
        Some((rx, tx))
    } else {
        None
    }
}

fn sectors_to_kb(sectors_delta: u64, secs: f64) -> f64 {
    (sectors_delta as f64) * 0.5 / secs.max(0.001)
}

pub fn kbps_delta(cur: u64, prev: u64, secs: f64) -> f64 {
    (cur.saturating_sub(prev)) as f64 / 1024.0 / secs.max(0.001)
}

const COLLECT_SCRIPT: &str = r#"
echo "===STAT1==="
head -1 /proc/stat
echo "===DISK1==="
cat /proc/diskstats
echo "===NET1==="
cat /proc/net/dev
sleep 0.5
echo "===STAT2==="
head -1 /proc/stat
echo "===DISK2==="
cat /proc/diskstats
echo "===NET2==="
cat /proc/net/dev
echo "===MEM==="
free -m
echo "===DISKFS==="
df -h /
echo "===UPTIME==="
uptime
"#;

fn section<'a>(all: &'a str, name: &str) -> Option<&'a str> {
    let start = all.find(&format!("==={name}===\n"))? + name.len() + 7;
    let rest = &all[start..];
    let end = rest.find("\n===").unwrap_or(rest.len());
    Some(rest[..end].trim_end())
}

pub async fn collect(host: &Host, password: &str) -> Result<HostMetrics, String> {
    let result = exec(host, password, COLLECT_SCRIPT).await?;
    let out = &result.stdout;

    let cpu1 = section(out, "STAT1").and_then(|s| parse_cpu(s.trim()));
    let cpu2 = section(out, "STAT2").and_then(|s| parse_cpu(s.trim()));
    let disk1 = section(out, "DISK1").and_then(|s| parse_diskstats(s));
    let disk2 = section(out, "DISK2").and_then(|s| parse_diskstats(s));
    let net1 = section(out, "NET1").and_then(|s| parse_netdev(s));
    let net2 = section(out, "NET2").and_then(|s| parse_netdev(s));
    let mem_parsed = section(out, "MEM").and_then(|s| parse_mem(s));
    let disk_fs = section(out, "DISKFS").and_then(|s| parse_df(s, "/"));
    let load = section(out, "UPTIME").and_then(|s| parse_load(s));

    let elapsed = 0.5f64;

    let mut m = HostMetrics {
        online: true,
        ..Default::default()
    };

    if let (Some(c1), Some(c2)) = (cpu1, cpu2) {
        m.cpu_percent = cpu_percent(c1, c2);
    }
    if let (Some((rd1, wr1)), Some((rd2, wr2))) = (disk1, disk2) {
        m.io_read_kbps = sectors_to_kb(rd2.saturating_sub(rd1), elapsed);
        m.io_write_kbps = sectors_to_kb(wr2.saturating_sub(wr1), elapsed);
    }
    if let (Some((rx1, tx1)), Some((rx2, tx2))) = (net1, net2) {
        m.net_rx_kbps = kbps_delta(rx2, rx1, elapsed);
        m.net_tx_kbps = kbps_delta(tx2, tx1, elapsed);
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
    if let Some((total, used, percent)) = disk_fs {
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

    const DISKSTATS: &str = "   8       0 sda 1000 0 2000 500 100 0 400 50 0 600 700 0
 252       0 dm-0 2000 0 4000 800 200 0 800 100 0 900 1100 0
   7       0 loop0 1 0 1 1 0 0 0 0 0 1 1 0";

    #[test]
    fn test_parse_diskstats_skips_virtual() {
        let (rd, wr) = parse_diskstats(DISKSTATS).unwrap();
        // only sda counted: rd=2000, wr=400 sectors
        assert_eq!(rd, 2000);
        assert_eq!(wr, 400);
    }

    #[test]
    fn test_parse_diskstats_empty() {
        assert!(parse_diskstats("").is_none());
    }

    const NETDEV: &str = "Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 1000    10    0    0    0     0          0         0   1000    10    0    0    0     0       0          0
  eth0: 5242880    512    0    0    0     0          0         0  1048576    128    0    0    0     0       0          0
 ens3: 2621440    256    0    0    0     0          0         0  2097152    256    0    0    0     0       0          0";

    #[test]
    fn test_parse_netdev_skips_lo() {
        let mut out = Vec::new();
        for line in NETDEV.lines().skip(2) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let idx = line.find(':').unwrap();
            let iface = line[..idx].trim();
            let rest = line[idx + 1..].trim();
            let nums: Vec<&str> = rest.split_whitespace().collect();
            out.push(format!("iface=[{iface}] first={:?} second={:?}", nums.first(), nums.get(1)));
        }
        eprintln!("DEBUG PARSE: {out:?}");
        let (rx, tx) = parse_netdev(NETDEV).unwrap();
        assert_eq!(rx, 5242880 + 2621440);
        assert_eq!(tx, 1048576 + 2097152);
    }

    #[test]
    fn test_parse_netdev_empty() {
        assert!(parse_netdev("").is_none());
    }

    #[test]
    fn test_kbps_delta() {
        // 1024 bytes in 1s = 1 KB/s = 1 kbps (using 1024 scale)
        assert!((kbps_delta(2048, 1024, 1.0) - 1.0).abs() < 0.001);
        // 0 delta -> 0
        assert_eq!(kbps_delta(100, 100, 1.0), 0.0);
    }

    #[test]
    fn test_sectors_to_kb() {
        // 2048 sectors = 1024 KB in 0.5s -> 2048 KB/s
        assert!((sectors_to_kb(2048, 0.5) - 2048.0).abs() < 0.001);
    }

    #[test]
    fn test_section_extraction() {
        let s = "===STAT1===\ncpu 1\n===MEM===\nMem: 1 2\n";
        assert_eq!(section(s, "STAT1"), Some("cpu 1"));
        assert_eq!(section(s, "MEM"), Some("Mem: 1 2"));
        assert_eq!(section(s, "NOPE"), None);
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
            "榛樿".to_string(),
            vec![],
        );
        let m = collect(&host, &pw).await.expect("collect metrics");
        assert!(m.online, "host should be online");
        assert!(m.mem_total_mb > 0, "mem total should be positive");
        assert!(m.cpu_percent >= 0.0 && m.cpu_percent <= 100.0);
        assert!(m.disk_total_gb > 0.0, "disk total should be positive");
        assert!(m.io_read_kbps >= 0.0 && m.io_write_kbps >= 0.0);
        assert!(m.net_rx_kbps >= 0.0 && m.net_tx_kbps >= 0.0);
        eprintln!(
            "metrics: cpu={:.1}% mem={}/{}MB ({:.1}%) disk={:.1}G/{:.1}G ({:.1}%) load={:.2}/{:.2}/{:.2} io={:.1}/{}KB/s net={:.1}/{}KB/s",
            m.cpu_percent, m.mem_used_mb, m.mem_total_mb, m.mem_percent,
            m.disk_used_gb, m.disk_total_gb, m.disk_percent,
            m.load_1, m.load_5, m.load_15,
            m.io_read_kbps, m.io_write_kbps, m.net_rx_kbps, m.net_tx_kbps
        );
    }
}



