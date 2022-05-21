use crate::store::Store;
use crate::Error;
use demostf_client::{ApiClient, Demo};
use std::fmt::{Debug, Formatter};
use std::io::Write;
use tracing::{instrument, warn};

pub struct Migrator {
    store: Store,
    client: ApiClient,
    backend: String,
    key: String,
}

impl Debug for Migrator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Migrator")
            .field("store", &self.store)
            .field("client", &self.client)
            .field("backend", &self.backend)
            .finish_non_exhaustive()
    }
}

impl Migrator {
    pub fn new(store: Store, client: ApiClient, backend: String, key: String) -> Self {
        Self {
            store,
            client,
            backend,
            key,
        }
    }

    #[instrument(skip(demo), fields(demo.id = demo.id))]
    pub async fn migrate(&self, demo: &Demo) -> Result<(), Error> {
        let name = demo.url.rsplit('/').next().unwrap();

        let stored_hash = self.store.hash(name)?;
        if stored_hash != demo.hash {
            warn!(
                stored_hash = debug(stored_hash),
                expected_hash = debug(demo.hash),
                "hash mismatch"
            );
            self.re_download(name, demo).await?;
        }

        self.client
            .set_url(
                demo.id,
                &self.backend,
                &self.store.generate_path(name).to_str().unwrap(),
                &self.store.generate_url(name),
                stored_hash,
                &self.key,
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(demo), fields(demo.id = demo.id, demo.name = name))]
    async fn re_download(&self, name: &str, demo: &Demo) -> Result<(), Error> {
        let mut data = Vec::with_capacity(demo.duration as usize / 60 * 1024);
        demo.save(&self.client, &mut data).await?;

        self.store.remove(name)?;
        let mut file = self.store.create(name).await?;

        file.write_all(&data)?;

        Ok(())
    }
}
