use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use eframe::egui;
use tokio::runtime::Runtime;

use crate::cliamp::spawn::{cliamp_binary, ConnectionPhase, DaemonChild};
use crate::ipc::client::IpcClient;
use crate::ipc::subscribe::spawn_subscribe_thread;
use crate::ipc::types::SubscribeMessage;
use crate::ipc::types::{playlists_from_job, ProviderPlaylist, Snapshot};
use crate::radio_api::channels::Channel;
use crate::radio_api::statistics::Statistics;
use crate::ui::{cliamp_radio, now_playing, radio_browser, status, transport};

pub const PROVIDER_RADIO: &str = "radio";

#[derive(Debug)]
enum WorkRequest {
    Playback(String),
    SetVolume(f64),
    UrlLoad { path: String, play: bool },
    ProviderPlaylists,
    ProviderCatalog { offset: u32, limit: u32 },
    ProviderSearch { query: String, offset: u32, limit: u32 },
    ProviderLoad { playlist: String },
    StatePoll,
}

#[derive(Debug)]
enum WorkResponse {
    Ok,
    Playlists(Vec<ProviderPlaylist>),
    Error(String),
    Snapshot(Snapshot),
}

pub struct CliampApp {
    rt: Runtime,
    http: reqwest::Client,
    connection: ConnectionPhase,
    daemon_child: Option<DaemonChild>,
    connect_started: Option<Instant>,
    subscribe_rx: Receiver<SubscribeMessage>,
    work_tx: Sender<WorkRequest>,
    work_rx: Receiver<WorkResponse>,
    snapshot: Option<Snapshot>,
    error_message: Option<String>,
    ipc_busy: bool,
    tab: AppTab,
    channels: Vec<Channel>,
    channels_loading: bool,
    stats: Option<Statistics>,
    radio_playlists: Vec<ProviderPlaylist>,
    radio_loading: bool,
    search_query: String,
    pending_refresh_channels: bool,
    pending_play_channel: Option<String>,
    pending_reload_radio_home: bool,
    pending_run_search: bool,
    pending_play_playlist_id: Option<String>,
    pending_open_catalog: bool,
    volume_debounce: Option<f64>,
    last_state_poll: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTab {
    CliampRadio,
    RadioBrowser,
}

impl CliampApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = &cc.egui_ctx;
        let rt = Runtime::new().expect("tokio runtime");
        let http = reqwest::Client::new();
        let (subscribe_tx, subscribe_rx) = mpsc::channel();
        spawn_subscribe_thread(subscribe_tx);
        let (work_tx, work_req_rx) = mpsc::channel();
        let (work_resp_tx, work_rx) = mpsc::channel();
        spawn_ipc_worker(work_req_rx, work_resp_tx);

        let mut app = Self {
            rt,
            http,
            connection: ConnectionPhase::Connecting,
            daemon_child: None,
            connect_started: Some(Instant::now()),
            subscribe_rx,
            work_tx,
            work_rx,
            snapshot: None,
            error_message: None,
            ipc_busy: false,
            tab: AppTab::CliampRadio,
            channels: Vec::new(),
            channels_loading: false,
            stats: None,
            radio_playlists: Vec::new(),
            radio_loading: false,
            search_query: String::new(),
            pending_refresh_channels: true,
            pending_reload_radio_home: false,
            pending_run_search: false,
            pending_play_channel: None,
            pending_play_playlist_id: None,
            pending_open_catalog: false,
            volume_debounce: None,
            last_state_poll: Instant::now(),
        };
        app.begin_connect();
        app
    }

    fn begin_connect(&mut self) {
        if cliamp_binary().is_none() {
            self.connection = ConnectionPhase::NoBinary;
            return;
        }
        if crate::cliamp::spawn::try_connect().is_ok() {
            self.connection = ConnectionPhase::Connected;
            self.pending_refresh_channels = true;
            self.queue_provider_playlists();
            return;
        }
        self.connection = ConnectionPhase::Connecting;
        self.connect_started = Some(Instant::now());
        match DaemonChild::spawn() {
            Ok(child) => self.daemon_child = Some(child),
            Err(e) => {
                self.connection = ConnectionPhase::Failed(e.to_string());
            }
        }
    }

    fn queue_provider_playlists(&mut self) {
        self.radio_loading = true;
        self.ipc_busy = true;
        let _ = self.work_tx.send(WorkRequest::ProviderPlaylists);
    }

    fn queue_playback(&mut self, op: &str) {
        self.ipc_busy = true;
        let _ = self.work_tx.send(WorkRequest::Playback(op.to_string()));
    }

    fn fetch_metadata(&mut self) {
        self.channels_loading = true;
        let http = self.http.clone();
        let channels_result = self.rt.block_on(async {
            crate::radio_api::channels::fetch_stations(&http).await
        });
        match channels_result {
            Ok(c) => self.channels = c,
            Err(e) => self.error_message = Some(format!("stations: {}", e)),
        }
        let stats_result = self.rt.block_on(async {
            crate::radio_api::statistics::fetch_statistics(&http).await
        });
        if let Ok(s) = stats_result {
            self.stats = Some(s);
        }
        self.channels_loading = false;
    }

    fn play_channel(&mut self, channel_id: &str) {
        let ch = self.channels.iter().find(|c| c.id == channel_id);
        let path = if let Some(c) = ch {
            if c.tracks > 0 {
                format!("https://radio.cliamp.stream/{}/stream", c.id)
            } else {
                format!("https://radio.cliamp.stream/{}/stream.pls", c.id)
            }
        } else {
            format!("https://radio.cliamp.stream/{}/stream.pls", channel_id)
        };
        self.ipc_busy = true;
        let _ = self.work_tx.send(WorkRequest::UrlLoad { path, play: true });
    }

    fn process_pending(&mut self) {
        if self.pending_refresh_channels {
            self.pending_refresh_channels = false;
            self.fetch_metadata();
        }
        if let Some(id) = self.pending_play_channel.take() {
            self.play_channel(&id);
        }
        if self.pending_reload_radio_home {
            self.pending_reload_radio_home = false;
            self.queue_provider_catalog(0, 50);
        }
        if self.pending_run_search {
            self.pending_run_search = false;
            let q = self.search_query.trim().to_string();
            if !q.is_empty() {
                self.radio_loading = true;
                self.ipc_busy = true;
                let _ = self.work_tx.send(WorkRequest::ProviderSearch {
                    query: q,
                    offset: 0,
                    limit: 50,
                });
            }
        }
        if let Some(pl) = self.pending_play_playlist_id.take() {
            self.ipc_busy = true;
            if let Some(path) = pl.strip_prefix("search:") {
                let _ = self.work_tx.send(WorkRequest::UrlLoad {
                    path: path.to_string(),
                    play: true,
                });
            } else {
                let _ = self.work_tx.send(WorkRequest::ProviderLoad { playlist: pl });
            }
        }
        if self.pending_open_catalog {
            self.pending_open_catalog = false;
            self.queue_provider_catalog(0, 50);
        }
        if let Some(vol) = self.volume_debounce.take() {
            self.ipc_busy = true;
            let _ = self.work_tx.send(WorkRequest::SetVolume(vol));
        }
    }

    fn queue_provider_catalog(&mut self, offset: u32, limit: u32) {
        self.radio_loading = true;
        self.ipc_busy = true;
        let _ = self.work_tx.send(WorkRequest::ProviderCatalog { offset, limit });
    }

    fn drain_work_responses(&mut self) {
        while let Ok(msg) = self.work_rx.try_recv() {
            self.ipc_busy = false;
            match msg {
                WorkResponse::Ok => {}
                WorkResponse::Snapshot(s) => self.snapshot = Some(s),
                WorkResponse::Playlists(list) => {
                    self.radio_playlists = list;
                    self.radio_loading = false;
                }
                WorkResponse::Error(e) => {
                    self.error_message = Some(e);
                    self.radio_loading = false;
                }
            }
        }
    }

    fn drain_subscribe(&mut self) {
        while let Ok(msg) = self.subscribe_rx.try_recv() {
            match msg {
                SubscribeMessage::Snapshot(s) => self.snapshot = Some(s),
                SubscribeMessage::Job(job) => {
                    if let Some(s) = job.snapshot {
                        self.snapshot = Some(s);
                    }
                    if job.state == "failed" {
                        let msg = job
                            .error
                            .and_then(|e| e.message)
                            .unwrap_or_else(|| "operation failed".to_string());
                        self.error_message = Some(msg);
                    }
                }
                SubscribeMessage::Overflow | SubscribeMessage::Disconnected => {
                    if self.connection == ConnectionPhase::Connected
                        && self.last_state_poll.elapsed() > Duration::from_secs(2)
                    {
                        self.last_state_poll = Instant::now();
                        let _ = self.work_tx.send(WorkRequest::StatePoll);
                    }
                }
            }
        }
    }

    fn tick_connection(&mut self) {
        if self.connection == ConnectionPhase::Connected
            || self.connection == ConnectionPhase::NoBinary
        {
            return;
        }
        if let Some(child) = &mut self.daemon_child {
            if let Some(msg) = child.exited() {
                self.connection = ConnectionPhase::Failed(msg);
                return;
            }
        }
        if crate::cliamp::spawn::try_connect().is_ok() {
            self.connection = ConnectionPhase::Connected;
            self.daemon_child = None;
            self.pending_refresh_channels = true;
            self.queue_provider_playlists();
            return;
        }
        if let Some(start) = self.connect_started {
            if start.elapsed() > Duration::from_secs(12) {
                let detail = self
                    .daemon_child
                    .as_ref()
                    .map(|c| c.stderr_tail())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "socket never appeared".to_string());
                self.connection = ConnectionPhase::Failed(detail);
            }
        }
    }
}

impl eframe::App for CliampApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick_connection();
        self.drain_subscribe();
        self.drain_work_responses();
        self.process_pending();

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Cliamp Desktop");
                ui.separator();
                if ui.selectable_label(self.tab == AppTab::CliampRadio, "Cliamp Radio").clicked() {
                    self.tab = AppTab::CliampRadio;
                }
                if ui
                    .selectable_label(self.tab == AppTab::RadioBrowser, "Radio Browser")
                    .clicked()
                {
                    self.tab = AppTab::RadioBrowser;
                    if self.radio_playlists.is_empty() && self.connection == ConnectionPhase::Connected {
                        self.pending_reload_radio_home = true;
                    }
                }
                if self.connection == ConnectionPhase::Connected {
                    ui.label(egui::RichText::new("● connected").color(egui::Color32::LIGHT_GREEN));
                }
            });
            status::connection_banner(ui, &self.connection);
            status::error_toast(ui, &self.error_message);
            if ui.button("Dismiss error").clicked() {
                self.error_message = None;
            }
        });

        egui::TopBottomPanel::bottom("transport").show(ctx, |ui| {
            let actions = transport::show_transport(ui, &self.snapshot);
            if actions.toggle {
                self.queue_playback("toggle");
            }
            if actions.next {
                self.queue_playback("next");
            }
            if actions.prev {
                self.queue_playback("prev");
            }
            if let Some(v) = actions.volume {
                self.volume_debounce = Some(v);
            }
            if self.ipc_busy {
                ui.spinner();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(280.0);
                    now_playing::show_now_playing(ui, &self.snapshot);
                });
                ui.separator();
                ui.vertical(|ui| {
                    if self.connection != ConnectionPhase::Connected {
                        ui.label("Connect to cliamp to browse and play.");
                        return;
                    }
                    match self.tab {
                        AppTab::CliampRadio => {
                            let mut refresh = false;
                            let mut play = None;
                            cliamp_radio::show_cliamp_radio(
                                ui,
                                &self.channels,
                                &self.stats,
                                self.channels_loading,
                                &mut refresh,
                                &mut play,
                            );
                            if refresh {
                                self.pending_refresh_channels = true;
                            }
                            if let Some(id) = play {
                                self.pending_play_channel = Some(id);
                            }
                        }
                        AppTab::RadioBrowser => {
                            let mut reload = false;
                            let mut search = false;
                            let mut play_id = None;
                            let mut open_cat = None;
                            radio_browser::show_radio_browser(
                                ui,
                                &mut self.search_query,
                                &self.radio_playlists,
                                self.radio_loading,
                                &mut reload,
                                &mut search,
                                &mut play_id,
                                &mut open_cat,
                            );
                            if reload {
                                self.pending_reload_radio_home = true;
                            }
                            if search {
                                self.pending_run_search = true;
                            }
                            if let Some(id) = play_id {
                                self.pending_play_playlist_id = Some(id);
                            }
                            if open_cat.is_some() {
                                self.pending_open_catalog = true;
                            }
                        }
                    }
                });
            });
        });

        ctx.request_repaint_after(Duration::from_millis(250));
    }
}

fn spawn_ipc_worker(req_rx: Receiver<WorkRequest>, resp_tx: Sender<WorkResponse>) {
    std::thread::spawn(move || {
        while let Ok(req) = req_rx.recv() {
            let result = handle_work(req);
            let _ = resp_tx.send(result);
        }
    });
}

fn handle_work(req: WorkRequest) -> WorkResponse {
    match req {
        WorkRequest::Playback(op) => match run_playback(&op) {
            Ok(_) => WorkResponse::Ok,
            Err(e) => WorkResponse::Error(e.to_string()),
        },
        WorkRequest::SetVolume(v) => match run_volume(v) {
            Ok(_) => WorkResponse::Ok,
            Err(e) => WorkResponse::Error(e.to_string()),
        },
        WorkRequest::UrlLoad { path, play } => match run_url_load(&path, play) {
            Ok(s) => {
                if let Some(snap) = s {
                    WorkResponse::Snapshot(snap)
                } else {
                    WorkResponse::Ok
                }
            }
            Err(e) => WorkResponse::Error(e.to_string()),
        },
        WorkRequest::ProviderPlaylists => match run_provider_playlists() {
            Ok(list) => WorkResponse::Playlists(list),
            Err(e) => WorkResponse::Error(e.to_string()),
        },
        WorkRequest::ProviderCatalog { offset, limit } => match run_provider_catalog(offset, limit) {
            Ok(list) => WorkResponse::Playlists(list),
            Err(e) => WorkResponse::Error(e.to_string()),
        },
        WorkRequest::ProviderSearch { query, offset, limit } => {
            match run_provider_search(&query, offset, limit) {
                Ok(list) => WorkResponse::Playlists(list),
                Err(e) => WorkResponse::Error(e.to_string()),
            }
        }
        WorkRequest::ProviderLoad { playlist } => match run_provider_load(&playlist) {
            Ok(s) => {
                if let Some(snap) = s {
                    WorkResponse::Snapshot(snap)
                } else {
                    WorkResponse::Ok
                }
            }
            Err(e) => WorkResponse::Error(e.to_string()),
        },
        WorkRequest::StatePoll => match run_state_poll() {
            Ok(s) => WorkResponse::Snapshot(s),
            Err(e) => WorkResponse::Error(e.to_string()),
        },
    }
}

fn with_client<F, T>(f: F) -> Result<T, anyhow::Error>
where
    F: FnOnce(&mut IpcClient) -> Result<T, anyhow::Error>,
{
    let mut client = IpcClient::connect()?;
    f(&mut client)
}

fn run_playback(op: &str) -> Result<(), anyhow::Error> {
    with_client(|c| c.playback(op))
}

fn run_volume(v: f64) -> Result<(), anyhow::Error> {
    with_client(|c| c.set_volume(v))
}

fn run_state_poll() -> Result<Snapshot, anyhow::Error> {
    with_client(|c| {
        c.state_get()?
            .ok_or_else(|| anyhow::anyhow!("empty snapshot"))
    })
}

fn run_url_load(path: &str, play: bool) -> Result<Option<Snapshot>, anyhow::Error> {
    with_client(|c| {
        let params = serde_json::json!({ "path": path, "play": play });
        let job = c.submit_and_wait("url.load", params, Duration::from_secs(120))?;
        if job.state == "failed" {
            let msg = job
                .error
                .and_then(|e| e.message)
                .unwrap_or_else(|| "url.load failed".to_string());
            anyhow::bail!(msg);
        }
        Ok(job.snapshot)
    })
}

fn run_provider_playlists() -> Result<Vec<ProviderPlaylist>, anyhow::Error> {
    with_client(|c| {
        let params = serde_json::json!({ "provider": PROVIDER_RADIO });
        let job = c.submit_and_wait("provider.playlists", params, Duration::from_secs(60))?;
        Ok(playlists_from_job(&job))
    })
}

fn run_provider_catalog(offset: u32, limit: u32) -> Result<Vec<ProviderPlaylist>, anyhow::Error> {
    with_client(|c| {
        let params = serde_json::json!({
            "provider": PROVIDER_RADIO,
            "offset": offset,
            "limit": limit
        });
        let job = c.submit_and_wait("provider.catalog", params, Duration::from_secs(120))?;
        Ok(playlists_from_job(&job))
    })
}

fn run_provider_search(query: &str, offset: u32, limit: u32) -> Result<Vec<ProviderPlaylist>, anyhow::Error> {
    with_client(|c| {
        let params = serde_json::json!({
            "provider": PROVIDER_RADIO,
            "query": query,
            "offset": offset,
            "limit": limit
        });
        let job = c.submit_and_wait("provider.search", params, Duration::from_secs(120))?;
        Ok(tracks_to_playlists(&job))
    })
}

fn tracks_to_playlists(job: &crate::ipc::types::JobRecord) -> Vec<ProviderPlaylist> {
    let mut out = Vec::new();
    if let Some(result) = &job.result {
        if let Some(items) = result.get("tracks").and_then(|v| v.as_array()) {
            for item in items.iter() {
                let title = item
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Station");
                let path = item.get("path").and_then(|v| v.as_str());
                out.push(ProviderPlaylist {
                    id: path.map(|p| format!("search:{}", p)),
                    name: Some(title.to_string()),
                    title: None,
                    section: Some("Search Results".to_string()),
                    extra: item.clone(),
                });
            }
        }
    }
    out
}

fn run_provider_load(playlist: &str) -> Result<Option<Snapshot>, anyhow::Error> {
    with_client(|c| {
        let params = serde_json::json!({
            "provider": PROVIDER_RADIO,
            "playlist": playlist
        });
        let job = c.submit_and_wait("provider.load", params, Duration::from_secs(120))?;
        if job.state == "failed" {
            let msg = job
                .error
                .and_then(|e| e.message)
                .unwrap_or_else(|| "provider.load failed".to_string());
            anyhow::bail!(msg);
        }
        let _ = c.playback("play");
        Ok(job.snapshot)
    })
}
