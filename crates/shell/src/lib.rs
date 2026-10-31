// crates/shell/src/lib.rs

//! Shell du moteur Verbum.
//!
//! Ce crate héberge la boucle externe qui pilote le moteur : cadencement des
//! itérations (pas fixe provisoire), acquisition des événements externes et
//! appel de la fermeture d'infrastructure une fois par tick.
//!
//! Il est piloté par des fermetures uniquement : il n'expose aucun trait
//! public et aucune abstraction de la plateforme. Il ne nomme aucun type du
//! noyau et n'a aucune dépendance.
//!
//! État actuel (unité U4) : boucle bornée ([`run_loop`]), cadencement à pas fixe
//! **provisoire** ([`FixedStepPacing`]) et boîte de réception à capacité fixe
//! ([`Inbox`], [`DepositHandle`], [`InboxFull`]) dont la coupure alimente le lot
//! remis à l'infrastructure à chaque itération.
//!
//! # Ressource empruntant des octets (porte G-LT)
//!
//! Une ressource du vrai `World` peut emprunter des octets que l'appelant possède ;
//! le compilateur interdit alors de libérer ces octets tant que le monde vit. Les deux
//! doctests ci-dessous sont jumeaux : seule la ligne `drop(buffer)` les distingue.
//! Les volets positifs (LT-1, LT-2, LT-4) sont dans `tests/borrowed_resource.rs`.
//!
//! LT-3 : libérer les octets après le dépôt est refusé à la compilation (E0505).
//!
//! ```compile_fail
//! # use core::convert::Infallible;
//! # use core::marker::PhantomData;
//! #
//! # use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
//! # use verbum_schedule::access::{Locate, Produce, Read};
//! # use verbum_schedule::scheduler::Scheduler;
//! # use verbum_schedule::system::{At, Binding, Cons, Nil, System};
//! #
//! # /// Vue synthétique : une tranche empruntée plus un entier.
//! # #[derive(Clone, Copy)]
//! # struct View<'a> {
//! #     bytes: &'a [u8],
//! #     factor: u32,
//! # }
//! #
//! # impl View<'_> {
//! #     fn weighted_sum(&self) -> u32 {
//! #         self.bytes.iter().map(|&b| u32::from(b)).sum::<u32>() * self.factor
//! #     }
//! # }
//! #
//! # const VIEW: u8 = 0;
//! # const OUT: u8 = 1;
//! #
//! # #[derive(Default)]
//! # struct Rs<'a> {
//! #     view: ResourceSlot<View<'a>>,
//! #     out: ResourceSlot<u32>,
//! # }
//! #
//! # type Buffer = CommandBuffer<1, Infallible, Infallible>;
//! #
//! # struct Parts<'w, 'a> {
//! #     view: Option<&'w mut ResourceSlot<View<'a>>>,
//! #     out: Option<&'w mut ResourceSlot<u32>>,
//! # }
//! #
//! # impl<'w, 'a> Locate<'w, VIEW> for Parts<'w, 'a> {
//! #     type Store = &'w mut ResourceSlot<View<'a>>;
//! #     fn take(&mut self) -> Option<Self::Store> {
//! #         self.view.take()
//! #     }
//! # }
//! # impl<'w, 'a> Locate<'w, OUT> for Parts<'w, 'a> {
//! #     type Store = &'w mut ResourceSlot<u32>;
//! #     fn take(&mut self) -> Option<Self::Store> {
//! #         self.out.take()
//! #     }
//! # }
//! #
//! # struct B<'a>(PhantomData<&'a ()>);
//! #
//! # impl<'a> Binding for B<'a> {
//! #     type World = World<1, (), Rs<'a>>;
//! #     type Buffer = Buffer;
//! #     type Parts<'w>
//! #         = Parts<'w, 'a>
//! #     where
//! #         Self: 'w;
//! #     fn parts<'w>(world: &'w mut Self::World, _buffer: &'w mut Buffer) -> Parts<'w, 'a> {
//! #         let ((), rs) = world.split_mut();
//! #         Parts {
//! #             view: Some(&mut rs.view),
//! #             out: Some(&mut rs.out),
//! #         }
//! #     }
//! # }
//! #
//! # struct Sum;
//! #
//! # impl<'a> System<B<'a>> for Sum {
//! #     type Access<'w>
//! #         = (
//! #         Read<VIEW, <Parts<'w, 'a> as Locate<'w, VIEW>>::Store>,
//! #         Produce<OUT, <Parts<'w, 'a> as Locate<'w, OUT>>::Store>,
//! #     )
//! #     where
//! #         B<'a>: 'w;
//! #     fn run((view, mut out): Self::Access<'_>) -> Result<(), BufferFull> {
//! #         let Some(view) = view.get() else {
//! #             panic!("vue absente")
//! #         };
//! #         out.insert(view.weighted_sum());
//! #         Ok(())
//! #     }
//! # }
//! #
//! # type Systems = Cons<At<0, Sum>, Nil>;
//! #
//! let scheduler = Scheduler::<B<'_>, Systems, 1, 1>::new().expect("modèle valide");
//! let buffer: Vec<u8> = (1..=4).collect();
//! let mut world = World::<1, (), Rs<'_>>::new();
//! world.resources_mut().view.insert(View { bytes: &buffer, factor: 3 });
//! let mut cb = Buffer::new();
//! drop(buffer);
//! scheduler.run_tick(&mut world, &mut cb).expect("tick");
//! assert_eq!(*world.resources().out.get().expect("sortie"), 30);
//! ```
//!
//! LT-3t : jumeau valide, sans `drop` ; il compile et s'exécute.
//!
//! ```
//! # use core::convert::Infallible;
//! # use core::marker::PhantomData;
//! #
//! # use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
//! # use verbum_schedule::access::{Locate, Produce, Read};
//! # use verbum_schedule::scheduler::Scheduler;
//! # use verbum_schedule::system::{At, Binding, Cons, Nil, System};
//! #
//! # /// Vue synthétique : une tranche empruntée plus un entier.
//! # #[derive(Clone, Copy)]
//! # struct View<'a> {
//! #     bytes: &'a [u8],
//! #     factor: u32,
//! # }
//! #
//! # impl View<'_> {
//! #     fn weighted_sum(&self) -> u32 {
//! #         self.bytes.iter().map(|&b| u32::from(b)).sum::<u32>() * self.factor
//! #     }
//! # }
//! #
//! # const VIEW: u8 = 0;
//! # const OUT: u8 = 1;
//! #
//! # #[derive(Default)]
//! # struct Rs<'a> {
//! #     view: ResourceSlot<View<'a>>,
//! #     out: ResourceSlot<u32>,
//! # }
//! #
//! # type Buffer = CommandBuffer<1, Infallible, Infallible>;
//! #
//! # struct Parts<'w, 'a> {
//! #     view: Option<&'w mut ResourceSlot<View<'a>>>,
//! #     out: Option<&'w mut ResourceSlot<u32>>,
//! # }
//! #
//! # impl<'w, 'a> Locate<'w, VIEW> for Parts<'w, 'a> {
//! #     type Store = &'w mut ResourceSlot<View<'a>>;
//! #     fn take(&mut self) -> Option<Self::Store> {
//! #         self.view.take()
//! #     }
//! # }
//! # impl<'w, 'a> Locate<'w, OUT> for Parts<'w, 'a> {
//! #     type Store = &'w mut ResourceSlot<u32>;
//! #     fn take(&mut self) -> Option<Self::Store> {
//! #         self.out.take()
//! #     }
//! # }
//! #
//! # struct B<'a>(PhantomData<&'a ()>);
//! #
//! # impl<'a> Binding for B<'a> {
//! #     type World = World<1, (), Rs<'a>>;
//! #     type Buffer = Buffer;
//! #     type Parts<'w>
//! #         = Parts<'w, 'a>
//! #     where
//! #         Self: 'w;
//! #     fn parts<'w>(world: &'w mut Self::World, _buffer: &'w mut Buffer) -> Parts<'w, 'a> {
//! #         let ((), rs) = world.split_mut();
//! #         Parts {
//! #             view: Some(&mut rs.view),
//! #             out: Some(&mut rs.out),
//! #         }
//! #     }
//! # }
//! #
//! # struct Sum;
//! #
//! # impl<'a> System<B<'a>> for Sum {
//! #     type Access<'w>
//! #         = (
//! #         Read<VIEW, <Parts<'w, 'a> as Locate<'w, VIEW>>::Store>,
//! #         Produce<OUT, <Parts<'w, 'a> as Locate<'w, OUT>>::Store>,
//! #     )
//! #     where
//! #         B<'a>: 'w;
//! #     fn run((view, mut out): Self::Access<'_>) -> Result<(), BufferFull> {
//! #         let Some(view) = view.get() else {
//! #             panic!("vue absente")
//! #         };
//! #         out.insert(view.weighted_sum());
//! #         Ok(())
//! #     }
//! # }
//! #
//! # type Systems = Cons<At<0, Sum>, Nil>;
//! #
//! let scheduler = Scheduler::<B<'_>, Systems, 1, 1>::new().expect("modèle valide");
//! let buffer: Vec<u8> = (1..=4).collect();
//! let mut world = World::<1, (), Rs<'_>>::new();
//! world.resources_mut().view.insert(View { bytes: &buffer, factor: 3 });
//! let mut cb = Buffer::new();
//! scheduler.run_tick(&mut world, &mut cb).expect("tick");
//! assert_eq!(*world.resources().out.get().expect("sortie"), 30);
//! ```
//!
//! # Carte réelle empruntant des octets (porte G-LT sur `Map<'a>`)
//!
//! Même montage que ci-dessus avec le véritable `Map<'a>` déposé par `load` : la
//! ressource emprunte les octets de l'artefact, possédés par l'appelant. LT-5n et LT-5t
//! sont jumeaux : seule la ligne `drop(map_bytes)` les distingue. Le volet d'exécution
//! (LT-5) est dans `tests/borrowed_resource.rs`.
//!
//! LT-5n : libérer les octets de la carte après le dépôt est refusé à la compilation.
//!
//! ```compile_fail
//! # use core::convert::Infallible;
//! # use core::marker::PhantomData;
//! #
//! # use verbum_assets::{Asset, Format, Map, load};
//! # use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
//! # use verbum_schedule::access::{Locate, Produce, Read};
//! # use verbum_schedule::scheduler::Scheduler;
//! # use verbum_schedule::system::{At, Binding, Cons, Nil, System};
//! #
//! # const MAP: u8 = 0;
//! # const OUT: u8 = 1;
//! #
//! # #[derive(Default)]
//! # struct Rs<'a> {
//! #     map: ResourceSlot<Map<'a>>,
//! #     out: ResourceSlot<u32>,
//! # }
//! #
//! # type Buffer = CommandBuffer<1, Infallible, Infallible>;
//! #
//! # struct Parts<'w, 'a> {
//! #     map: Option<&'w mut ResourceSlot<Map<'a>>>,
//! #     out: Option<&'w mut ResourceSlot<u32>>,
//! # }
//! #
//! # impl<'w, 'a> Locate<'w, MAP> for Parts<'w, 'a> {
//! #     type Store = &'w mut ResourceSlot<Map<'a>>;
//! #     fn take(&mut self) -> Option<Self::Store> {
//! #         self.map.take()
//! #     }
//! # }
//! #
//! # impl<'w, 'a> Locate<'w, OUT> for Parts<'w, 'a> {
//! #     type Store = &'w mut ResourceSlot<u32>;
//! #     fn take(&mut self) -> Option<Self::Store> {
//! #         self.out.take()
//! #     }
//! # }
//! #
//! # struct B<'a>(PhantomData<&'a ()>);
//! #
//! # impl<'a> Binding for B<'a> {
//! #     type World = World<1, (), Rs<'a>>;
//! #     type Buffer = Buffer;
//! #     type Parts<'w>
//! #         = Parts<'w, 'a>
//! #     where
//! #         Self: 'w;
//! #     fn parts<'w>(world: &'w mut Self::World, _buffer: &'w mut Buffer) -> Parts<'w, 'a> {
//! #         let ((), rs) = world.split_mut();
//! #         Parts {
//! #             map: Some(&mut rs.map),
//! #             out: Some(&mut rs.out),
//! #         }
//! #     }
//! # }
//! #
//! # /// Somme des indices de toutes les cases de la carte lue.
//! # fn checksum(map: &Map<'_>) -> u32 {
//! #     let mut sum = 0u32;
//! #     for row in 0..map.height() {
//! #         for column in 0..map.width() {
//! #             sum += u32::from(map.tile(column, row).unwrap_or(0));
//! #         }
//! #     }
//! #     sum
//! # }
//! #
//! # struct Sum;
//! #
//! # impl<'a> System<B<'a>> for Sum {
//! #     type Access<'w>
//! #         = (
//! #         Read<MAP, <Parts<'w, 'a> as Locate<'w, MAP>>::Store>,
//! #         Produce<OUT, <Parts<'w, 'a> as Locate<'w, OUT>>::Store>,
//! #     )
//! #     where
//! #         B<'a>: 'w;
//! #     fn run((map, mut out): Self::Access<'_>) -> Result<(), BufferFull> {
//! #         let Some(map) = map.get() else {
//! #             panic!("carte absente")
//! #         };
//! #         out.insert(checksum(map));
//! #         Ok(())
//! #     }
//! # }
//! #
//! # type Systems = Cons<At<0, Sum>, Nil>;
//! #
//! # /// Manifeste littéral : tileset (artefact 0, slot 0), carte (artefact 1, slot 1).
//! # const MANIFEST: [u8; 42] = [
//! #     0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // en-tête, n = 2
//! #     0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id du tileset
//! #     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // artefact 0, slot 0
//! #     0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id de la carte
//! #     0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // artefact 1, slot 1
//! # ];
//! #
//! # /// Tileset littéral (16 × 8, 4 colonnes, 12 tuiles).
//! # const TILESET: [u8; 18] = [
//! #     0x56, 0x54, 0x4C, 0x53, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x08, 0x00, 0x04, 0x00,
//! #     0x0C, 0x00,
//! # ];
//! #
//! # /// Carte 3 × 2 littérale (MAP-R1) référençant le tileset ci-dessus ; indices 0x0001,
//! # /// 0x0203, 0x0405, 0x0607, 0x0809, 0x0A0B, de somme 7716.
//! # const MAP_R1: [u8; 34] = [
//! #     0x56, 0x4D, 0x41, 0x50, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, // préfixe, charge 24
//! #     0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // identifiant du tileset
//! #     0x03, 0x00, 0x02, 0x00, // 3 × 2
//! #     0x01, 0x00, 0x03, 0x02, 0x05, 0x04, 0x07, 0x06, 0x09, 0x08, 0x0B, 0x0A,
//! # ];
//! #
//! let scheduler = Scheduler::<B<'_>, Systems, 1, 1>::new().expect("modèle valide");
//! let tileset_bytes = TILESET.to_vec();
//! let map_bytes = MAP_R1.to_vec();
//! let mut world = World::<1, (), Rs<'_>>::new();
//! load(
//!     &MANIFEST,
//!     &[tileset_bytes.as_slice(), map_bytes.as_slice()],
//!     &[Format::Tileset, Format::Map],
//!     |_slot, asset| {
//!         if let Asset::Map(map) = asset {
//!             world.resources_mut().map.insert(map);
//!         }
//!     },
//! )
//! .expect("chargement");
//! let mut cb = Buffer::new();
//! drop(map_bytes);
//! scheduler.run_tick(&mut world, &mut cb).expect("tick");
//! assert_eq!(*world.resources().out.get().expect("sortie"), 7716);
//! ```
//!
//! LT-5t : jumeau valide, sans `drop` ; il compile et s'exécute.
//!
//! ```
//! # use core::convert::Infallible;
//! # use core::marker::PhantomData;
//! #
//! # use verbum_assets::{Asset, Format, Map, load};
//! # use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
//! # use verbum_schedule::access::{Locate, Produce, Read};
//! # use verbum_schedule::scheduler::Scheduler;
//! # use verbum_schedule::system::{At, Binding, Cons, Nil, System};
//! #
//! # const MAP: u8 = 0;
//! # const OUT: u8 = 1;
//! #
//! # #[derive(Default)]
//! # struct Rs<'a> {
//! #     map: ResourceSlot<Map<'a>>,
//! #     out: ResourceSlot<u32>,
//! # }
//! #
//! # type Buffer = CommandBuffer<1, Infallible, Infallible>;
//! #
//! # struct Parts<'w, 'a> {
//! #     map: Option<&'w mut ResourceSlot<Map<'a>>>,
//! #     out: Option<&'w mut ResourceSlot<u32>>,
//! # }
//! #
//! # impl<'w, 'a> Locate<'w, MAP> for Parts<'w, 'a> {
//! #     type Store = &'w mut ResourceSlot<Map<'a>>;
//! #     fn take(&mut self) -> Option<Self::Store> {
//! #         self.map.take()
//! #     }
//! # }
//! #
//! # impl<'w, 'a> Locate<'w, OUT> for Parts<'w, 'a> {
//! #     type Store = &'w mut ResourceSlot<u32>;
//! #     fn take(&mut self) -> Option<Self::Store> {
//! #         self.out.take()
//! #     }
//! # }
//! #
//! # struct B<'a>(PhantomData<&'a ()>);
//! #
//! # impl<'a> Binding for B<'a> {
//! #     type World = World<1, (), Rs<'a>>;
//! #     type Buffer = Buffer;
//! #     type Parts<'w>
//! #         = Parts<'w, 'a>
//! #     where
//! #         Self: 'w;
//! #     fn parts<'w>(world: &'w mut Self::World, _buffer: &'w mut Buffer) -> Parts<'w, 'a> {
//! #         let ((), rs) = world.split_mut();
//! #         Parts {
//! #             map: Some(&mut rs.map),
//! #             out: Some(&mut rs.out),
//! #         }
//! #     }
//! # }
//! #
//! # /// Somme des indices de toutes les cases de la carte lue.
//! # fn checksum(map: &Map<'_>) -> u32 {
//! #     let mut sum = 0u32;
//! #     for row in 0..map.height() {
//! #         for column in 0..map.width() {
//! #             sum += u32::from(map.tile(column, row).unwrap_or(0));
//! #         }
//! #     }
//! #     sum
//! # }
//! #
//! # struct Sum;
//! #
//! # impl<'a> System<B<'a>> for Sum {
//! #     type Access<'w>
//! #         = (
//! #         Read<MAP, <Parts<'w, 'a> as Locate<'w, MAP>>::Store>,
//! #         Produce<OUT, <Parts<'w, 'a> as Locate<'w, OUT>>::Store>,
//! #     )
//! #     where
//! #         B<'a>: 'w;
//! #     fn run((map, mut out): Self::Access<'_>) -> Result<(), BufferFull> {
//! #         let Some(map) = map.get() else {
//! #             panic!("carte absente")
//! #         };
//! #         out.insert(checksum(map));
//! #         Ok(())
//! #     }
//! # }
//! #
//! # type Systems = Cons<At<0, Sum>, Nil>;
//! #
//! # /// Manifeste littéral : tileset (artefact 0, slot 0), carte (artefact 1, slot 1).
//! # const MANIFEST: [u8; 42] = [
//! #     0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // en-tête, n = 2
//! #     0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id du tileset
//! #     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // artefact 0, slot 0
//! #     0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id de la carte
//! #     0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // artefact 1, slot 1
//! # ];
//! #
//! # /// Tileset littéral (16 × 8, 4 colonnes, 12 tuiles).
//! # const TILESET: [u8; 18] = [
//! #     0x56, 0x54, 0x4C, 0x53, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x08, 0x00, 0x04, 0x00,
//! #     0x0C, 0x00,
//! # ];
//! #
//! # /// Carte 3 × 2 littérale (MAP-R1) référençant le tileset ci-dessus ; indices 0x0001,
//! # /// 0x0203, 0x0405, 0x0607, 0x0809, 0x0A0B, de somme 7716.
//! # const MAP_R1: [u8; 34] = [
//! #     0x56, 0x4D, 0x41, 0x50, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, // préfixe, charge 24
//! #     0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // identifiant du tileset
//! #     0x03, 0x00, 0x02, 0x00, // 3 × 2
//! #     0x01, 0x00, 0x03, 0x02, 0x05, 0x04, 0x07, 0x06, 0x09, 0x08, 0x0B, 0x0A,
//! # ];
//! #
//! let scheduler = Scheduler::<B<'_>, Systems, 1, 1>::new().expect("modèle valide");
//! let tileset_bytes = TILESET.to_vec();
//! let map_bytes = MAP_R1.to_vec();
//! let mut world = World::<1, (), Rs<'_>>::new();
//! load(
//!     &MANIFEST,
//!     &[tileset_bytes.as_slice(), map_bytes.as_slice()],
//!     &[Format::Tileset, Format::Map],
//!     |_slot, asset| {
//!         if let Asset::Map(map) = asset {
//!             world.resources_mut().map.insert(map);
//!         }
//!     },
//! )
//! .expect("chargement");
//! let mut cb = Buffer::new();
//! scheduler.run_tick(&mut world, &mut cb).expect("tick");
//! assert_eq!(*world.resources().out.get().expect("sortie"), 7716);
//! ```

mod inbox;
mod pacing;
mod shell;

pub use inbox::{DepositHandle, Inbox, InboxFull};
pub use pacing::FixedStepPacing;
pub use shell::run_loop;
