// crates/shell/src/inbox.rs

//! Boîte de réception des événements externes.
//!
//! La boîte vit côté Plateforme, jamais dans le `World`. Sa capacité est fixée
//! à la construction et ne change jamais. Des poignées de dépôt clonables
//! ([`DepositHandle`]) y déposent des événements ; la boucle prélève
//! l'ensemble du contenu à la coupure ([`Inbox::cut`]) dans un lot de même
//! capacité alloué à la construction.
//!
//! # Atomicité de la coupure
//!
//! Dépôts et prélèvement sont sérialisés par un `Mutex` : chaque dépôt est
//! entièrement avant ou entièrement après la coupure. Le verrou est libéré
//! **avant** que le lot soit remis à l'infrastructure, de sorte qu'un dépôt
//! effectué pendant un tick est accepté sans interblocage et appartient à une
//! itération ultérieure.
//!
//! # Allocation
//!
//! Les allocations n'existent qu'à la construction. Ni le dépôt ni la coupure
//! n'allouent ni ne font croître une collection.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Contenu partagé : événements acceptés, dans l'ordre d'acceptation.
#[derive(Debug)]
struct Content<Ev> {
    items: Vec<Ev>,
    limit: usize,
}

/// Prend le verrou. Un verrou empoisonné est repris : les sections critiques
/// ci-dessous ne contiennent aucun chemin pouvant paniquer et laissent le
/// contenu cohérent à tout point, l'empoisonnement ne peut donc pas révéler
/// d'état partiel.
fn lock<Ev>(shared: &Mutex<Content<Ev>>) -> MutexGuard<'_, Content<Ev>> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Erreur de dépôt : la boîte est saturée. Rien n'a été modifié.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InboxFull;

impl std::fmt::Display for InboxFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("boîte de réception saturée")
    }
}

impl std::error::Error for InboxFull {}

/// Boîte de réception à capacité fixe, détenue par la boucle.
#[derive(Debug)]
pub struct Inbox<Ev: Copy> {
    shared: Arc<Mutex<Content<Ev>>>,
    batch: Vec<Ev>,
}

/// Poignée de dépôt clonable vers une [`Inbox`].
#[derive(Debug)]
pub struct DepositHandle<Ev: Copy> {
    shared: Arc<Mutex<Content<Ev>>>,
}

impl<Ev: Copy> Clone for DepositHandle<Ev> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<Ev: Copy> Inbox<Ev> {
    /// Construit une boîte de capacité `capacity`, avec son lot de même
    /// capacité. Seule étape qui alloue.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Content {
                items: Vec::with_capacity(capacity),
                limit: capacity,
            })),
            batch: Vec::with_capacity(capacity),
        }
    }

    /// Capacité fixée à la construction.
    #[must_use]
    pub fn capacity(&self) -> usize {
        lock(&self.shared).limit
    }

    /// Rend une poignée de dépôt (clonable) vers cette boîte.
    #[must_use]
    pub fn handle(&self) -> DepositHandle<Ev> {
        DepositHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Coupure : prélève **tout** le contenu dans le lot préalloué, dans
    /// l'ordre d'acceptation, vide la boîte, puis rend le lot.
    ///
    /// Un seul passage sous verrou ; le verrou est libéré au retour, avant
    /// toute invocation de l'infrastructure. Le lot est vide si rien n'a été
    /// déposé.
    pub fn cut(&mut self) -> &[Ev] {
        {
            let mut content = lock(&self.shared);
            // Copie complète dans le lot préalloué (`batch.capacity() >= limit`
            // par construction, donc sans allocation), puis vidage.
            self.batch.clear();
            self.batch.extend_from_slice(&content.items);
            content.items.clear();
        } // Verrou libéré ici, avant tout appel à l'infrastructure.
        &self.batch
    }
}

impl<Ev: Copy> DepositHandle<Ev> {
    /// Dépose un événement.
    ///
    /// # Erreurs
    ///
    /// [`InboxFull`] si la boîte est saturée ; rien n'est alors modifié.
    pub fn push(&self, event: Ev) -> Result<(), InboxFull> {
        let mut content = lock(&self.shared);
        if content.items.len() >= content.limit {
            return Err(InboxFull);
        }
        // `len < limit <= capacity` : le `push` n'alloue pas.
        content.items.push(event);
        Ok(())
    }
}
