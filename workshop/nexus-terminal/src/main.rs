use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
    time::Duration,
};
use sysinfo::System;
use tokio::sync::mpsc;

struct HistoryEntry {
    output: String,
}

struct App {
    input: String,
    cursor_position: usize,
    history: Vec<HistoryEntry>,
    suggestions: Vec<String>,
    suggestion_state: ListState,
    show_suggestions: bool,
    current_theme_color: Color,
    pty_writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    // Telemetría
    sys: System,
    cpu_usage: f32,
    mem_used: u64,
    mem_total: u64,
}

impl App {
    fn new(pty_writer: Box<dyn std::io::Write + Send>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let mem_total = sys.total_memory();
        let mem_used = sys.used_memory();
        let cpu_usage = sys.global_cpu_usage();

        Self {
            input: String::new(),
            cursor_position: 0,
            history: vec![HistoryEntry {
                output: "🔱 NEXUS Sovereign Terminal v0.1.0 inicializada.\nEscribe tu primer comando:\n".to_string(),
            }],
            suggestions: vec![],
            suggestion_state: ListState::default(),
            show_suggestions: false,
            current_theme_color: Color::Cyan,
            pty_writer: Arc::new(Mutex::new(pty_writer)),
            sys,
            cpu_usage,
            mem_used,
            mem_total,
        }
    }

    fn update_telemetry(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.cpu_usage = self.sys.global_cpu_usage();
        self.mem_used = self.sys.used_memory();
    }

    fn update_local_suggestions(&mut self) {
        if self.input.is_empty() {
            self.suggestions.clear();
            self.show_suggestions = false;
            return;
        }

        let common_commands = vec![
            "git status",
            "git diff",
            "cargo check -p core",
            "cargo run -p core",
            "just dev",
            "just sovereign",
            "cargo test",
            "nix-shell",
            "ls -la",
            "df -h",
            "free -m",
            "exit",
        ];

        self.suggestions = common_commands
            .into_iter()
            .filter(|cmd| cmd.starts_with(&self.input))
            .map(|cmd| cmd.to_string())
            .collect();

        self.show_suggestions = !self.suggestions.is_empty();
        if self.show_suggestions {
            self.suggestion_state.select(Some(0));
        } else {
            self.suggestion_state.select(None);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Inicializar PTY
    let pty_system = NativePtySystem::default();
    let pair = pty_system.openpty(PtySize {
        rows: 40,
        cols: 100,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let cmd = CommandBuilder::new("bash");
    let mut _child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    // 2. Inicializar consola de Ratatui
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(writer);

    let (tx, mut rx) = mpsc::channel::<String>(100);
    let (ai_tx, mut ai_rx) = mpsc::channel::<Vec<String>>(10);

    let mut pty_reader = reader;
    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            match pty_reader.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    let raw_str = String::from_utf8_lossy(&buffer[..n]).to_string();
                    let clean_str = clean_ansi_escapes(&raw_str);
                    let _ = tx.send(clean_str).await;
                }
                Err(_) => break,
            }
        }
    });

    let res = run_app(&mut terminal, &mut app, &mut rx, ai_tx, &mut ai_rx).await;

    // Restauración limpia
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("❌ Error en NEXUS Terminal: {:?}", err);
    }

    Ok(())
}

fn clean_ansi_escapes(input: &str) -> String {
    let re = regex::Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
    let cleaned = re.replace_all(input, "");
    cleaned.replace("\r\n", "\n").replace('\r', "\n")
}

async fn fetch_ai_completions(input: String) -> Result<Vec<String>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()?;
        
    let prompt = format!(
        "Sugiere exactamente de 3 a 5 comandos válidos de terminal Linux que comiencen o se relacionen con '{}'. Devuelve SOLAMENTE los comandos sugeridos separados por saltos de línea, sin explicaciones ni formato markdown.",
        input
    );

    let res = client
        .post("http://localhost:43211/v1/chat/completions")
        .json(&serde_json::json!({
            "model": "gemini-2.5-flash",
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 80,
            "temperature": 0.1
        }))
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("");

    let suggestions: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('`'))
        .collect();

    Ok(suggestions)
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: &mut mpsc::Receiver<String>,
    ai_tx: mpsc::Sender<Vec<String>>,
    ai_rx: &mut mpsc::Receiver<Vec<String>>,
) -> Result<()> {
    let mut last_telemetry_update = std::time::Instant::now();
    let mut last_input = String::new();
    let mut last_keypress = std::time::Instant::now();
    let mut pending_ai_request = false;

    loop {
        // Actualizar telemetría cada 2 segundos
        if last_telemetry_update.elapsed() >= Duration::from_secs(2) {
            app.update_telemetry();
            last_telemetry_update = std::time::Instant::now();
        }

        // Consultar autocompletado de la IA en background con debounce de 300ms
        if !app.input.is_empty() 
            && app.input != last_input 
            && last_keypress.elapsed() >= Duration::from_millis(300) 
            && !pending_ai_request 
        {
            last_input = app.input.clone();
            pending_ai_request = true;
            let ai_tx_clone = ai_tx.clone();
            let input_val = app.input.clone();

            tokio::spawn(async move {
                if let Ok(suggestions) = fetch_ai_completions(input_val).await {
                    if !suggestions.is_empty() {
                        let _ = ai_tx_clone.send(suggestions).await;
                    }
                }
            });
        }

        // Recibir sugerencias de la IA en background
        if let Ok(ai_suggestions) = ai_rx.try_recv() {
            app.suggestions = ai_suggestions;
            app.show_suggestions = !app.suggestions.is_empty();
            if app.show_suggestions {
                app.suggestion_state.select(Some(0));
            }
            pending_ai_request = false;
        }

        // Renderizado
        terminal.draw(|f| ui(f, app))?;

        // Leer del canal del PTY
        while let Ok(pty_output) = rx.try_recv() {
            if let Some(entry) = app.history.last_mut() {
                entry.output.push_str(&pty_output);
            }
        }

        // Eventos de teclado
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    last_keypress = std::time::Instant::now();
                    pending_ai_request = false; // Resetear bandera al presionar tecla

                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        return Ok(());
                    }

                    match key.code {
                        KeyCode::Char(c) => {
                            app.input.insert(app.cursor_position, c);
                            app.cursor_position += 1;
                            app.update_local_suggestions();
                        }
                        KeyCode::Backspace => {
                            if app.cursor_position > 0 {
                                app.cursor_position -= 1;
                                app.input.remove(app.cursor_position);
                                app.update_local_suggestions();
                            }
                        }
                        KeyCode::Left => {
                            if app.cursor_position > 0 {
                                app.cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if app.cursor_position < app.input.len() {
                                app.cursor_position += 1;
                            }
                        }
                        KeyCode::Up => {
                            if app.show_suggestions {
                                let index = match app.suggestion_state.selected() {
                                    Some(i) => {
                                        if i == 0 {
                                            app.suggestions.len() - 1
                                        } else {
                                            i - 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.suggestion_state.select(Some(index));
                            }
                        }
                        KeyCode::Down => {
                            if app.show_suggestions {
                                let index = match app.suggestion_state.selected() {
                                    Some(i) => {
                                        if i >= app.suggestions.len() - 1 {
                                            0
                                        } else {
                                            i + 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.suggestion_state.select(Some(index));
                            }
                        }
                        KeyCode::Enter => {
                            if app.show_suggestions && app.suggestion_state.selected().is_some() {
                                if let Some(selected) = app.suggestion_state.selected() {
                                    app.input = app.suggestions[selected].clone();
                                    app.cursor_position = app.input.len();
                                    app.show_suggestions = false;
                                }
                            } else {
                                let cmd = app.input.trim().to_string();
                                if !cmd.is_empty() {
                                    let mut writer = app.pty_writer.lock().unwrap();
                                    let _ = writeln!(writer, "{}", cmd);
                                    let _ = writer.flush();

                                    app.input.clear();
                                    app.cursor_position = 0;
                                    app.show_suggestions = false;
                                    last_input.clear();
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.show_suggestions = false;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let size = f.area();

    // Dividir pantalla horizontalmente: 75% Consola Principal, 25% Telemetría de Silicio
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(size);

    // Dividir panel izquierdo verticalmente: Historial de comandos (arriba) + Prompt (abajo)
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(main_layout[0]);

    // 1. RENDERIZAR HISTORIAL DE LÍNEAS (Panel Izquierdo Arriba)
    let mut history_lines = Vec::new();
    for entry in &app.history {
        for line in entry.output.lines() {
            let display_line = line.replace('\t', "    ");
            history_lines.push(Line::from(Span::styled(display_line, Style::default().fg(Color::Gray))));
        }
    }

    let max_visible_lines = left_chunks[0].height.saturating_sub(2) as usize;
    if history_lines.len() > max_visible_lines {
        let start_index = history_lines.len() - max_visible_lines;
        history_lines = history_lines[start_index..].to_vec();
    }

    let history_widget = Paragraph::new(history_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" NEXUS SOVEREIGN SHELL ")
                .border_style(Style::default().fg(app.current_theme_color)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(history_widget, left_chunks[0]);

    // 2. RENDERIZAR PROMPT DE ENTRADA (Panel Izquierdo Abajo)
    let input_widget = Paragraph::new(app.input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" ORÁCULO PROMPT ")
            .border_style(Style::default().fg(Color::Green)),
    );

    f.render_widget(input_widget, left_chunks[1]);

    f.set_cursor_position((
        left_chunks[1].x + app.cursor_position as u16 + 1,
        left_chunks[1].y + 1,
    ));

    // 3. RENDERIZAR TELEMETRÍA (Panel Derecho)
    let mem_total_gb = app.mem_total as f64 / 1024.0 / 1024.0 / 1024.0;
    let mem_used_gb = app.mem_used as f64 / 1024.0 / 1024.0 / 1024.0;

    let telemetry_text = vec![
        Line::from(vec![
            Span::styled("🔱 CORE: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("OMEGA-18 ÆGIS", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("⚡ STATUS: ", Style::default().fg(Color::Cyan)),
            Span::styled("ESTABILIZADO", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🖥️ CPU: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:.1}%", app.cpu_usage), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("💾 RAM: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{:.1} / {:.1} GB", mem_used_gb, mem_total_gb),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("📡 GATEWAY: ", Style::default().fg(Color::Cyan)),
            Span::styled(":43211", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("🏡 SANTUARIO: ", Style::default().fg(Color::Cyan)),
            Span::styled(":1420", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(Span::styled("--- CONTROLES ---", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("Ctrl+C: Salida Segura", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("Esc: Cerrar Autocomp.", Style::default().fg(Color::Gray))),
    ];

    let telemetry_widget = Paragraph::new(telemetry_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" TELEMETRÍA ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(telemetry_widget, main_layout[1]);

    // 4. POPUP DE SUGERENCIAS
    if app.show_suggestions {
        let items: Vec<ListItem> = app
            .suggestions
            .iter()
            .map(|s| ListItem::new(Span::raw(s)))
            .collect();

        let width = 40;
        let height = (app.suggestions.len() as u16 + 2).min(8);

        let popup_layout = Rect {
            x: left_chunks[1].x + 1,
            y: left_chunks[1].y.saturating_sub(height),
            width,
            height,
        };

        let list_widget = List::new(items)
            .block(
                Block::default()
                    .title(" Sugerencias ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(0, 80, 80))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(Clear, popup_layout);
        f.render_stateful_widget(list_widget, popup_layout, &mut app.suggestion_state);
    }
}
