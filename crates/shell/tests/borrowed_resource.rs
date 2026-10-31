// crates/shell/tests/borrowed_resource.rs

//! Porte G-LT, volet positif : une ressource du vrai `World` emprunte des octets
//! construits à l'exécution et un vrai système les lit sous `Scheduler::run_tick`.
//!
//! - LT-1 : le montage compile et s'exécute sous le vrai ordonnanceur ;
//! - LT-2 : la sortie dépend des octets empruntés (deux tampons distincts) ;
//! - LT-4 : le cas `'static` compile aussi ; il n'est **pas** une preuve G-LT.
//!
//! Le volet négatif (LT-3, refus du `drop` prématuré) et son jumeau valide (LT-3t)
//! sont des doctests de `crates/shell/src/lib.rs`.
//!
//! La vue est synthétique : en U1, aucun type d'actif n'emprunte d'octets.
use core::convert::Infallible;
use core::marker::PhantomData;

use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
use verbum_schedule::access::{Locate, Produce, Read};
use verbum_schedule::scheduler::Scheduler;
use verbum_schedule::system::{At, Binding, Cons, Nil, System};

/// Vue synthétique : une tranche empruntée plus un entier.
#[derive(Clone, Copy)]
struct View<'a> {
    bytes: &'a [u8],
    factor: u32,
}

impl View<'_> {
    fn weighted_sum(&self) -> u32 {
        self.bytes.iter().map(|&b| u32::from(b)).sum::<u32>() * self.factor
    }
}

const VIEW: u8 = 0;
const OUT: u8 = 1;

#[derive(Default)]
struct Rs<'a> {
    view: ResourceSlot<View<'a>>,
    out: ResourceSlot<u32>,
}

type Buffer = CommandBuffer<1, Infallible, Infallible>;

struct Parts<'w, 'a> {
    view: Option<&'w mut ResourceSlot<View<'a>>>,
    out: Option<&'w mut ResourceSlot<u32>>,
}

impl<'w, 'a> Locate<'w, VIEW> for Parts<'w, 'a> {
    type Store = &'w mut ResourceSlot<View<'a>>;
    fn take(&mut self) -> Option<Self::Store> {
        self.view.take()
    }
}
impl<'w, 'a> Locate<'w, OUT> for Parts<'w, 'a> {
    type Store = &'w mut ResourceSlot<u32>;
    fn take(&mut self) -> Option<Self::Store> {
        self.out.take()
    }
}

struct B<'a>(PhantomData<&'a ()>);

impl<'a> Binding for B<'a> {
    type World = World<1, (), Rs<'a>>;
    type Buffer = Buffer;
    type Parts<'w>
        = Parts<'w, 'a>
    where
        Self: 'w;
    fn parts<'w>(world: &'w mut Self::World, _buffer: &'w mut Buffer) -> Parts<'w, 'a> {
        let ((), rs) = world.split_mut();
        Parts {
            view: Some(&mut rs.view),
            out: Some(&mut rs.out),
        }
    }
}

struct Sum;

impl<'a> System<B<'a>> for Sum {
    type Access<'w>
        = (
        Read<VIEW, <Parts<'w, 'a> as Locate<'w, VIEW>>::Store>,
        Produce<OUT, <Parts<'w, 'a> as Locate<'w, OUT>>::Store>,
    )
    where
        B<'a>: 'w;
    fn run((view, mut out): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(view) = view.get() else {
            panic!("vue absente")
        };
        out.insert(view.weighted_sum());
        Ok(())
    }
}

type Systems = Cons<At<0, Sum>, Nil>;

fn run_ticks(bytes: &[u8], ticks: usize) -> u32 {
    let scheduler = Scheduler::<B<'_>, Systems, 1, 1>::new().expect("modèle valide");
    let mut world = World::<1, (), Rs<'_>>::new();
    world.resources_mut().view.insert(View { bytes, factor: 3 });
    let mut buffer = Buffer::new();
    for _ in 0..ticks {
        scheduler.run_tick(&mut world, &mut buffer).expect("tick");
    }
    *world.resources().out.get().expect("sortie")
}

fn runtime_buffer(seed: u8) -> Vec<u8> {
    (0..4u8).map(|i| i + seed).collect()
}

#[test]
fn lt1_the_borrowed_view_runs_under_the_real_scheduler() {
    let buffer = runtime_buffer(1); // [1,2,3,4]
    assert_eq!(run_ticks(&buffer, 3), 30);
}

#[test]
fn lt2_the_system_reads_the_borrowed_bytes_not_a_constant_copy() {
    let first = runtime_buffer(1);
    let second = runtime_buffer(2); // [2,3,4,5]
    assert_eq!(run_ticks(&first, 1), 30);
    assert_eq!(run_ticks(&second, 1), 42);
}

/// Exécution avec des octets `'static` : `B<'static>`, `Rs<'static>` explicites.
fn run_static(bytes: &'static [u8]) -> u32 {
    let scheduler = Scheduler::<B<'static>, Systems, 1, 1>::new().expect("modèle valide");
    let mut world = World::<1, (), Rs<'static>>::new();
    world.resources_mut().view.insert(View { bytes, factor: 3 });
    let mut buffer = Buffer::new();
    scheduler.run_tick(&mut world, &mut buffer).expect("tick");
    *world.resources().out.get().expect("sortie")
}

/// LT-4 : cas `'static`. Les octets sont un littéral promu ; ce test montre seulement
/// que les bornes généralisées acceptent aussi le cas sans emprunt réel. Il n'est
/// **pas** une preuve G-LT.
#[test]
fn lt4_the_static_case_compiles_and_is_not_a_g_lt_proof() {
    assert_eq!(run_static(&[1, 2, 3, 4]), 30);
}

/// LT-5 : le même montage sur le véritable `Map<'a>`, déposé par le véritable `load`.
mod real_map {
    use core::convert::Infallible;
    use core::marker::PhantomData;

    use verbum_assets::{Asset, Format, Map, load};
    use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
    use verbum_schedule::access::{Locate, Produce, Read};
    use verbum_schedule::scheduler::Scheduler;
    use verbum_schedule::system::{At, Binding, Cons, Nil, System};

    const MAP: u8 = 0;
    const OUT: u8 = 1;

    #[derive(Default)]
    struct Rs<'a> {
        map: ResourceSlot<Map<'a>>,
        out: ResourceSlot<u32>,
    }

    type Buffer = CommandBuffer<1, Infallible, Infallible>;

    struct Parts<'w, 'a> {
        map: Option<&'w mut ResourceSlot<Map<'a>>>,
        out: Option<&'w mut ResourceSlot<u32>>,
    }

    impl<'w, 'a> Locate<'w, MAP> for Parts<'w, 'a> {
        type Store = &'w mut ResourceSlot<Map<'a>>;
        fn take(&mut self) -> Option<Self::Store> {
            self.map.take()
        }
    }

    impl<'w, 'a> Locate<'w, OUT> for Parts<'w, 'a> {
        type Store = &'w mut ResourceSlot<u32>;
        fn take(&mut self) -> Option<Self::Store> {
            self.out.take()
        }
    }

    struct B<'a>(PhantomData<&'a ()>);

    impl<'a> Binding for B<'a> {
        type World = World<1, (), Rs<'a>>;
        type Buffer = Buffer;
        type Parts<'w>
            = Parts<'w, 'a>
        where
            Self: 'w;
        fn parts<'w>(world: &'w mut Self::World, _buffer: &'w mut Buffer) -> Parts<'w, 'a> {
            let ((), rs) = world.split_mut();
            Parts {
                map: Some(&mut rs.map),
                out: Some(&mut rs.out),
            }
        }
    }

    /// Somme des indices de toutes les cases de la carte lue.
    fn checksum(map: &Map<'_>) -> u32 {
        let mut sum = 0u32;
        for row in 0..map.height() {
            for column in 0..map.width() {
                sum += u32::from(map.tile(column, row).unwrap_or(0));
            }
        }
        sum
    }

    struct Sum;

    impl<'a> System<B<'a>> for Sum {
        type Access<'w>
            = (
            Read<MAP, <Parts<'w, 'a> as Locate<'w, MAP>>::Store>,
            Produce<OUT, <Parts<'w, 'a> as Locate<'w, OUT>>::Store>,
        )
        where
            B<'a>: 'w;
        fn run((map, mut out): Self::Access<'_>) -> Result<(), BufferFull> {
            let Some(map) = map.get() else {
                panic!("carte absente")
            };
            out.insert(checksum(map));
            Ok(())
        }
    }

    type Systems = Cons<At<0, Sum>, Nil>;

    /// Manifeste littéral : tileset (artefact 0, slot 0), carte (artefact 1, slot 1).
    const MANIFEST: [u8; 42] = [
        0x56, 0x4D, 0x41, 0x4E, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, // en-tête, n = 2
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // id du tileset
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // artefact 0, slot 0
        0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, // id de la carte
        0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // artefact 1, slot 1
    ];

    /// Tileset littéral (16 × 8, 4 colonnes, 12 tuiles).
    const TILESET: [u8; 18] = [
        0x56, 0x54, 0x4C, 0x53, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x10, 0x00, 0x08, 0x00, 0x04,
        0x00, 0x0C, 0x00,
    ];

    /// Carte 3 × 2 littérale (MAP-R1) référençant le tileset ci-dessus ; indices 0x0001,
    /// 0x0203, 0x0405, 0x0607, 0x0809, 0x0A0B, de somme 7716.
    const MAP_R1: [u8; 34] = [
        0x56, 0x4D, 0x41, 0x50, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, // préfixe, charge 24
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // identifiant du tileset
        0x03, 0x00, 0x02, 0x00, // 3 × 2
        0x01, 0x00, 0x03, 0x02, 0x05, 0x04, 0x07, 0x06, 0x09, 0x08, 0x0B, 0x0A,
    ];

    /// Charge le lot (tileset et carte) depuis des octets empruntés, puis exécute `ticks`
    /// ticks du véritable ordonnanceur ; rend la somme calculée par le système.
    fn run_ticks(tileset: &[u8], map: &[u8], ticks: usize) -> u32 {
        let scheduler = Scheduler::<B<'_>, Systems, 1, 1>::new().expect("modèle valide");
        let mut world = World::<1, (), Rs<'_>>::new();
        load(
            &MANIFEST,
            &[tileset, map],
            &[Format::Tileset, Format::Map],
            |_slot, asset| {
                if let Asset::Map(map) = asset {
                    world.resources_mut().map.insert(map);
                }
            },
        )
        .expect("chargement");
        let mut buffer = Buffer::new();
        for _ in 0..ticks {
            scheduler.run_tick(&mut world, &mut buffer).expect("tick");
        }
        *world.resources().out.get().expect("sortie")
    }

    #[test]
    fn lt5_the_real_map_view_runs_under_the_real_scheduler_and_depends_on_its_bytes() {
        // Tampons construits à l'exécution à partir des octets littéraux de MAP-R1,
        // déclarés avant le `World` (qui vit dans `run_ticks`).
        let tileset = TILESET.to_vec();
        let first = MAP_R1.to_vec();
        assert_eq!(run_ticks(&tileset, &first, 3), 7716);

        // Autre tampon : la case (0, 0) passe de 0x0001 à 0x0002 ; la sortie suit.
        let mut second = MAP_R1.to_vec();
        second[22] = 0x02;
        assert_eq!(run_ticks(&tileset, &second, 1), 7717);
    }
}
