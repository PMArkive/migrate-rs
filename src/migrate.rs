use crate::store::Store;
use crate::Error;
use demostf_client::{ApiClient, Demo, ListOrder, ListParams};
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::time::timeout;
use tracing::{info, instrument, warn};

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

    pub async fn migrate_till(
        &self,
        from_backend: &str,
        time: OffsetDateTime,
    ) -> Result<(), Error> {
        let demos = self
            .client
            .list(
                ListParams::default()
                    .with_order(ListOrder::Ascending)
                    .with_backend(from_backend)
                    .with_before(time),
                1,
            )
            .await?;

        for demo in demos {
            assert!(demo.time < time);
            self.migrate(&demo).await?;
        }

        Ok(())
    }

    #[instrument(skip(demo), fields(demo.id = demo.id))]
    pub async fn migrate(&self, demo: &Demo) -> Result<(), Error> {
        let name = demo.url.rsplit('/').next().unwrap();

        let stored_hash = self.store.hash(name)?;
        if stored_hash != Some(demo.hash) {
            warn!(
                stored_hash = debug(stored_hash),
                expected_hash = debug(demo.hash),
                "hash mismatch"
            );
            self.re_download(name, demo).await?;
        }

        info!("updating demo link");

        self.client
            .set_url(
                demo.id,
                &self.backend,
                self.store.generate_path(name).to_str().unwrap(),
                &self.store.generate_url(name),
                demo.hash,
                &self.key,
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(demo), fields(demo.id = demo.id, demo.name = name))]
    async fn re_download(&self, name: &str, demo: &Demo) -> Result<(), Error> {
        let mut data = Vec::with_capacity(demo.duration as usize / 60 * 1024);

        timeout(
            Duration::from_secs(5 * 60),
            demo.save(&self.client, &mut data),
        )
        .await
        .map_err(|_| Error::Timeout)??;

        self.store.remove(name)?;
        let mut file = self.store.create(name).await?;

        file.write_all(&data)?;

        Ok(())
    }
}
