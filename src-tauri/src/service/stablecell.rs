use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce, aead::Aead};
use ciborium::{from_reader, into_writer};
use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use tauri::{
    Manager, RunEvent, Runtime,
    plugin::{Builder, TauriPlugin},
};

use crate::{Result, utils::rand_bytes};

pub struct PlainCell<T>
where
    T: Serialize + DeserializeOwned,
{
    path: PathBuf,
    value: RwLock<T>,
}

impl<T> PlainCell<T>
where
    T: Serialize + DeserializeOwned + Default + Send + Sync + 'static,
{
    const NAME: &'static str = "plain-cell";

    pub fn init<R>(file: PathBuf) -> TauriPlugin<R>
    where
        R: Runtime,
    {
        Builder::new(Self::NAME)
            .setup(move |app, _api| {
                let app_data_dir = app
                    .path()
                    .app_local_data_dir()
                    .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
                let path = app_data_dir.join(file);
                let cell = PlainCell::<T>::load(path.clone())?;
                app.manage(cell);
                log::info!("Initialized {} at {:?}", Self::NAME, path);
                Ok(())
            })
            .on_event(|app, event| {
                if let RunEvent::Exit = event {
                    // app is going to exit, you can cleanup here
                    let cell = app.state::<PlainCell<T>>();

                    if let Err(err) = cell.save() {
                        log::error!(
                            path = format!("{:?}", cell.path.display());
                            "Failed to save {}: {err:?}", Self::NAME,
                        );
                    };
                }
            })
            .build()
    }

    fn load(path: PathBuf) -> Result<Self> {
        match fs::read(&path) {
            Ok(data) => {
                let value = from_reader(&data[..]).map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to deserialize cell: {err:?}"),
                    )
                })?;

                Ok(PlainCell {
                    path,
                    value: RwLock::new(value),
                })
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                // If the file does not exist, create a new one
                let value = T::default();
                let cell = PlainCell {
                    path,
                    value: RwLock::new(value),
                };

                cell.save()?;
                Ok(cell)
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.value.read())
    }

    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.value.write())
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        } else {
            return Err("Invalid cell path: no parent directory".into());
        }

        let mut data = Vec::new();
        into_writer(&*self.value.read(), &mut data)?;
        fs::write(&self.path, data)?;
        Ok(())
    }
}

pub struct CipherCell<T>
where
    T: Serialize + DeserializeOwned,
{
    path: PathBuf,
    cipher: Aes256Gcm,
    value: RwLock<T>,
}

impl<T> CipherCell<T>
where
    T: Serialize + DeserializeOwned + Default + Send + Sync + 'static,
{
    const NAME: &'static str = "cipher-cell";

    pub fn init<R>(file: PathBuf, secret: [u8; 32]) -> TauriPlugin<R>
    where
        R: Runtime,
    {
        Builder::new(Self::NAME)
            .setup(move |app, _api| {
                let app_data_dir = app
                    .path()
                    .app_local_data_dir()
                    .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
                let path = app_data_dir.join(file);
                let cell = CipherCell::<T>::load(path.clone(), secret)?;
                app.manage(cell);
                log::info!("Initialized {} at {:?}", Self::NAME, path);
                Ok(())
            })
            .on_event(|app, event| {
                if let RunEvent::Exit = event {
                    // app is going to exit, you can cleanup here
                    let cell = app.state::<CipherCell<T>>();

                    if let Err(err) = cell.save() {
                        log::error!(
                            path = format!("{:?}", cell.path.display());
                            "Failed to save {}: {err:?}", Self::NAME,
                        );
                    };
                }
            })
            .build()
    }

    fn load(path: PathBuf, secret: [u8; 32]) -> Result<Self> {
        let key = Key::<Aes256Gcm>::from(secret);
        let cipher = Aes256Gcm::new(&key);
        match fs::read(&path) {
            Ok(data) => {
                if data.len() < 28 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid file data length",
                    )
                    .into());
                }

                let nonce = Nonce::from_slice(&data[..12]);
                let data = cipher.decrypt(nonce, data[12..].as_ref()).map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to decrypt cell value: {err:?}"),
                    )
                })?;

                let value = from_reader(&data[..]).map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to deserialize cell value: {err:?}"),
                    )
                })?;

                Ok(CipherCell {
                    path,
                    cipher,
                    value: RwLock::new(value),
                })
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                // If the file does not exist, create a new one
                let value = T::default();
                let cell = CipherCell {
                    path,
                    cipher,
                    value: RwLock::new(value),
                };

                cell.save()?;
                Ok(cell)
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.value.read())
    }

    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.value.write())
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        } else {
            return Err("Invalid cell path: no parent directory".into());
        }

        let mut data = Vec::new();
        into_writer(&*self.value.read(), &mut data)?;
        let nonce = rand_bytes::<12>();
        let encrypted_data = self
            .cipher
            .encrypt(Nonce::from_slice(&nonce), data.as_ref())
            .map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to encrypt cell value: {err:?}"),
                )
            })?;

        let mut result = Vec::with_capacity(nonce.len() + encrypted_data.len());
        result.extend(nonce);
        result.extend(encrypted_data);
        fs::write(&self.path, result)?;
        Ok(())
    }
}
