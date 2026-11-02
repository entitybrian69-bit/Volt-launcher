use std::{
    collections::HashMap,
    io::Read as _,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::skin_preview::{CapePreviewTexture, SkinPreviewTexture, fnv1a};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccountProvider {
    Offline,
    ElyBy,
    Microsoft,
}

impl AccountProvider {
    pub fn badge(self) -> &'static str {
        match self {
            Self::Offline => "OFFLINE",
            Self::ElyBy => "ELY.BY",
            Self::Microsoft => "MICROSOFT",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetCape {
    VoltCrimson,
    RoyalGold,
    ObsidianStealth,
}

impl PresetCape {
    pub fn label(self) -> &'static str {
        match self {
            Self::VoltCrimson => "Volt V Cape",
            Self::RoyalGold => "Crimson Crown",
            Self::ObsidianStealth => "Obsidian Dark",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct VoltAccount {
    pub id: String,
    pub provider: AccountProvider,
    pub name: String,
    pub uuid_hex: String,
    pub slim: bool,
    pub has_custom_skin: bool,
    pub has_custom_cape: bool,
    pub sync_to_servers: bool,
    pub signed_skin_url: Option<String>,
    pub skin_label: String,
    pub cape_label: String,
    pub ely_access_token: Option<String>,
    pub ely_client_token: Option<String>,
}

impl Default for VoltAccount {
    fn default() -> Self {
        let name = "Player".to_owned();
        let uuid = horizon_protocol::crypto::offline_uuid(&name);
        Self {
            id: format!("offline-{uuid:032x}"),
            provider: AccountProvider::Offline,
            name,
            uuid_hex: format!("{uuid:032x}"),
            slim: false,
            has_custom_skin: false,
            has_custom_cape: false,
            sync_to_servers: true,
            signed_skin_url: None,
            skin_label: "Default".into(),
            cape_label: "None".into(),
            ely_access_token: None,
            ely_client_token: None,
        }
    }
}

impl VoltAccount {
    pub fn uuid_u128(&self) -> u128 {
        horizon_client::auth::parse_uuid(&self.uuid_hex)
            .unwrap_or_else(|| horizon_protocol::crypto::offline_uuid(&self.name))
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct StoredState {
    active_id: String,
    accounts: Vec<VoltAccount>,
}

enum WorkerEvent {
    ElySignedIn {
        account: VoltAccount,
        skin_rgba: Option<(Vec<u8>, bool)>,
        cape_rgba: Option<Vec<u8>>,
    },
    SkinFetched {
        account_id: String,
        skin_rgba: Vec<u8>,
        slim: bool,
        cape_rgba: Option<Vec<u8>>,
        signed_url: Option<String>,
        source_label: String,
    },
    UrlCosmeticFetched {
        account_id: String,
        is_cape: bool,
        rgba: Vec<u8>,
        label: String,
    },
    SkinSigned {
        account_id: String,
        signed_url: String,
    },
    MsaPreview {
        account_id: String,
        skin_rgba: Option<(Vec<u8>, bool)>,
        cape_rgba: Option<Vec<u8>>,
    },
    Failed(String),
}

pub struct VoltAccountManager {
    data_dir: PathBuf,
    pub accounts: Vec<VoltAccount>,
    pub active_id: String,
    pub busy: Option<String>,
    pub status: Option<String>,
    tx: Sender<WorkerEvent>,
    rx: Receiver<WorkerEvent>,
    skin_cache: HashMap<String, SkinPreviewTexture>,
    cape_cache: HashMap<String, CapePreviewTexture>,
    msa_fetched: std::collections::HashSet<String>,
}

pub enum LaunchSession {
    Immediate(Option<PathBuf>),
    Microsoft,
    ElyBy {
        account_id: String,
        name: String,
        uuid_hex: String,
        access_token: String,
        client_token: Option<String>,
        session_path: PathBuf,
    },
}

impl VoltAccountManager {
    pub fn load(
        data_dir: &Path,
        fallback_name: &str,
        msa_accounts: &crate::account::Accounts,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let file = Self::state_path(data_dir);
        let stored: StoredState = std::fs::read_to_string(&file)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let mut mgr = Self {
            data_dir: data_dir.to_path_buf(),
            accounts: stored.accounts,
            active_id: stored.active_id,
            busy: None,
            status: None,
            tx,
            rx,
            skin_cache: HashMap::new(),
            cape_cache: HashMap::new(),
            msa_fetched: std::collections::HashSet::new(),
        };

        mgr.sync_msa_accounts(msa_accounts, false);

        if mgr.accounts.is_empty() {
            let initial_name = valid_or_default_name(fallback_name);
            let _ = mgr.create_offline(&initial_name);
        }

        if !mgr.accounts.iter().any(|a| a.id == mgr.active_id)
            && let Some(first) = mgr.accounts.first()
        {
            mgr.active_id = first.id.clone();
        }

        let ids: Vec<String> = mgr.accounts.iter().map(|a| a.id.clone()).collect();
        for id in ids {
            mgr.reload_preview_from_disk(&id);
        }
        mgr.save();
        mgr
    }

    fn state_path(data_dir: &Path) -> PathBuf {
        data_dir.join("launcher").join("volt_accounts.json")
    }

    fn cosmetics_dir(&self, account_id: &str) -> PathBuf {
        let safe: String = account_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        self.data_dir.join("launcher").join("cosmetics").join(safe)
    }

    pub fn skin_png_path(&self, account_id: &str) -> PathBuf {
        self.cosmetics_dir(account_id).join("skin.png")
    }

    pub fn cape_png_path(&self, account_id: &str) -> PathBuf {
        self.cosmetics_dir(account_id).join("cape.png")
    }

    pub fn ely_session_path(&self, account_id: &str) -> PathBuf {
        let safe: String = account_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        self.data_dir
            .join("launcher")
            .join(format!("ely_session_{safe}.json"))
    }

    pub fn save(&self) {
        let path = Self::state_path(&self.data_dir);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let stored = StoredState {
            active_id: self.active_id.clone(),
            accounts: self.accounts.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&stored) {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, json).is_ok() {
                let _ = std::fs::rename(&tmp, &path);
            }
        }
    }

    pub fn active(&self) -> Option<&VoltAccount> {
        self.accounts
            .iter()
            .find(|a| a.id == self.active_id)
            .or_else(|| self.accounts.first())
    }

    pub fn active_mut(&mut self) -> Option<&mut VoltAccount> {
        let id = self.active_id.clone();
        if self.accounts.iter().any(|a| a.id == id) {
            self.accounts.iter_mut().find(|a| a.id == id)
        } else {
            self.accounts.first_mut()
        }
    }

    pub fn select(&mut self, id: &str) {
        if self.accounts.iter().any(|a| a.id == id) {
            self.active_id = id.to_owned();
            self.save();
            self.ensure_msa_preview_if_needed();
        }
    }

    pub fn remove_account(&mut self, id: &str) -> Option<String> {
        let idx = self.accounts.iter().position(|a| a.id == id)?;
        let removed = self.accounts.remove(idx);
        let _ = std::fs::remove_dir_all(self.cosmetics_dir(id));
        let _ = std::fs::remove_file(self.ely_session_path(id));
        self.skin_cache.remove(id);
        self.cape_cache.remove(id);
        if self.accounts.is_empty() {
            let _ = self.create_offline("Player");
        } else if self.active_id == id
            && let Some(first) = self.accounts.first()
        {
            self.active_id = first.id.clone();
        }
        self.save();
        if removed.provider == AccountProvider::Microsoft {
            Some(removed.uuid_hex)
        } else {
            None
        }
    }

    pub fn sync_msa_accounts(&mut self, msa: &crate::account::Accounts, select_newest: bool) {
        let mut last_added = None;
        for entry in &msa.accounts {
            let clean_hex: String = entry.uuid.chars().filter(|c| *c != '-').collect();
            let id = format!("msa-{}", clean_hex.to_ascii_lowercase());
            if let Some(existing) = self.accounts.iter_mut().find(|a| a.id == id) {
                existing.name = entry.name.clone();
                existing.uuid_hex = clean_hex.to_ascii_lowercase();
            } else {
                self.accounts.push(VoltAccount {
                    id: id.clone(),
                    provider: AccountProvider::Microsoft,
                    name: entry.name.clone(),
                    uuid_hex: clean_hex.to_ascii_lowercase(),
                    slim: false,
                    has_custom_skin: false,
                    has_custom_cape: false,
                    sync_to_servers: true,
                    signed_skin_url: None,
                    skin_label: "Microsoft Profile".into(),
                    cape_label: "Microsoft Profile".into(),
                    ely_access_token: None,
                    ely_client_token: None,
                });
                last_added = Some(id);
            }
        }
        // Prune Microsoft entries that were signed out of Accounts store.
        self.accounts.retain(|a| {
            if a.provider != AccountProvider::Microsoft {
                return true;
            }
            msa.accounts.iter().any(|m| {
                let clean: String = m.uuid.chars().filter(|c| *c != '-').collect();
                format!("msa-{}", clean.to_ascii_lowercase()) == a.id
            })
        });
        if select_newest && let Some(id) = last_added {
            self.active_id = id;
        }
        self.ensure_msa_preview_if_needed();
        self.save();
    }

    fn ensure_msa_preview_if_needed(&mut self) {
        let Some(active) = self.active().cloned() else {
            return;
        };
        if active.provider == AccountProvider::Microsoft
            && !self.msa_fetched.contains(&active.id)
        {
            self.msa_fetched.insert(active.id.clone());
            let tx = self.tx.clone();
            let id = active.id.clone();
            let uuid_hex = active.uuid_hex.clone();
            let _ = std::thread::Builder::new()
                .name("volt-msa-preview".into())
                .stack_size(4 * 1024 * 1024)
                .spawn(move || {
                    if let Some((skin_rgba, cape_rgba)) = fetch_mojang_preview_by_uuid(&uuid_hex) {
                        let _ = tx.send(WorkerEvent::MsaPreview {
                            account_id: id,
                            skin_rgba,
                            cape_rgba,
                        });
                    }
                });
        }
    }

    pub fn create_offline(&mut self, raw_name: &str) -> Result<(), String> {
        let name = validate_mc_username(raw_name)?;
        let uuid = horizon_protocol::crypto::offline_uuid(&name);
        let id = format!("offline-{uuid:032x}");
        if let Some(existing) = self.accounts.iter_mut().find(|a| a.id == id) {
            existing.name = name;
            self.active_id = id;
            self.save();
            return Ok(());
        }
        let account = VoltAccount {
            id: id.clone(),
            provider: AccountProvider::Offline,
            name,
            uuid_hex: format!("{uuid:032x}"),
            slim: false,
            has_custom_skin: false,
            has_custom_cape: false,
            sync_to_servers: true,
            signed_skin_url: None,
            skin_label: "Default".into(),
            cape_label: "None".into(),
            ely_access_token: None,
            ely_client_token: None,
        };
        self.accounts.push(account);
        self.active_id = id;
        self.save();
        self.status = Some("Offline account ready".into());
        Ok(())
    }

    pub fn rename_active_offline(&mut self, raw_name: &str) -> Result<(), String> {
        let name = validate_mc_username(raw_name)?;
        let Some(acc) = self.active_mut() else {
            return Err("No active account".into());
        };
        if acc.provider != AccountProvider::Offline {
            return Err("Only Offline accounts can change username locally".into());
        }
        let uuid = horizon_protocol::crypto::offline_uuid(&name);
        acc.name = name;
        acc.uuid_hex = format!("{uuid:032x}");
        self.save();
        self.status = Some("Offline username updated".into());
        Ok(())
    }

    pub fn toggle_slim(&mut self) {
        let mut trigger_sign = None;
        if let Some(acc) = self.active_mut() {
            acc.slim = !acc.slim;
            let id = acc.id.clone();
            let slim = acc.slim;
            let need_resign = acc.has_custom_skin && acc.sync_to_servers;
            if let Some(tex) = self.skin_cache.get_mut(&id) {
                tex.slim = slim;
            }
            if need_resign {
                trigger_sign = Some((id, slim));
            }
        }
        self.save();
        if let Some((id, slim)) = trigger_sign {
            self.spawn_mineskin_sign(&id, slim);
        }
    }

    pub fn toggle_server_sync(&mut self) {
        let mut trigger_sign = None;
        if let Some(acc) = self.active_mut() {
            acc.sync_to_servers = !acc.sync_to_servers;
            if acc.sync_to_servers && acc.has_custom_skin && acc.signed_skin_url.is_none() {
                trigger_sign = Some((acc.id.clone(), acc.slim));
            }
        }
        self.save();
        if let Some((id, slim)) = trigger_sign {
            self.spawn_mineskin_sign(&id, slim);
        }
    }

    pub fn clear_skin(&mut self) {
        let Some(id) = self.active().map(|a| a.id.clone()) else {
            return;
        };
        let _ = std::fs::remove_file(self.skin_png_path(&id));
        self.skin_cache.remove(&id);
        if let Some(acc) = self.active_mut() {
            acc.has_custom_skin = false;
            acc.signed_skin_url = None;
            acc.skin_label = "Default".into();
        }
        self.save();
        self.status = Some("Reset to default skin".into());
    }

    pub fn clear_cape(&mut self) {
        let Some(id) = self.active().map(|a| a.id.clone()) else {
            return;
        };
        let _ = std::fs::remove_file(self.cape_png_path(&id));
        self.cape_cache.remove(&id);
        if let Some(acc) = self.active_mut() {
            acc.has_custom_cape = false;
            acc.cape_label = "None".into();
        }
        self.save();
        self.status = Some("Cape removed".into());
    }

    pub fn apply_preset_cape(&mut self, preset: PresetCape) -> Result<(), String> {
        let Some(id) = self.active().map(|a| a.id.clone()) else {
            return Err("No active account".into());
        };
        let rgba = generate_preset_cape_rgba(preset);
        let path = self.cape_png_path(&id);
        write_rgba_png(&path, 64, 32, &rgba[..64 * 32 * 4])?;
        self.cape_cache.insert(
            id,
            CapePreviewTexture {
                hash: fnv1a(&rgba),
                rgba,
            },
        );
        if let Some(acc) = self.active_mut() {
            acc.has_custom_cape = true;
            acc.cape_label = preset.label().into();
        }
        self.save();
        self.status = Some(format!("Equipped {}", preset.label()));
        Ok(())
    }

    pub fn on_saf_cosmetic_imported(&mut self, is_cape: bool, path: &Path) {
        let Some((id, slim, sync)) = self
            .active()
            .map(|a| (a.id.clone(), a.slim, a.sync_to_servers))
        else {
            return;
        };
        self.reload_preview_from_disk(&id);
        if let Some(acc) = self.active_mut() {
            if is_cape {
                acc.has_custom_cape = path.exists();
                acc.cape_label = "Custom PNG".into();
            } else {
                acc.has_custom_skin = path.exists();
                acc.signed_skin_url = None;
                acc.skin_label = "Custom PNG".into();
            }
        }
        self.save();
        if !is_cape && sync {
            self.spawn_mineskin_sign(&id, slim);
        }
    }

    pub fn reload_preview_from_disk(&mut self, account_id: &str) {
        let slim = self
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .is_some_and(|a| a.slim);
        let skin_path = self.skin_png_path(account_id);
        if let Ok(bytes) = std::fs::read(&skin_path)
            && let Ok((w, h, rgba)) = horizon_client::hud::decode_bytes(&bytes)
            && let Some(processed) = horizon_client::skins::process_legacy_skin(w, h, &rgba)
        {
            self.skin_cache.insert(
                account_id.to_owned(),
                SkinPreviewTexture {
                    hash: fnv1a(&processed),
                    rgba: processed,
                    slim,
                },
            );
        }
        let cape_path = self.cape_png_path(account_id);
        if let Ok(bytes) = std::fs::read(&cape_path)
            && let Ok((w, h, rgba)) = horizon_client::hud::decode_bytes(&bytes)
            && let Some(processed) = horizon_client::skins::process_cape(w, h, &rgba)
        {
            self.cape_cache.insert(
                account_id.to_owned(),
                CapePreviewTexture {
                    hash: fnv1a(&processed),
                    rgba: processed,
                },
            );
        }
    }

    pub fn active_skin_preview(&self) -> Option<&SkinPreviewTexture> {
        let acc = self.active()?;
        self.skin_cache.get(&acc.id)
    }

    pub fn active_cape_preview(&self) -> Option<&CapePreviewTexture> {
        let acc = self.active()?;
        self.cape_cache.get(&acc.id)
    }

    pub fn begin_ely_login(&mut self, login: &str, password: &str, totp: &str) {
        let login = login.trim().to_owned();
        let password = password.to_owned();
        let totp = totp.trim().to_owned();
        if login.is_empty() || password.is_empty() {
            self.status = Some("Enter your Ely.by username/email and password".into());
            return;
        }
        self.busy = Some("Signing in to Ely.by...".into());
        let tx = self.tx.clone();
        if let Err(err) = std::thread::Builder::new()
            .name("volt-ely-auth".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                match authenticate_ely_by(&login, &password, &totp) {
                    Ok((account, skin_rgba, cape_rgba)) => {
                        let _ = tx.send(WorkerEvent::ElySignedIn {
                            account,
                            skin_rgba,
                            cape_rgba,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(WorkerEvent::Failed(e));
                    }
                }
            })
        {
            self.busy = None;
            self.status = Some(format!("Could not start Ely.by login thread: {err}"));
        }
    }

    pub fn begin_fetch_skin_by_username(&mut self, username: &str) {
        let Some(account_id) = self.active().map(|a| a.id.clone()) else {
            return;
        };
        let username = username.trim().to_owned();
        if let Err(e) = validate_mc_username(&username) {
            self.status = Some(e);
            return;
        }
        self.busy = Some(format!("Fetching skin for {username}..."));
        let tx = self.tx.clone();
        if let Err(err) = std::thread::Builder::new()
            .name("volt-skin-fetch".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || match fetch_skin_by_username_blocking(&username) {
                Ok((skin_rgba, slim, cape_rgba, signed_url, source_label)) => {
                    let _ = tx.send(WorkerEvent::SkinFetched {
                        account_id,
                        skin_rgba,
                        slim,
                        cape_rgba,
                        signed_url,
                        source_label,
                    });
                }
                Err(e) => {
                    let _ = tx.send(WorkerEvent::Failed(e));
                }
            })
        {
            self.busy = None;
            self.status = Some(format!("Could not start skin fetch thread: {err}"));
        }
    }

    pub fn begin_fetch_cosmetic_url(&mut self, url: &str, is_cape: bool) {
        let Some(account_id) = self.active().map(|a| a.id.clone()) else {
            return;
        };
        let url = url.trim().to_owned();
        if !url.starts_with("https://") && !url.starts_with("http://") {
            self.status = Some("Enter an https:// or http:// PNG URL".into());
            return;
        }
        self.busy = Some(if is_cape {
            "Downloading cape PNG...".into()
        } else {
            "Downloading skin PNG...".into()
        });
        let tx = self.tx.clone();
        if let Err(err) = std::thread::Builder::new()
            .name("volt-cosmetic-url".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || match fetch_cosmetic_url_blocking(&url, is_cape) {
                Ok(rgba) => {
                    let _ = tx.send(WorkerEvent::UrlCosmeticFetched {
                        account_id,
                        is_cape,
                        rgba,
                        label: "Custom URL".into(),
                    });
                }
                Err(e) => {
                    let _ = tx.send(WorkerEvent::Failed(e));
                }
            })
        {
            self.busy = None;
            self.status = Some(format!("Could not start cosmetic download thread: {err}"));
        }
    }

    fn spawn_mineskin_sign(&self, account_id: &str, slim: bool) {
        let path = self.skin_png_path(account_id);
        if !path.exists() {
            return;
        }
        let id = account_id.to_owned();
        let tx = self.tx.clone();
        let _ = std::thread::Builder::new()
            .name("volt-mineskin".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                if let Ok(png_bytes) = std::fs::read(&path)
                    && let Some(signed_url) = upload_mineskin_signed_url(&png_bytes, slim)
                {
                    let _ = tx.send(WorkerEvent::SkinSigned {
                        account_id: id,
                        signed_url,
                    });
                }
            });
    }

    pub fn poll(&mut self) -> Option<String> {
        let mut latest_msg = None;
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                WorkerEvent::ElySignedIn {
                    mut account,
                    skin_rgba,
                    cape_rgba,
                } => {
                    self.busy = None;
                    let id = account.id.clone();
                    if let Some((rgba, slim)) = skin_rgba {
                        if write_rgba_png(&self.skin_png_path(&id), 64, 64, &rgba).is_ok() {
                            account.has_custom_skin = true;
                            account.slim = slim;
                            account.skin_label = "Ely.by Skin".into();
                        }
                        self.skin_cache.insert(
                            id.clone(),
                            SkinPreviewTexture {
                                hash: fnv1a(&rgba),
                                rgba,
                                slim,
                            },
                        );
                    }
                    if let Some(rgba) = cape_rgba {
                        if write_rgba_png(&self.cape_png_path(&id), 64, 32, &rgba[..64 * 32 * 4])
                            .is_ok()
                        {
                            account.has_custom_cape = true;
                            account.cape_label = "Ely.by Cape".into();
                        }
                        self.cape_cache.insert(
                            id.clone(),
                            CapePreviewTexture {
                                hash: fnv1a(&rgba),
                                rgba,
                            },
                        );
                    }
                    let _ = self.write_ely_session_file(&account);
                    let name = account.name.clone();
                    if let Some(existing) = self.accounts.iter_mut().find(|a| a.id == id) {
                        *existing = account;
                    } else {
                        self.accounts.push(account);
                    }
                    self.active_id = id;
                    self.save();
                    let msg = format!("Signed in to Ely.by as {name}");
                    self.status = Some(msg.clone());
                    latest_msg = Some(msg);
                }
                WorkerEvent::SkinFetched {
                    account_id,
                    skin_rgba,
                    slim,
                    cape_rgba,
                    signed_url,
                    source_label,
                } => {
                    self.busy = None;
                    let _ = write_rgba_png(&self.skin_png_path(&account_id), 64, 64, &skin_rgba);
                    self.skin_cache.insert(
                        account_id.clone(),
                        SkinPreviewTexture {
                            hash: fnv1a(&skin_rgba),
                            rgba: skin_rgba,
                            slim,
                        },
                    );
                    let mut equipped_cape = false;
                    if let Some(c_rgba) = cape_rgba
                        && write_rgba_png(
                            &self.cape_png_path(&account_id),
                            64,
                            32,
                            &c_rgba[..64 * 32 * 4],
                        )
                        .is_ok()
                    {
                        self.cape_cache.insert(
                            account_id.clone(),
                            CapePreviewTexture {
                                hash: fnv1a(&c_rgba),
                                rgba: c_rgba,
                            },
                        );
                        equipped_cape = true;
                    }
                    let mut need_mineskin = false;
                    if let Some(acc) = self.accounts.iter_mut().find(|a| a.id == account_id) {
                        acc.has_custom_skin = true;
                        acc.slim = slim;
                        acc.signed_skin_url = signed_url;
                        acc.skin_label = source_label.clone();
                        if equipped_cape {
                            acc.has_custom_cape = true;
                            acc.cape_label = source_label.clone();
                        }
                        need_mineskin = acc.sync_to_servers && acc.signed_skin_url.is_none();
                    }
                    self.save();
                    if need_mineskin {
                        self.spawn_mineskin_sign(&account_id, slim);
                    }
                    let msg = format!("Equipped skin ({source_label})");
                    self.status = Some(msg.clone());
                    latest_msg = Some(msg);
                }
                WorkerEvent::UrlCosmeticFetched {
                    account_id,
                    is_cape,
                    rgba,
                    label,
                } => {
                    self.busy = None;
                    let mut trigger_sign = None;
                    if is_cape {
                        let _ = write_rgba_png(
                            &self.cape_png_path(&account_id),
                            64,
                            32,
                            &rgba[..64 * 32 * 4],
                        );
                        self.cape_cache.insert(
                            account_id.clone(),
                            CapePreviewTexture {
                                hash: fnv1a(&rgba),
                                rgba,
                            },
                        );
                        if let Some(acc) = self.accounts.iter_mut().find(|a| a.id == account_id) {
                            acc.has_custom_cape = true;
                            acc.cape_label = label;
                        }
                    } else {
                        let _ = write_rgba_png(&self.skin_png_path(&account_id), 64, 64, &rgba);
                        let slim = self
                            .accounts
                            .iter()
                            .find(|a| a.id == account_id)
                            .is_some_and(|a| a.slim);
                        self.skin_cache.insert(
                            account_id.clone(),
                            SkinPreviewTexture {
                                hash: fnv1a(&rgba),
                                rgba,
                                slim,
                            },
                        );
                        if let Some(acc) = self.accounts.iter_mut().find(|a| a.id == account_id) {
                            acc.has_custom_skin = true;
                            acc.signed_skin_url = None;
                            acc.skin_label = label;
                            if acc.sync_to_servers {
                                trigger_sign = Some(acc.slim);
                            }
                        }
                    }
                    self.save();
                    if let Some(slim) = trigger_sign {
                        self.spawn_mineskin_sign(&account_id, slim);
                    }
                    let msg = if is_cape {
                        "Cape equipped from URL".to_owned()
                    } else {
                        "Skin equipped from URL".to_owned()
                    };
                    self.status = Some(msg.clone());
                    latest_msg = Some(msg);
                }
                WorkerEvent::SkinSigned {
                    account_id,
                    signed_url,
                } => {
                    if let Some(acc) = self.accounts.iter_mut().find(|a| a.id == account_id) {
                        acc.signed_skin_url = Some(signed_url);
                    }
                    self.save();
                    let msg = "Skin signed for multiplayer server sync".to_owned();
                    self.status = Some(msg.clone());
                    latest_msg = Some(msg);
                }
                WorkerEvent::MsaPreview {
                    account_id,
                    skin_rgba,
                    cape_rgba,
                } => {
                    if let Some((rgba, slim)) = skin_rgba {
                        self.skin_cache.insert(
                            account_id.clone(),
                            SkinPreviewTexture {
                                hash: fnv1a(&rgba),
                                rgba,
                                slim,
                            },
                        );
                    }
                    if let Some(rgba) = cape_rgba {
                        self.cape_cache.insert(
                            account_id,
                            CapePreviewTexture {
                                hash: fnv1a(&rgba),
                                rgba,
                            },
                        );
                    }
                }
                WorkerEvent::Failed(err) => {
                    self.busy = None;
                    self.status = Some(err.clone());
                    latest_msg = Some(err);
                }
            }
        }
        latest_msg
    }

    pub fn write_ely_session_file(&self, acc: &VoltAccount) -> Option<PathBuf> {
        let token = acc.ely_access_token.as_deref()?;
        let path = self.ely_session_path(&acc.id);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let body = serde_json::json!({
            "name": acc.name,
            "uuid": acc.uuid_hex,
            "access_token": token,
            "user_type": "ely",
            "join_url": "https://authserver.ely.by/session/join",
        });
        std::fs::write(&path, body.to_string()).ok()?;
        Some(path)
    }

    /// Prepares the active account right before launching Minecraft without
    /// blocking the Android UI thread on network validation/refresh calls.
    pub fn prepare_for_launch(&mut self) -> (String, LaunchSession) {
        let Some(acc) = self.active().cloned() else {
            horizon_client::skins::clear_offline_runtime();
            return ("Player".into(), LaunchSession::Immediate(None));
        };

        if acc.provider == AccountProvider::ElyBy {
            self.apply_cosmetic_runtime(&acc);
            if let Some(access_token) = acc.ely_access_token.clone() {
                let session_path = self.ely_session_path(&acc.id);
                return (
                    acc.name.clone(),
                    LaunchSession::ElyBy {
                        account_id: acc.id,
                        name: acc.name,
                        uuid_hex: acc.uuid_hex,
                        access_token,
                        client_token: acc.ely_client_token,
                        session_path,
                    },
                );
            }
            return (acc.name, LaunchSession::Immediate(None));
        }

        if acc.provider == AccountProvider::Microsoft {
            let cape_path = acc
                .has_custom_cape
                .then(|| self.cape_png_path(&acc.id))
                .filter(|p| p.exists());
            if cape_path.is_some() {
                horizon_client::skins::set_offline_runtime(None, false, cape_path.as_deref(), None);
            } else {
                horizon_client::skins::clear_offline_runtime();
            }
            return (acc.name, LaunchSession::Microsoft);
        }

        // Offline account
        self.apply_cosmetic_runtime(&acc);
        (acc.name, LaunchSession::Immediate(None))
    }

    pub fn update_ely_token(&mut self, account_id: &str, new_token: String) {
        if let Some(stored) = self.accounts.iter_mut().find(|a| a.id == account_id) {
            stored.ely_access_token = Some(new_token);
            self.save();
        }
    }

    fn apply_cosmetic_runtime(&self, acc: &VoltAccount) {
        let skin_path = acc
            .has_custom_skin
            .then(|| self.skin_png_path(&acc.id))
            .filter(|p| p.exists());
        let cape_path = acc
            .has_custom_cape
            .then(|| self.cape_png_path(&acc.id))
            .filter(|p| p.exists());

        let sync_cmd = if acc.sync_to_servers && acc.has_custom_skin {
            if let Some(url) = &acc.signed_skin_url {
                let variant = if acc.slim { "SLIM" } else { "CLASSIC" };
                Some(format!("skin url \"{url}\" {variant}"))
            } else if let Some(src_name) = acc.skin_label.strip_prefix("Mojang: ") {
                Some(format!("skin set {src_name}"))
            } else {
                None
            }
        } else {
            None
        };

        if skin_path.is_some() || cape_path.is_some() || sync_cmd.is_some() {
            horizon_client::skins::set_offline_runtime(
                skin_path.as_deref(),
                acc.slim,
                cape_path.as_deref(),
                sync_cmd,
            );
        } else {
            horizon_client::skins::clear_offline_runtime();
        }
    }
}

fn valid_or_default_name(raw: &str) -> String {
    validate_mc_username(raw).unwrap_or_else(|_| "Player".into())
}

pub fn validate_mc_username(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 16 {
        return Err("Username must be 1 to 16 characters".into());
    }
    if !trimmed
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("Username can only use letters, numbers, and '_'".into());
    }
    Ok(trimmed.to_owned())
}

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(7))
        .timeout(Duration::from_secs(14))
        .build()
}

fn read_bounded(resp: ureq::Response, limit: usize) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    resp.into_reader()
        .take((limit + 1) as u64)
        .read_to_end(&mut buf)
        .ok()?;
    (buf.len() <= limit).then_some(buf)
}

fn authenticate_ely_by(
    login: &str,
    password: &str,
    totp: &str,
) -> Result<(VoltAccount, Option<(Vec<u8>, bool)>, Option<Vec<u8>>), String> {
    let client_token = format!("volt-{:016x}", rand_u64());
    let full_secret = if totp.is_empty() {
        password.to_owned()
    } else {
        format!("{password}:{totp}")
    };
    let req_body = serde_json::json!({
        "username": login,
        "password": full_secret,
        "clientToken": client_token,
        "requestUser": true,
    });

    let resp = http_agent()
        .post("https://authserver.ely.by/auth/authenticate")
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .send_string(&req_body.to_string());

    let body_bytes = match resp {
        Ok(r) => read_bounded(r, 256 * 1024).ok_or("Empty response from Ely.by")?,
        Err(ureq::Error::Status(_code, r)) => {
            let err_bytes = read_bounded(r, 64 * 1024).unwrap_or_default();
            let msg = serde_json::from_slice::<serde_json::Value>(&err_bytes)
                .ok()
                .and_then(|v| {
                    v.get("errorMessage")
                        .and_then(|s| s.as_str())
                        .map(str::to_owned)
                })
                .unwrap_or_else(|| "Ely.by rejected credentials".into());
            if msg.to_ascii_lowercase().contains("two factor") {
                return Err("2FA enabled on Ely.by: enter your 6-digit code in 2FA".into());
            }
            return Err(format!("Ely.by: {msg}"));
        }
        Err(e) => return Err(format!("Could not reach Ely.by: {e}")),
    };

    let json: serde_json::Value =
        serde_json::from_slice(&body_bytes).map_err(|_| "Invalid JSON from Ely.by")?;
    let access_token = json
        .get("accessToken")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or("Ely.by response missing accessToken")?
        .to_owned();
    let profile = json
        .get("selectedProfile")
        .ok_or("No Minecraft profile on this Ely.by account")?;
    let name = profile
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Ely.by profile missing name")?
        .to_owned();
    let raw_id = profile
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or("Ely.by profile missing UUID")?;
    let uuid = horizon_client::auth::parse_uuid(raw_id).ok_or("Ely.by UUID invalid")?;
    let uuid_hex = format!("{uuid:032x}");

    let (skin_rgba, cape_rgba) = fetch_ely_textures_by_name(&name);
    let slim = skin_rgba.as_ref().is_some_and(|(_, s)| *s);

    let acc = VoltAccount {
        id: format!("ely-{uuid_hex}"),
        provider: AccountProvider::ElyBy,
        name,
        uuid_hex,
        slim,
        has_custom_skin: skin_rgba.is_some(),
        has_custom_cape: cape_rgba.is_some(),
        sync_to_servers: true,
        signed_skin_url: None,
        skin_label: if skin_rgba.is_some() {
            "Ely.by Skin".into()
        } else {
            "Default".into()
        },
        cape_label: if cape_rgba.is_some() {
            "Ely.by Cape".into()
        } else {
            "None".into()
        },
        ely_access_token: Some(access_token),
        ely_client_token: Some(client_token),
    };

    Ok((acc, skin_rgba, cape_rgba))
}

pub fn prepare_ely_session_off_thread(
    account_id: String,
    name: String,
    uuid_hex: String,
    access_token: String,
    client_token: Option<String>,
    session_path: PathBuf,
) -> (Option<PathBuf>, Option<(String, String)>) {
    let refreshed = client_token
        .as_deref()
        .and_then(|client| refresh_ely_token_fast(&access_token, client));
    let active_token = refreshed.as_deref().unwrap_or(access_token.as_str());
    if let Some(parent) = session_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let body = serde_json::json!({
        "name": name,
        "uuid": uuid_hex,
        "access_token": active_token,
        "user_type": "ely",
        "join_url": "https://authserver.ely.by/session/join",
    });
    let written = std::fs::write(&session_path, body.to_string())
        .ok()
        .map(|()| session_path);
    (written, refreshed.map(|tok| (account_id, tok)))
}

fn fast_launch_http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(4))
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build()
}

fn refresh_ely_token_fast(access_token: &str, client_token: &str) -> Option<String> {
    let agent = fast_launch_http_agent();
    let validate_body = serde_json::json!({ "accessToken": access_token });
    if agent
        .post("https://authserver.ely.by/auth/validate")
        .set("Content-Type", "application/json")
        .send_string(&validate_body.to_string())
        .is_ok()
    {
        return None;
    }
    let refresh_body = serde_json::json!({
        "accessToken": access_token,
        "clientToken": client_token,
    });
    let resp = agent
        .post("https://authserver.ely.by/auth/refresh")
        .set("Content-Type", "application/json")
        .send_string(&refresh_body.to_string())
        .ok()?;
    let bytes = read_bounded(resp, 128 * 1024)?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    v.get("accessToken")
        .and_then(|s| s.as_str())
        .map(str::to_owned)
}

fn fetch_ely_textures_by_name(name: &str) -> (Option<(Vec<u8>, bool)>, Option<Vec<u8>>) {
    let url = format!("https://skinsystem.ely.by/textures/{name}");
    let Ok(resp) = http_agent().get(&url).call() else {
        return (None, None);
    };
    let Some(bytes) = read_bounded(resp, 128 * 1024) else {
        return (None, None);
    };
    let Ok(tex_json) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return (None, None);
    };
    let skin = tex_json.get("SKIN").and_then(|s| {
        let skin_url = s.get("url")?.as_str()?;
        let slim = s
            .get("metadata")
            .and_then(|m| m.get("model"))
            .and_then(|m| m.as_str())
            == Some("slim");
        let rgba = fetch_cosmetic_url_blocking(skin_url, false).ok()?;
        Some((rgba, slim))
    });
    let cape = tex_json.get("CAPE").and_then(|c| {
        let cape_url = c.get("url")?.as_str()?;
        fetch_cosmetic_url_blocking(cape_url, true).ok()
    });
    (skin, cape)
}

fn fetch_mojang_preview_by_uuid(
    uuid_hex: &str,
) -> Option<(Option<(Vec<u8>, bool)>, Option<Vec<u8>>)> {
    let url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{uuid_hex}?unsigned=false"
    );
    let resp = http_agent().get(&url).call().ok()?;
    let bytes = read_bounded(resp, 256 * 1024)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let props = json.get("properties")?.as_array()?;
    let tex_b64 = props.iter().find_map(|p| {
        (p.get("name")?.as_str()? == "textures")
            .then(|| p.get("value")?.as_str())
            .flatten()
    })?;
    let decoded = horizon_client::skins::base64_decode(tex_b64)?;
    let payload: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let textures = payload.get("textures")?;
    let skin = textures.get("SKIN").and_then(|s| {
        let skin_url = s.get("url")?.as_str()?;
        let slim = s
            .get("metadata")
            .and_then(|m| m.get("model"))
            .and_then(|m| m.as_str())
            == Some("slim");
        let rgba = fetch_cosmetic_url_blocking(skin_url, false).ok()?;
        Some((rgba, slim))
    });
    let cape = textures.get("CAPE").and_then(|c| {
        let cape_url = c.get("url")?.as_str()?;
        fetch_cosmetic_url_blocking(cape_url, true).ok()
    });
    Some((skin, cape))
}

fn fetch_skin_by_username_blocking(
    username: &str,
) -> Result<(Vec<u8>, bool, Option<Vec<u8>>, Option<String>, String), String> {
    // 1. Try Mojang official profile first (provides signed textures.minecraft.net URL for server sync!).
    let mojang_lookup = format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        username.to_ascii_lowercase()
    );
    if let Ok(resp) = http_agent().get(&mojang_lookup).call()
        && let Some(bytes) = read_bounded(resp, 64 * 1024)
        && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes)
        && let Some(id_str) = v.get("id").and_then(|s| s.as_str())
        && let Some(real_name) = v.get("name").and_then(|s| s.as_str())
    {
        let prof_url = format!(
            "https://sessionserver.mojang.com/session/minecraft/profile/{id_str}?unsigned=false"
        );
        if let Ok(p_resp) = http_agent().get(&prof_url).call()
            && let Some(p_bytes) = read_bounded(p_resp, 256 * 1024)
            && let Ok(p_json) = serde_json::from_slice::<serde_json::Value>(&p_bytes)
            && let Some(props) = p_json.get("properties").and_then(|p| p.as_array())
            && let Some(tex_b64) = props.iter().find_map(|p| {
                (p.get("name")?.as_str()? == "textures")
                    .then(|| p.get("value")?.as_str())
                    .flatten()
            })
            && let Some(decoded) = horizon_client::skins::base64_decode(tex_b64)
            && let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&decoded)
            && let Some(textures) = payload.get("textures")
            && let Some(skin_obj) = textures.get("SKIN")
            && let Some(skin_url) = skin_obj.get("url").and_then(|u| u.as_str())
            && let Ok(skin_rgba) = fetch_cosmetic_url_blocking(skin_url, false)
        {
            let slim = skin_obj
                .get("metadata")
                .and_then(|m| m.get("model"))
                .and_then(|m| m.as_str())
                == Some("slim");
            let cape_rgba = textures
                .get("CAPE")
                .and_then(|c| c.get("url")?.as_str())
                .and_then(|u| fetch_cosmetic_url_blocking(u, true).ok());
            return Ok((
                skin_rgba,
                slim,
                cape_rgba,
                Some(skin_url.to_owned()),
                format!("Mojang: {real_name}"),
            ));
        }
    }

    // 2. Fallback to Ely.by CustomSkinLoader catalog.
    let (ely_skin, ely_cape) = fetch_ely_textures_by_name(username);
    if let Some((skin_rgba, slim)) = ely_skin {
        return Ok((
            skin_rgba,
            slim,
            ely_cape,
            None,
            format!("Ely.by: {username}"),
        ));
    }

    Err(format!("No Mojang or Ely.by skin found for '{username}'"))
}

fn fetch_cosmetic_url_blocking(url: &str, is_cape: bool) -> Result<Vec<u8>, String> {
    let resp = http_agent()
        .get(url)
        .call()
        .map_err(|e| format!("Download failed: {e}"))?;
    let bytes = read_bounded(resp, 4 * 1024 * 1024).ok_or("Image too large or unreadable")?;
    let (w, h, rgba) = horizon_client::hud::decode_bytes(&bytes)
        .or_else(|_| {
            image::load_from_memory(&bytes)
                .map(|img| {
                    let rgba8 = img.to_rgba8();
                    let (iw, ih) = rgba8.dimensions();
                    (iw, ih, rgba8.into_raw())
                })
                .map_err(|_| ())
        })
        .map_err(|_| "URL did not return a valid image")?;
    if is_cape {
        horizon_client::skins::process_cape(w, h, &rgba).ok_or("Invalid cape dimensions".into())
    } else {
        horizon_client::skins::process_legacy_skin(w, h, &rgba)
            .ok_or("Skin image must be 64x64 or 64x32".into())
    }
}

/// Signs a custom 64x64 PNG via MineSkin's public relay so SkinsRestorer / `/skin`
/// servers can display custom local PNG skins to every player on the server.
fn upload_mineskin_signed_url(png_bytes: &[u8], slim: bool) -> Option<String> {
    let boundary = format!("----VoltLauncherBoundary{:016x}", rand_u64());
    let variant = if slim { "slim" } else { "classic" };
    let mut body = Vec::with_capacity(png_bytes.len() + 512);
    let push_field = |buf: &mut Vec<u8>, name: &str, val: &str| {
        buf.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        buf.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n{val}\r\n").as_bytes(),
        );
    };
    push_field(&mut body, "variant", variant);
    push_field(&mut body, "visibility", "1");
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"skin.png\"\r\nContent-Type: image/png\r\n\r\n",
    );
    body.extend_from_slice(png_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let resp = http_agent()
        .post("https://api.mineskin.org/generate/upload")
        .set("User-Agent", "VoltLauncher/0.7.6")
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        )
        .send_bytes(&body)
        .ok()?;
    let bytes = read_bounded(resp, 256 * 1024)?;
    let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    json.get("data")?
        .get("texture")?
        .get("url")?
        .as_str()
        .filter(|u| u.contains("textures.minecraft.net/texture/"))
        .map(str::to_owned)
}

fn write_rgba_png(path: &Path, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("png.tmp");
    std::fs::write(&tmp, &png_bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

fn rand_u64() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0x9e37_79b9_7f4a_7c15, |d| {
            (d.as_secs() << 20) ^ u64::from(d.subsec_nanos())
        });
    let addr = (&nanos as *const u64 as usize) as u64;
    nanos.wrapping_mul(0x517c_c1b7_2722_0a95) ^ addr
}

/// Procedurally generates crisp 64x64 Minecraft cape textures (front at 1..11 x 1..17,
/// inner back at 12..22 x 1..17, sides/top/bottom filled cleanly).
fn generate_preset_cape_rgba(preset: PresetCape) -> Vec<u8> {
    let mut img = vec![0u8; 64 * 64 * 4];
    let mut set_px = |x: usize, y: usize, c: [u8; 4]| {
        if x < 64 && y < 64 {
            let idx = (y * 64 + x) * 4;
            img[idx..idx + 4].copy_from_slice(&c);
        }
    };

    let (bg, border, accent, highlight, inner) = match preset {
        PresetCape::VoltCrimson => (
            [16, 16, 20, 255],
            [218, 30, 55, 255],
            [236, 42, 68, 255],
            [250, 245, 248, 255],
            [48, 12, 18, 255],
        ),
        PresetCape::RoyalGold => (
            [118, 16, 28, 255],
            [234, 186, 62, 255],
            [248, 212, 92, 255],
            [255, 244, 198, 255],
            [64, 10, 18, 255],
        ),
        PresetCape::ObsidianStealth => (
            [12, 13, 16, 255],
            [42, 44, 52, 255],
            [188, 28, 48, 255],
            [115, 120, 132, 255],
            [20, 20, 24, 255],
        ),
    };

    // Fill entire 22x17 cape cuboid net so edges/top/bottom have zero transparent seams.
    for y in 0..17 {
        for x in 0..22 {
            set_px(x, y, border);
        }
    }
    // Inner face (12..22, 1..17)
    for y in 1..17 {
        for x in 12..22 {
            set_px(x, y, inner);
        }
    }
    // Outer front face (1..11, 1..17) — 10 wide (dx = 0..10), 16 tall (dy = 0..16)
    for dy in 0..16 {
        for dx in 0..10 {
            let edge = dx == 0 || dx == 9 || dy == 0 || dy == 15;
            let shade = if edge {
                border
            } else {
                let grad = (dy as i32 * 2) - 10;
                [
                    (bg[0] as i32 + grad).clamp(6, 255) as u8,
                    (bg[1] as i32 + grad / 2).clamp(6, 255) as u8,
                    (bg[2] as i32 + grad / 2).clamp(8, 255) as u8,
                    255,
                ]
            };
            set_px(1 + dx, 1 + dy, shade);
        }
    }

    match preset {
        PresetCape::VoltCrimson => {
            // Crisp pixel-art Volt 'V' emblem centered on 10x16 back!
            let v_crimson: &[(usize, usize)] = &[
                (1, 3), (2, 3), (7, 3), (8, 3),
                (1, 4), (2, 4), (7, 4), (8, 4),
                (2, 5), (3, 5), (6, 5), (7, 5),
                (2, 6), (3, 6), (6, 6), (7, 6),
                (2, 7), (3, 7), (6, 7), (7, 7),
                (3, 8), (4, 8), (5, 8), (6, 8),
                (3, 9), (4, 9), (5, 9), (6, 9),
                (4, 10), (5, 10),
                (4, 11), (5, 11),
            ];
            let v_white: &[(usize, usize)] = &[
                (2, 4), (7, 4),
                (3, 6), (6, 6),
                (4, 9), (5, 9),
            ];
            for &(dx, dy) in v_crimson {
                set_px(1 + dx, 1 + dy, accent);
            }
            for &(dx, dy) in v_white {
                set_px(1 + dx, 1 + dy, highlight);
            }
        }
        PresetCape::RoyalGold => {
            // Pixel-art golden crown centered on 10x16 back
            let crown: &[(usize, usize)] = &[
                (2, 5), (4, 4), (5, 4), (7, 5),
                (2, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6),
                (2, 7), (3, 7), (4, 7), (5, 7), (6, 7), (7, 7),
                (2, 8), (3, 8), (4, 8), (5, 8), (6, 8), (7, 8),
                (2, 10), (3, 10), (4, 10), (5, 10), (6, 10), (7, 10),
            ];
            for &(dx, dy) in crown {
                set_px(1 + dx, 1 + dy, accent);
            }
            set_px(1 + 4, 1 + 6, highlight);
            set_px(1 + 5, 1 + 6, highlight);
        }
        PresetCape::ObsidianStealth => {
            // Subtle tactical chevron
            for i in 0..4 {
                set_px(1 + 1 + i, 1 + 6 + i, accent);
                set_px(1 + 8 - i, 1 + 6 + i, accent);
                set_px(1 + 1 + i, 1 + 9 + i, highlight);
                set_px(1 + 8 - i, 1 + 9 + i, highlight);
            }
        }
    }

    img
}
