use anyhow::{Context, Result};
use hyprland::
    keyword::{Keyword, OptionValue}
;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_appconfig_columns() -> u16 {
    4
}
fn default_appconfig_rows() -> u16 {
    2
}
fn default_appconfig_keeb() -> Vec<Vec<String>> {
    Vec::from([
        "QWERTYUIOP".chars().map(|c| c.to_string()).collect(),
        "ASDFGHJKL;".chars().map(|c| c.to_string()).collect(),
        "ZXCVBNM,.".chars().map(|c| c.to_string()).collect(),
    ])
}

fn default_appconfig_border_width() -> u16 {
    let border_width: u16 = match Keyword::get("general:border_size")
        .expect("Failed to get hyprland border settings")
        .value
    {
        OptionValue::Int(border) => border as u16,
        OptionValue::Float(border) => border as u16,
        _ => 5,
    };
    border_width
}

fn default_appconfig_margin() -> u16 {
    15
}

fn default_appconfig_waybar_height() -> u16 {
    48
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_appconfig_columns")]
    pub columns: u16,
    #[serde(default = "default_appconfig_rows")]
    pub rows: u16,
    #[serde(default = "default_appconfig_keeb")]
    pub keeb: Vec<Vec<String>>,
    #[serde(default = "default_appconfig_border_width")]
    pub border_width: u16,
    #[serde(default = "default_appconfig_margin")]
    pub margin: u16,
    #[serde(default = "default_appconfig_waybar_height")]
    pub waybar_height: u16,
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            columns: default_appconfig_columns(),
            rows: default_appconfig_rows(),
            keeb: default_appconfig_keeb(),
            border_width: default_appconfig_border_width(),
            margin: default_appconfig_margin(),
            waybar_height: default_appconfig_waybar_height(),
            config_path: None,
        }
    }
}

impl AppConfig {
    fn bootstrap() -> Result<PathBuf> {
        let homedir = Self::calc_config_dir()?;
        if homedir.is_file() {
            panic!(
                "Configuration dir is a file instead of a dir. {:?}",
                &homedir
            );
        }
        if !homedir.exists() {

            std::fs::create_dir_all(&homedir)?;
        }

        let mut cfgpath = homedir.clone();
        cfgpath.push("hypr-gridtile.ron");
        if !cfgpath.is_file() {
            let tmpconfig = AppConfig::default();
            // let cfgstr = ron::ser::to_string_pretty(&tmpconfig, ron::ser::PrettyConfig::default())
            //     .expect("Wups, my default config is borked?!");
            // std::fs::write(&cfgpath, cfgstr.as_bytes())?
            tmpconfig.save()?
        }

        Ok(cfgpath)
    }

    fn calc_config_dir() -> Result<PathBuf> {
        let mut homedir = dirs::home_dir().context("Failed to read $HOMEDIR from env.")?;
        homedir.push(".config");
        homedir.push("hypr-gridtile");
        Ok(homedir)
    }

    pub fn save(&self) -> Result<()> {
        let cfgpath = match self.config_path.clone(){
            Some(cfgpath) => cfgpath,
            None => Self::bootstrap()?,
        };
        let var_name = ron::ser::PrettyConfig::default();
        let cfgstr = ron::ser::to_string_pretty(self, var_name)?;
        std::fs::write(cfgpath, cfgstr.as_bytes())?;
        Ok(())
    }

    pub fn load() -> Result<AppConfig> {
        let cfgpath = match Self::bootstrap() {
            Ok(cfgpath) => cfgpath,
            Err(_) => Self::bootstrap()?,
        };
        let cfgfile = std::fs::read_to_string(&cfgpath)?;
        let mut cfg: AppConfig = ron::de::from_bytes(cfgfile.as_bytes())?;
        cfg.config_path = Some(cfgpath.clone());
        Ok(cfg)
    }
}
