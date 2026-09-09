use axum::{extract::State, routing::get, Router};
use aya::{
    include_bytes_aligned,
    maps::{PerCpuHashMap, Array},
    programs::{Lsm, Xdp, XdpFlags, TracePoint},
    Btf, Ebpf,
};
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::str::FromStr;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::Mutex,
    time::{self, Duration},
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    // 1. Data Resilience: Auto-create SQLite DB
    let db_path = "/home/soberano/NEXUS_ULTIMATE_CORE/nexus_intelligence.db";
    let opts =
        SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))?.create_if_missing(true);
    let db = SqlitePool::connect_with(opts).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS traffic_stats (
            protocol TEXT PRIMARY KEY,
            packet_count INTEGER DEFAULT 0,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&db)
    .await?;

    // 2. Load eBPF (XDP + LSM)
    let btf = Btf::from_sys_fs()?;
    let buf = include_bytes_aligned!("../../target/bpfel-unknown-none/release/nexus_ebpf");
    let mut bvar = aya::EbpfLoader::new().btf(Some(&btf)).load(buf)?;

    // 2.A Load XDP (Network)
    let prg: &mut Xdp = bvar.program_mut("nexus_monitor").unwrap().try_into()?;
    prg.load()?;
    let iface = get_default_interface().unwrap_or_else(|| "lo".to_string());
    println!(
        "📡 [eBPF] Enlazando sensor XDP a la interfaz de red: {}",
        iface
    );
    let _ = prg.attach(&iface, XdpFlags::default());

    // 2.B Load LSM (IP Protection)
    let lsm_prg: &mut Lsm = bvar.program_mut("nexus_ip_guard").unwrap().try_into()?;
    lsm_prg.load("file_open", &btf)?;
    let _ = lsm_prg.attach()?;

    // 2.C Load Tracepoints for Scheduler Latency
    let tp_wakeup: &mut TracePoint = bvar.program_mut("sched_wakeup").unwrap().try_into()?;
    tp_wakeup.load()?;
    let _ = tp_wakeup.attach("sched", "sched_wakeup");

    let tp_switch: &mut TracePoint = bvar.program_mut("sched_switch").unwrap().try_into()?;
    tp_switch.load()?;
    let _ = tp_switch.attach("sched", "sched_switch");



    let bpf_arc = Arc::new(Mutex::new(bvar));
    let db_clone = db.clone();
    let bpf_clone = bpf_arc.clone();

    // 3. Background Optimization: 5s Flush Cycle & Alerts
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if let Err(e) = flush_stats_to_db(&bpf_clone, &db_clone).await {
                eprintln!("X-FLUSH ERROR: {}", e);
            }
        }
    });

    let st = Arc::new(AppStateRef {
        bpf: bpf_arc,
        db: db.clone(),
    });

    let app = Router::new().route("/", get(dashboard)).with_state(st);

    println!("NEXUS: Online en Puerto 5000 (SQLite + 16-Thread eBPF Bridge)");
    let l = TcpListener::bind("127.0.0.1:5000").await?;
    axum::serve(l, app).await?;
    Ok(())
}

struct AppStateRef {
    #[allow(dead_code)]
    bpf: Arc<Mutex<Ebpf>>,
    db: SqlitePool,
}

async fn flush_stats_to_db(bpf_mutex: &Arc<Mutex<Ebpf>>, db: &SqlitePool) -> anyhow::Result<()> {
    let bpf = bpf_mutex.lock().await;
    if let Ok(m) = PerCpuHashMap::<_, u32, u64>::try_from(bpf.map("STATS").unwrap()) {
        for i in m.iter().flatten() {
            let n = match i.0 {
                1 => "ICMP",
                6 => "TCP",
                17 => "UDP",
                _ => "UNKNOWN",
            };
            let total: u64 = i.1.iter().sum();

            // 4. Mimetismo & Persistence: UPSERT in traffic_stats
            sqlx::query(
                "INSERT INTO traffic_stats (protocol, packet_count, last_updated)
                 VALUES ($1, $2, CURRENT_TIMESTAMP)
                 ON CONFLICT(protocol) DO UPDATE SET 
                    packet_count = $2,
                    last_updated = CURRENT_TIMESTAMP",
            )
            .bind(n)
            .bind(total as i64)
            .execute(db)
            .await?;

            // 5. Protocolo de Alerta: Telegram ID 8472077868
            if n == "TCP" && total > 1000 {
                send_telegram_alert(total).await?;
            }
        }
    }
    Ok(())
}

async fn send_telegram_alert(packet_count: u64) -> anyhow::Result<()> {
    let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_else(|_| "PLACEHOLDER".into());
    if token == "PLACEHOLDER" {
        return Ok(());
    }

    let chat_id = "8472077868";
    let text = format!(
        "🚨 NEXUS ALERT: Tráfico TCP Anómalo Detectado! Pkts: {}",
        packet_count
    );
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);

    let client = reqwest::Client::new();
    let _ = client
        .post(url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text
        }))
        .send()
        .await;

    Ok(())
}

async fn dashboard(State(st): State<Arc<AppStateRef>>) -> String {
    let mut out = String::from("--- NEXUS LIVE STATUS (SQLite Persistence Enabled) ---\n");

    if let Ok(rows) = sqlx::query("SELECT protocol, packet_count FROM traffic_stats")
        .fetch_all(&st.db)
        .await
    {
        for row in rows {
            let n: String = row.get("protocol");
            let c: i64 = row.get("packet_count");
            out.push_str(&format!("{}: {} pkts\n", n, c));
        }
    }

    out.push_str("\n--- ⚡ CPU SCHEDULER LATENCY (EMA) ---\n");
    let bpf = st.bpf.lock().await;
    if let Ok(cpu_latency_map) = Array::<_, u64>::try_from(bpf.map("CPU_LATENCY").unwrap()) {
        for cpu_id in 0..20 {
            if let Ok(val) = cpu_latency_map.get(&cpu_id, 0) {
                let lat_us = (val as f64) / 1000.0;
                let core_type = if cpu_id < 16 { "P-Core" } else { "E-Core" };
                out.push_str(&format!("CPU {:02} ({}): {:.3} μs\n", cpu_id, core_type, lat_us));
            }
        }
    } else {
        out.push_str("Error: No se pudo mapear CPU_LATENCY\n");
    }

    out
}

fn get_default_interface() -> Option<String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open("/proc/net/route").ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 2 {
            let iface = parts[0];
            let dest = parts[1];
            if dest == "00000000" {
                return Some(iface.to_string());
            }
        }
    }
    None
}
