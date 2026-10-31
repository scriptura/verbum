// crates/shell/examples/level_headless.rs

//! Exemple headless de l'étape 5 : charge un niveau compilé par la Forge et fait circuler ses
//! contributions jusqu'à une fermeture de présentation qui les imprime. La scène (sources,
//! producteurs, assemblage) est celle de `tests/level_loaded.rs`, recopiée.

use core::convert::Infallible;
use core::marker::PhantomData;
use std::cell::Cell;

use verbum_assets::{Asset, Font, Format, LoadError, Map, SlotIndex, Sprite, Tileset, load};
use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
use verbum_forge::{AssetKind, AssetSource, CompiledSet, compile_assets};
use verbum_render::{Contribution, Contributions, Frame};
use verbum_schedule::access::{Locate, Produce, Read};
use verbum_schedule::scheduler::Scheduler;
use verbum_schedule::system::{At, Binding, Cons, Nil, System, SystemList};
use verbum_shell::{Inbox, run_loop};

// ---------------------------------------------------------------------------
// Sources TOML de la scène : littéraux, synthétiques, sans nom de jeu
// ---------------------------------------------------------------------------

const SPRITE_TOML: &str = "width = 20\nheight = 10\n";
const TILESET_TOML: &str = "tile_width = 16\ntile_height = 8\ncolumns = 4\ncount = 12\n";
/// Second tileset, de tuile distincte : rend visible une résolution vers le mauvais tileset.
const DECOY_TOML: &str = "tile_width = 32\ntile_height = 32\ncolumns = 2\ncount = 12\n";
const MAP_TOML: &str = "tileset = \"tiles\"\nwidth = 3\nheight = 2\ncells = [0, 1, 2, 3, 4, 11]\n";
const FONT_TOML: &str = "line_height = 10\n\
    [[glyph]]\ncodepoint = 65\nwidth = 5\nadvance = 6\n\
    [[glyph]]\ncodepoint = 66\nwidth = 7\nadvance = 8\n\
    [[glyph]]\ncodepoint = 67\nwidth = 4\nadvance = 4\n";

/// Slots (ordinaux de la configuration statique) et formats attendus.
const SLOT_SPRITE: u32 = 0;
const SLOT_TILESET: u32 = 1;
const SLOT_MAP: u32 = 2;
const SLOT_FONT: u32 = 3;
const SLOT_DECOY: u32 = 4;
static TABLE: [Format; 5] = [
    Format::Sprite,
    Format::Tileset,
    Format::Map,
    Format::Font,
    Format::Tileset,
];

/// Ordre de compilation : `artifact_index ≠ slot_index` pour chaque entrée.
fn level_sources() -> [AssetSource<'static>; 5] {
    let source = |kind, key, toml, slot| AssetSource {
        kind,
        key,
        toml,
        slot: SlotIndex::new(slot),
    };
    [
        source(AssetKind::Font, "face", FONT_TOML, SLOT_FONT),
        source(AssetKind::Map, "level", MAP_TOML, SLOT_MAP),
        source(AssetKind::Tileset, "tiles", TILESET_TOML, SLOT_TILESET),
        source(AssetKind::Tileset, "decoy", DECOY_TOML, SLOT_DECOY),
        source(AssetKind::Sprite, "hero", SPRITE_TOML, SLOT_SPRITE),
    ]
}

fn slices(set: &CompiledSet) -> Vec<&[u8]> {
    set.artifacts().iter().map(Vec::as_slice).collect()
}

// ---------------------------------------------------------------------------
// Constantes de scène : règle de dérivation synthétique, propre au montage
// ---------------------------------------------------------------------------

/// Nombre de contributions : 6 tuiles, 1 sprite, 3 glyphes.
const SCENE_LEN: usize = 6 + 1 + 3;
/// Capacité de l'accumulateur et de la `Frame`.
const CAP: usize = 12;
/// Borne de production par tick : somme des productions des trois producteurs.
const MAX_PAR_TICK: usize = SCENE_LEN;
const _: () = assert!(MAX_PAR_TICK <= CAP);

/// Position du sprite : sous la grille (qui occupe `y < 16`).
const SPRITE_X: i32 = 0;
const SPRITE_Y: i32 = 20;
const SPRITE_COLOR: [u8; 4] = [255, 255, 255, 255];
/// Ligne du texte : sous le sprite (qui occupe `y < 30`).
const TEXT_Y: i32 = 40;
const TEXT_COLOR: [u8; 4] = [200, 200, 0, 255];

/// Couleur d'une tuile : fonction affine de son indice.
fn tile_color(index: u16) -> [u8; 4] {
    let t = u8::try_from(index).expect("indice de tuile < 256");
    [t * 20, 255 - t * 10, 64, 255]
}

/// Nombre d'itérations des scénarios.
const N: usize = 3;

type Acc = Contributions<CAP>;
type Fr = Frame<CAP>;

const ACC: u8 = 0;
const FRAME: u8 = 1;
const WITNESS: u8 = 2;
const SPRITE: u8 = 3;
const TILESET: u8 = 4;
const MAP: u8 = 5;
const FONT: u8 = 6;

/// Ancre de durée de vie. Un système dont l'accès ne mentionne aucun type portant `'a` ne
/// compile pas sous une liaison `B<'a>` : `fn run(Self::Access<'_>)` exige `B<'a>: '_`
/// (E0477), que rien dans ses arguments n'implique. Envelopper la ressource dans un type qui
/// porte `'a` rend `'a: 'w` implicite (WF de `&'w mut ResourceSlot<Anchored<'a, T>>`) sans
/// changer ni catégorie ni contrat. Propre à ce montage ; voir le rapport de U5.
struct Anchored<'a, T>(T, PhantomData<&'a ()>);

impl<T> Anchored<'_, T> {
    fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

/// `R` de l'application : un slot par asset ; les vues empruntent les octets des artefacts.
#[derive(Default)]
struct Rs<'a> {
    acc: ResourceSlot<Anchored<'a, Acc>>,
    frame: ResourceSlot<Anchored<'a, Fr>>,
    witness: ResourceSlot<Anchored<'a, Fr>>,
    sprite: ResourceSlot<Anchored<'a, Sprite>>,
    tileset: ResourceSlot<Tileset>,
    map: ResourceSlot<Map<'a>>,
    font: ResourceSlot<Font<'a>>,
    decoy: ResourceSlot<Tileset>,
}

type SceneWorld<'a> = World<1, (), Rs<'a>>;
type Buffer = CommandBuffer<1, Infallible, Infallible>;

struct Parts<'w, 'a> {
    acc: Option<&'w mut ResourceSlot<Anchored<'a, Acc>>>,
    frame: Option<&'w mut ResourceSlot<Anchored<'a, Fr>>>,
    witness: Option<&'w mut ResourceSlot<Anchored<'a, Fr>>>,
    sprite: Option<&'w mut ResourceSlot<Anchored<'a, Sprite>>>,
    tileset: Option<&'w mut ResourceSlot<Tileset>>,
    map: Option<&'w mut ResourceSlot<Map<'a>>>,
    font: Option<&'w mut ResourceSlot<Font<'a>>>,
}

macro_rules! locate {
    ($id:ident, $field:ident, $ty:ty) => {
        impl<'w, 'a> Locate<'w, $id> for Parts<'w, 'a> {
            type Store = &'w mut ResourceSlot<$ty>;
            fn take(&mut self) -> Option<Self::Store> {
                self.$field.take()
            }
        }
    };
}

locate!(ACC, acc, Anchored<'a, Acc>);
locate!(FRAME, frame, Anchored<'a, Fr>);
locate!(WITNESS, witness, Anchored<'a, Fr>);
locate!(SPRITE, sprite, Anchored<'a, Sprite>);
locate!(TILESET, tileset, Tileset);
locate!(MAP, map, Map<'a>);
locate!(FONT, font, Font<'a>);

struct B<'a>(PhantomData<&'a ()>);

impl<'a> Binding for B<'a> {
    type World = SceneWorld<'a>;
    type Buffer = Buffer;
    type Parts<'w>
        = Parts<'w, 'a>
    where
        Self: 'w;

    fn parts<'w>(world: &'w mut Self::World, _buffer: &'w mut Buffer) -> Parts<'w, 'a> {
        let ((), rs) = world.split_mut();
        Parts {
            acc: Some(&mut rs.acc),
            frame: Some(&mut rs.frame),
            witness: Some(&mut rs.witness),
            sprite: Some(&mut rs.sprite),
            tileset: Some(&mut rs.tileset),
            map: Some(&mut rs.map),
            font: Some(&mut rs.font),
        }
    }
}

type ProduceAcc<'w, 'a> = Produce<ACC, <Parts<'w, 'a> as Locate<'w, ACC>>::Store>;
type ReadAcc<'w, 'a> = Read<ACC, <Parts<'w, 'a> as Locate<'w, ACC>>::Store>;
type ProduceFrame<'w, 'a> = Produce<FRAME, <Parts<'w, 'a> as Locate<'w, FRAME>>::Store>;
type ReadFrame<'w, 'a> = Read<FRAME, <Parts<'w, 'a> as Locate<'w, FRAME>>::Store>;
type ProduceWitness<'w, 'a> = Produce<WITNESS, <Parts<'w, 'a> as Locate<'w, WITNESS>>::Store>;
type ReadSprite<'w, 'a> = Read<SPRITE, <Parts<'w, 'a> as Locate<'w, SPRITE>>::Store>;
type ReadTileset<'w, 'a> = Read<TILESET, <Parts<'w, 'a> as Locate<'w, TILESET>>::Store>;
type ReadMap<'w, 'a> = Read<MAP, <Parts<'w, 'a> as Locate<'w, MAP>>::Store>;
type ReadFont<'w, 'a> = Read<FONT, <Parts<'w, 'a> as Locate<'w, FONT>>::Store>;

// ---------------------------------------------------------------------------
// Systèmes : `Clear`, `Compose`, `Witness` (motif de l'étape 4) et trois producteurs
// ---------------------------------------------------------------------------

/// Phase 0 : remet l'accumulateur à l'état initial.
struct Clear;

impl<'a> System<B<'a>> for Clear {
    type Access<'w>
        = (ProduceAcc<'w, 'a>,)
    where
        B<'a>: 'w;

    fn run((mut acc,): Self::Access<'_>) -> Result<(), BufferFull> {
        acc.insert(Anchored::new(Acc::new()));
        Ok(())
    }
}

/// Phase 1 : unique lecteur de l'accumulateur, unique producteur de la `Frame`.
struct Compose;

impl<'a> System<B<'a>> for Compose {
    type Access<'w>
        = (ReadAcc<'w, 'a>, ProduceFrame<'w, 'a>)
    where
        B<'a>: 'w;

    fn run((acc, mut frame): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(source) = acc.get() else {
            panic!("violation d'assemblage : accumulateur absent");
        };
        frame.insert(Anchored::new(Fr::compose(&source.0)));
        Ok(())
    }
}

/// Phase 1 : lecture seule de la `Frame`, copie dans une catégorie de test.
struct Witness;

impl<'a> System<B<'a>> for Witness {
    type Access<'w>
        = (ReadFrame<'w, 'a>, ProduceWitness<'w, 'a>)
    where
        B<'a>: 'w;

    fn run((frame, mut witness): Self::Access<'_>) -> Result<(), BufferFull> {
        let Some(frame) = frame.get() else {
            panic!("violation d'assemblage : frame absente");
        };
        witness.insert(Anchored::new(frame.0.clone()));
        Ok(())
    }
}

/// Émet une contribution ; la saturation est une violation d'assemblage (`panic!`).
fn emit(acc: &mut Anchored<'_, Acc>, contribution: Contribution, producer: &str) {
    if acc.0.push(contribution).is_err() {
        panic!("borne de production dépassée : accumulateur plein ({producer})");
    }
}

/// Phase 1 : une contribution par case de la carte, à la taille de tuile du tileset lu.
struct ProduceTiles;

impl<'a> System<B<'a>> for ProduceTiles {
    type Access<'w>
        = (ReadMap<'w, 'a>, ReadTileset<'w, 'a>, ProduceAcc<'w, 'a>)
    where
        B<'a>: 'w;

    fn run((map, tileset, mut acc): Self::Access<'_>) -> Result<(), BufferFull> {
        let (Some(map), Some(tileset), Some(acc)) = (map.get(), tileset.get(), acc.get_mut())
        else {
            panic!("violation d'assemblage : ressource absente (ProduceTiles)");
        };
        // La carte désigne le slot du tileset lu : identifiant résolu avant le hot path.
        assert_eq!(map.tileset(), SlotIndex::new(SLOT_TILESET));
        for row in 0..map.height() {
            for column in 0..map.width() {
                let Some(index) = map.tile(column, row) else {
                    panic!("case hors de la carte (ProduceTiles)");
                };
                emit(
                    acc,
                    Contribution {
                        x: i32::from(column) * i32::from(tileset.tile_width),
                        y: i32::from(row) * i32::from(tileset.tile_height),
                        width: u32::from(tileset.tile_width),
                        height: u32::from(tileset.tile_height),
                        color: tile_color(index),
                    },
                    "ProduceTiles",
                );
            }
        }
        Ok(())
    }
}

/// Phase 1 : un rectangle aux dimensions du sprite, sous la grille.
struct ProduceSprite;

impl<'a> System<B<'a>> for ProduceSprite {
    type Access<'w>
        = (ReadSprite<'w, 'a>, ProduceAcc<'w, 'a>)
    where
        B<'a>: 'w;

    fn run((sprite, mut acc): Self::Access<'_>) -> Result<(), BufferFull> {
        let (Some(sprite), Some(acc)) = (sprite.get(), acc.get_mut()) else {
            panic!("violation d'assemblage : ressource absente (ProduceSprite)");
        };
        emit(
            acc,
            Contribution {
                x: SPRITE_X,
                y: SPRITE_Y,
                width: u32::from(sprite.0.width),
                height: u32::from(sprite.0.height),
                color: SPRITE_COLOR,
            },
            "ProduceSprite",
        );
        Ok(())
    }
}

/// Phase 1 : un rectangle par glyphe, à l'abscisse cumulée des avances précédentes.
struct ProduceText;

impl<'a> System<B<'a>> for ProduceText {
    type Access<'w>
        = (ReadFont<'w, 'a>, ProduceAcc<'w, 'a>)
    where
        B<'a>: 'w;

    fn run((font, mut acc): Self::Access<'_>) -> Result<(), BufferFull> {
        let (Some(font), Some(acc)) = (font.get(), acc.get_mut()) else {
            panic!("violation d'assemblage : ressource absente (ProduceText)");
        };
        let mut pen = 0i32;
        for index in 0..font.len() {
            let Some(glyph) = font.glyph(index) else {
                panic!("glyphe hors de la police (ProduceText)");
            };
            emit(
                acc,
                Contribution {
                    x: pen,
                    y: TEXT_Y,
                    width: u32::from(glyph.width),
                    height: u32::from(font.line_height()),
                    color: TEXT_COLOR,
                },
                "ProduceText",
            );
            pen += i32::from(glyph.advance);
        }
        Ok(())
    }
}

const SYSTEMS: usize = 6;
const PHASES: usize = 2;

/// Déclaration des producteurs : tuiles, sprite, texte.
type ForwardSystems = Cons<
    At<0, Clear>,
    Cons<
        At<1, Compose>,
        Cons<
            At<1, Witness>,
            Cons<At<1, ProduceTiles>, Cons<At<1, ProduceSprite>, Cons<At<1, ProduceText>, Nil>>>,
        >,
    >,
>;

/// Déclaration des producteurs inversée : texte, sprite, tuiles.
type ReversedSystems = Cons<
    At<0, Clear>,
    Cons<
        At<1, Compose>,
        Cons<
            At<1, Witness>,
            Cons<At<1, ProduceText>, Cons<At<1, ProduceSprite>, Cons<At<1, ProduceTiles>, Nil>>>,
        >,
    >,
>;

// ---------------------------------------------------------------------------
// Exécution : chargement, puis `N` itérations sous `run_loop`
// ---------------------------------------------------------------------------

/// Entrée du journal de la fermeture d'infrastructure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    TickStart(usize),
    TickEnd(usize),
    Present(usize),
}

/// Valeurs des ressources, lues juste après `load` et avant tout tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Loaded<'a> {
    deposits: usize,
    sprite: Option<Sprite>,
    tileset: Option<Tileset>,
    decoy: Option<Tileset>,
    map: Option<Map<'a>>,
    font: Option<Font<'a>>,
}

struct Execution<'a> {
    loaded: Loaded<'a>,
    world: SceneWorld<'a>,
}

fn deposit<'a>(world: &mut SceneWorld<'a>, slot: SlotIndex, asset: Asset<'a>) {
    let rs = world.resources_mut();
    let replaced = match (slot.get(), asset) {
        (SLOT_SPRITE, Asset::Sprite(value)) => rs.sprite.insert(Anchored::new(value)).is_some(),
        (SLOT_TILESET, Asset::Tileset(value)) => rs.tileset.insert(value).is_some(),
        (SLOT_MAP, Asset::Map(value)) => rs.map.insert(value).is_some(),
        (SLOT_FONT, Asset::Font(value)) => rs.font.insert(value).is_some(),
        (SLOT_DECOY, Asset::Tileset(value)) => rs.decoy.insert(value).is_some(),
        (slot, other) => panic!("dépôt inattendu : slot {slot}, {other:?}"),
    };
    assert!(!replaced, "slot déjà occupé");
}

/// Charge le lot, puis exécute `iterations` itérations sous `run_loop` avec la liste de
/// systèmes `L`. Le journal et la présentation appartiennent à l'appelant : sur une erreur de
/// chargement ils restent intacts (aucun tick, aucune présentation).
fn execute<'a, L, P>(
    manifest: &[u8],
    artifacts: &[&'a [u8]],
    table: &[Format],
    iterations: usize,
    journal: &mut Vec<Entry>,
    mut present: P,
) -> Result<Execution<'a>, LoadError>
where
    L: SystemList<B<'a>>,
    P: FnMut(&Fr),
{
    let scheduler = Scheduler::<B<'a>, L, SYSTEMS, PHASES>::new().expect("un modèle valide");
    let mut world = SceneWorld::new();
    let mut deposits = 0usize;
    load(manifest, artifacts, table, |slot, asset| {
        deposits += 1;
        deposit(&mut world, slot, asset);
    })?;
    {
        let rs = world.resources_mut();
        rs.acc.insert(Anchored::new(Acc::new()));
        rs.frame.insert(Anchored::new(Fr::empty()));
        rs.witness.insert(Anchored::new(Fr::empty()));
    }
    let rs = world.resources();
    let loaded = Loaded {
        deposits,
        sprite: rs.sprite.get().map(|anchored| anchored.0),
        tileset: rs.tileset.get().copied(),
        decoy: rs.decoy.get().copied(),
        map: rs.map.get().copied(),
        font: rs.font.get().copied(),
    };

    let mut buffer = Buffer::new();
    let mut inbox = Inbox::<u8>::new(4);
    let stops = Cell::new(0usize);
    let mut k = 0usize;
    let ran = run_loop(
        || {},
        || {
            stops.set(stops.get() + 1);
            stops.get() > iterations
        },
        &mut inbox,
        |_lot: &[u8]| -> Result<(), verbum_schedule::scheduler::TickError> {
            k += 1;
            journal.push(Entry::TickStart(k));
            scheduler.run_tick(&mut world, &mut buffer)?;
            journal.push(Entry::TickEnd(k));
            let Some(frame) = world.resources().frame.get() else {
                panic!("violation d'assemblage : frame absente");
            };
            present(&frame.0);
            journal.push(Entry::Present(k));
            Ok(())
        },
    );
    assert_eq!(ran, Ok(iterations as u64), "boucle");
    Ok(Execution { loaded, world })
}

fn main() {
    let set = compile_assets(&level_sources(), 5).expect("lot valide");
    let artifacts = slices(&set);
    let mut journal = Vec::new();
    let mut presented = 0usize;
    let execution = execute::<ForwardSystems, _>(
        set.manifest(),
        &artifacts,
        &TABLE,
        N,
        &mut journal,
        |frame| {
            presented += 1;
            println!("frame {presented} : {} contributions", frame.len());
            for contribution in frame.as_slice() {
                println!(
                    "  ({}, {}) {}x{} {:?}",
                    contribution.x,
                    contribution.y,
                    contribution.width,
                    contribution.height,
                    contribution.color
                );
            }
        },
    )
    .expect("chargement");
    let loaded = execution.loaded;
    println!("{} dépôts avant le premier tick", loaded.deposits);
    if let (Some(sprite), Some(tileset), Some(decoy), Some(map), Some(font)) = (
        loaded.sprite,
        loaded.tileset,
        loaded.decoy,
        loaded.map,
        loaded.font,
    ) {
        println!(
            "sprite {}x{}, tileset {}x{} ({} colonnes, {} tuiles), leurre {}x{}",
            sprite.width,
            sprite.height,
            tileset.tile_width,
            tileset.tile_height,
            tileset.columns,
            tileset.count,
            decoy.tile_width,
            decoy.tile_height
        );
        println!(
            "carte {}x{} sur le slot {}, police {} glyphes",
            map.width(),
            map.height(),
            map.tileset().get(),
            font.len()
        );
    }
    let ticks = journal
        .iter()
        .filter(|entry| matches!(entry, Entry::Present(_)))
        .count();
    println!(
        "{ticks} présentations ; frame finale : {} contributions",
        execution
            .world
            .resources()
            .frame
            .get()
            .map_or(0, |frame| frame.0.len())
    );

    // Même scène, producteurs déclarés dans l'ordre inverse : le résultat visuel est le même.
    let mut reversed_journal = Vec::new();
    let mut reversed_len = 0usize;
    execute::<ReversedSystems, _>(
        set.manifest(),
        &artifacts,
        &TABLE,
        1,
        &mut reversed_journal,
        |frame| reversed_len = frame.len(),
    )
    .expect("chargement");
    println!("déclaration inversée : {reversed_len} contributions");
}
