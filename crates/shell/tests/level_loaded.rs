// crates/shell/tests/level_loaded.rs

//! Acceptation de l'étape 5 (U5) : un niveau chargé et affiché, SC-1 à SC-6.
//!
//! Chaîne : sources TOML (littéraux ci-dessous) → `compile_assets` → `CompiledSet` → `load`
//! (table `[Sprite, Tileset, Map, Font, Tileset]`, un `ResourceSlot` par asset) → `N`
//! itérations de `run_loop` → `run_tick` → `Frame` → fermeture de présentation
//! `FnMut(&Frame)` (Backend headless, montage de test comme à l'étape 4). Le lot compilé est
//! déclaré avant le `World` : les vues `Map<'a>` et `Font<'a>` empruntent ses octets.
//!
//! Les systèmes `Clear`, `Compose` et `Witness` sont ceux de l'étape 4. Les trois producteurs
//! (tuiles, sprite, texte) dérivent des contributions des ressources selon une règle
//! **synthétique, propre à ce test, hors de tout format d'artefact** ; aucun nom de partie,
//! de couche ou de jeu. La scène est **disjointe** (SC-2) et son résultat ne dépend d'aucun
//! ordre d'empilement (G-ORD) : aucun test ne compare l'ordre de `as_slice()`.
//!
//! Le lot compte deux tilesets de tuiles distinctes ; le second n'est lu par aucun producteur
//! (il rend visible une résolution de la carte vers le mauvais tileset) : SC-4 compte donc
//! cinq dépôts.
//!
//! **Non démontré** : contenu raster et Backend graphique ; règle d'ordre d'empilement (point
//! ouvert avant toute scène qui en dépend) ; streaming ; cohérence métier au-delà de la Forge.

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

// ---------------------------------------------------------------------------
// Instruments et attendus calculés à la main
// ---------------------------------------------------------------------------

/// Clé de tri locale au test : `Contribution` n'implémente ni `Ord` ni `Hash`.
fn key(c: &Contribution) -> (i32, i32, u32, u32, [u8; 4]) {
    (c.x, c.y, c.width, c.height, c.color)
}

/// Multiensemble : tri par clé locale, multiplicité conservée.
fn multiset(contributions: &[Contribution]) -> Vec<Contribution> {
    let mut copy = contributions.to_vec();
    copy.sort_by_key(key);
    copy
}

const fn rect(x: i32, y: i32, width: u32, height: u32, color: [u8; 4]) -> Contribution {
    Contribution {
        x,
        y,
        width,
        height,
        color,
    }
}

/// Contributions attendues, calculées **à la main** à partir des valeurs des sources TOML :
/// grille 3 × 2 de tuiles 16 × 8 (cases 0, 1, 2, 3, 4, 11 ; couleur `[20t, 255 − 10t, 64,
/// 255]`), sprite 20 × 10 en (0, 20), glyphes de largeurs 5, 7, 4 et d'avances 6, 8, 4 sur la
/// ligne `y = 40` de hauteur 10 (abscisses 0, 6, 14).
const EXPECTED: [Contribution; SCENE_LEN] = [
    rect(0, 0, 16, 8, [0, 255, 64, 255]),
    rect(16, 0, 16, 8, [20, 245, 64, 255]),
    rect(32, 0, 16, 8, [40, 235, 64, 255]),
    rect(0, 8, 16, 8, [60, 225, 64, 255]),
    rect(16, 8, 16, 8, [80, 215, 64, 255]),
    rect(32, 8, 16, 8, [220, 145, 64, 255]),
    rect(0, 20, 20, 10, [255, 255, 255, 255]),
    rect(0, 40, 5, 10, [200, 200, 0, 255]),
    rect(6, 40, 7, 10, [200, 200, 0, 255]),
    rect(14, 40, 4, 10, [200, 200, 0, 255]),
];

/// Deux rectangles se recouvrent si leurs intérieurs se coupent (aire d'intersection non nulle).
fn overlap(a: &Contribution, b: &Contribution) -> bool {
    let (ax, ay) = (i64::from(a.x), i64::from(a.y));
    let (bx, by) = (i64::from(b.x), i64::from(b.y));
    ax < bx + i64::from(b.width)
        && bx < ax + i64::from(a.width)
        && ay < by + i64::from(b.height)
        && by < ay + i64::from(a.height)
}

/// Vrai si les contributions sont deux à deux disjointes.
fn pairwise_disjoint(contributions: &[Contribution]) -> bool {
    contributions
        .iter()
        .enumerate()
        .all(|(i, a)| contributions[i + 1..].iter().all(|b| !overlap(a, b)))
}

/// Instrument de présentation (Backend headless) : conserve chaque `Frame` reçue.
struct Recorder {
    frames: Vec<Fr>,
}

impl Recorder {
    fn new() -> Self {
        Self { frames: Vec::new() }
    }

    fn present(&mut self, frame: &Fr) {
        self.frames.push(frame.clone());
    }
}

/// Résultat d'une exécution nominale : journal, `Frame` reçues et état chargé.
struct Run {
    journal: Vec<Entry>,
    frames: Vec<Fr>,
    /// `Frame` relue dans le `World` après la dernière itération.
    final_frame: Fr,
    loaded: LoadedValues,
}

/// Copie sans emprunt des valeurs chargées (le lot est détruit à la fin de `run_scene`).
#[derive(Debug, PartialEq, Eq)]
struct LoadedValues {
    deposits: usize,
    sprite: Option<Sprite>,
    tileset: Option<Tileset>,
    decoy: Option<Tileset>,
    map: Option<(SlotIndex, u16, u16, Vec<u16>)>,
    font: Option<(u16, u16, Vec<verbum_assets::Glyph>)>,
}

fn detach(loaded: &Loaded<'_>) -> LoadedValues {
    LoadedValues {
        deposits: loaded.deposits,
        sprite: loaded.sprite,
        tileset: loaded.tileset,
        decoy: loaded.decoy,
        map: loaded.map.map(|map| {
            let mut cells = Vec::new();
            for row in 0..map.height() {
                for column in 0..map.width() {
                    cells.push(map.tile(column, row).expect("case de la carte"));
                }
            }
            (map.tileset(), map.width(), map.height(), cells)
        }),
        font: loaded.font.map(|font| {
            let glyphs = (0..font.len())
                .map(|index| font.glyph(index).expect("glyphe de la police"))
                .collect();
            (font.len(), font.line_height(), glyphs)
        }),
    }
}

/// Compile `sources`, charge avec `table`, exécute `iterations` itérations. Le lot compilé est
/// déclaré **avant** le `World` : les vues du `World` empruntent ses octets.
fn run_scene<L>(
    sources: &[AssetSource<'_>],
    capacity: usize,
    table: &[Format],
    iterations: usize,
) -> Run
where
    L: for<'a> SystemList<B<'a>>,
{
    let set = compile_assets(sources, capacity).expect("lot valide");
    let artifacts = slices(&set);
    let mut journal = Vec::new();
    let mut recorder = Recorder::new();
    let execution = execute::<L, _>(
        set.manifest(),
        &artifacts,
        table,
        iterations,
        &mut journal,
        |frame| recorder.present(frame),
    )
    .expect("chargement");
    Run {
        journal,
        frames: recorder.frames,
        final_frame: execution
            .world
            .resources()
            .frame
            .get()
            .expect("frame")
            .0
            .clone(),
        loaded: detach(&execution.loaded),
    }
}

fn nominal<L>() -> Run
where
    L: for<'a> SystemList<B<'a>>,
{
    run_scene::<L>(&level_sources(), 5, &TABLE, N)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// SC-1 : sur `N` ticks, le Backend reçoit `N` `Frame`, chacune égale en multiensemble à
/// `EXPECTED` ; la multiplicité est conservée (`len` = 10).
#[test]
fn sc1_n_frames_equal_to_the_hand_computed_multiset() {
    let run = nominal::<ForwardSystems>();
    assert_eq!(run.frames.len(), N);
    for (k, frame) in run.frames.iter().enumerate() {
        assert_eq!(frame.len(), SCENE_LEN, "k = {k}");
        assert_eq!(multiset(frame.as_slice()), multiset(&EXPECTED), "k = {k}");
    }
}

/// SC-2 (G-ORD) : `EXPECTED` et chaque `Frame` reçue sont deux à deux disjointes ; la fonction
/// de disjonction détecte un recouvrement (le test n'est pas vacueux).
#[test]
fn sc2_contributions_are_pairwise_disjoint_so_no_stacking_order_matters() {
    assert!(pairwise_disjoint(&EXPECTED));
    let run = nominal::<ForwardSystems>();
    for frame in &run.frames {
        assert!(pairwise_disjoint(frame.as_slice()));
    }
    let shifted = rect(0, 4, 20, 10, SPRITE_COLOR);
    assert!(overlap(&shifted, &EXPECTED[0]));
    assert!(!pairwise_disjoint(&[EXPECTED[0], shifted]));
    // Rectangles qui se touchent par un bord ne se recouvrent pas.
    assert!(!overlap(&EXPECTED[0], &EXPECTED[1]));
}

/// SC-3 (G-ORD) : l'ordre de déclaration des trois producteurs est inversé ; les `Frame` sont
/// égales en multiensemble, le journal et l'état chargé sont égaux. Aucune comparaison de
/// l'ordre de `as_slice()`.
#[test]
fn sc3_reversing_the_producer_declaration_order_leaves_the_frames_equal_as_multisets() {
    let forward = nominal::<ForwardSystems>();
    let reversed = nominal::<ReversedSystems>();
    assert_eq!(forward.frames.len(), N);
    assert_eq!(reversed.frames.len(), N);
    for (a, b) in forward.frames.iter().zip(&reversed.frames) {
        assert_eq!(multiset(a.as_slice()), multiset(b.as_slice()));
        assert_eq!(multiset(b.as_slice()), multiset(&EXPECTED));
    }
    assert_eq!(forward.journal, reversed.journal);
    assert_eq!(forward.loaded, reversed.loaded);
}

/// SC-4 : avant le premier tick, exactement cinq dépôts (les quatre formats, le lot comptant
/// deux tilesets) ; chaque ressource égale aux littéraux ; `map.tileset()` est le `SlotIndex`
/// du slot du tileset, résolu avant le hot path.
#[test]
fn sc4_before_the_first_tick_every_resource_is_deposited_and_the_map_names_its_tileset_slot() {
    let run = run_scene::<ForwardSystems>(&level_sources(), 5, &TABLE, 0);
    assert!(run.journal.is_empty(), "aucun tick");
    assert!(run.frames.is_empty(), "aucune présentation");
    let loaded = run.loaded;
    assert_eq!(loaded.deposits, 5);
    assert_eq!(
        loaded.sprite,
        Some(Sprite {
            width: 20,
            height: 10
        })
    );
    assert_eq!(
        loaded.tileset,
        Some(Tileset {
            tile_width: 16,
            tile_height: 8,
            columns: 4,
            count: 12
        })
    );
    assert_eq!(
        loaded.decoy,
        Some(Tileset {
            tile_width: 32,
            tile_height: 32,
            columns: 2,
            count: 12
        })
    );
    assert_eq!(
        loaded.map,
        Some((SlotIndex::new(SLOT_TILESET), 3, 2, vec![0, 1, 2, 3, 4, 11]))
    );
    assert_eq!(
        loaded.font,
        Some((
            3,
            10,
            vec![
                verbum_assets::Glyph {
                    codepoint: 65,
                    width: 5,
                    advance: 6
                },
                verbum_assets::Glyph {
                    codepoint: 66,
                    width: 7,
                    advance: 8
                },
                verbum_assets::Glyph {
                    codepoint: 67,
                    width: 4,
                    advance: 4
                },
            ]
        ))
    );
}

/// SC-5 : un artefact corrompu dans le lot (troncature, format de slot erroné, référence
/// invalide) : `load` échoue ; aucun tick, aucune présentation.
#[test]
fn sc5_a_corrupted_lot_fails_to_load_with_no_tick_and_no_presentation() {
    let set = compile_assets(&level_sources(), 5).expect("lot valide");
    let good = slices(&set);
    // Artefacts : 0 police, 1 carte, 2 tileset, 3 leurre, 4 sprite.

    let run = |manifest: &[u8], artifacts: &[&[u8]], table: &[Format]| {
        let mut journal = Vec::new();
        let mut presented = 0usize;
        let result =
            execute::<ForwardSystems, _>(manifest, artifacts, table, N, &mut journal, |_| {
                presented += 1;
            });
        assert!(journal.is_empty(), "aucun tick");
        assert_eq!(presented, 0, "aucune présentation");
        result.err().expect("chargement rejeté")
    };

    // Troncature de la police (artefact 0).
    let truncated = &set.artifacts()[0][..set.artifacts()[0].len() - 1];
    let lot: Vec<&[u8]> = vec![truncated, good[1], good[2], good[3], good[4]];
    assert!(matches!(
        run(set.manifest(), &lot, &TABLE),
        LoadError::Artifact {
            error: verbum_assets::ReadError::Truncated,
            ..
        }
    ));

    // Format de slot erroné (le slot 3 attend un sprite alors que l'artefact est une police).
    let wrong = [
        Format::Sprite,
        Format::Tileset,
        Format::Map,
        Format::Sprite,
        Format::Tileset,
    ];
    assert_eq!(
        run(set.manifest(), &good, &wrong),
        LoadError::SlotFormat {
            slot: SlotIndex::new(SLOT_FONT)
        }
    );

    // Référence invalide : l'identifiant du tileset, dans la carte (artefact 1), est altéré.
    let mut dangling = set.artifacts()[1].clone();
    dangling[10] ^= 0xFF;
    let lot: Vec<&[u8]> = vec![good[0], &dangling, good[2], good[3], good[4]];
    assert!(matches!(
        run(set.manifest(), &lot, &TABLE),
        LoadError::Reference {
            error: verbum_assets::ReferenceError::Unresolved,
            ..
        }
    ));
}

/// Accepte toute présentation de la forme `FnMut(&Frame)`.
fn accept<F: FnMut(&Fr)>(mut presentation: F, frame: &Fr) {
    presentation(frame);
}

/// SC-6 : séquence tick → présentation sur le journal (`[TickStart(k), TickEnd(k),
/// Present(k)]` pour `k` de 1 à `N`) et signature `FnMut(&Frame)` sans retour. Les tests de
/// l'étape 4 ne sont pas modifiés et `crates/render/**` n'a aucun diff (contrôle de dépôt).
#[test]
fn sc6_the_journal_orders_tick_then_presentation_and_the_backend_takes_a_frame_by_reference() {
    let run = nominal::<ForwardSystems>();
    let expected: Vec<Entry> = (1..=N)
        .flat_map(|k| [Entry::TickStart(k), Entry::TickEnd(k), Entry::Present(k)])
        .collect();
    assert_eq!(run.journal, expected);
    // La `Frame` relue dans le `World` après la présentation est celle qui a été présentée.
    assert_eq!(Some(&run.final_frame), run.frames.last());

    let _: fn(&mut Recorder, &Fr) = Recorder::present;
    let mut recorder = Recorder::new();
    accept(|frame| recorder.present(frame), &Fr::empty());
    assert_eq!(recorder.frames, [Fr::empty()]);
}
