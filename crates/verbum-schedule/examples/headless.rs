// crates/verbum-schedule/examples/headless.rs

//! Headless run of the stage 1 scheduler: resources only, several ticks, no output.
//!
//! Three phases per tick. Phase 0 clears the transient `SAMPLE`; phase 1 has two producers
//! add to it, freely; phase 2 has the single resolver add it to the persistent `TOTAL`.
//! The scheduler resets nothing itself: the clearing is a system, declared and ordered.
//!
//! The example prints nothing and exits 0 when the observed totals match the expected ones;
//! a failed check panics, which exits non-zero.
//!
//! ```text
//! cargo run -p verbum-schedule --example headless
//! ```

use core::convert::Infallible;

use verbum_ecs::{BufferFull, CommandBuffer, ResourceSlot, World};
use verbum_schedule::access::{Locate, Produce, Read, Write};
use verbum_schedule::scheduler::Scheduler;
use verbum_schedule::system::{At, Binding, Cons, Nil, System};

/// Transient: rebuilt every tick.
struct Sample(u32);

/// Persistent: kept across ticks.
struct Total(u32);

const SAMPLE: u8 = 0;
const TOTAL: u8 = 1;

#[derive(Default)]
struct Globals {
    sample: ResourceSlot<Sample>,
    total: ResourceSlot<Total>,
}

/// No component: the world carries resources only.
type Headless = World<1, (), Globals>;
type Buffer = CommandBuffer<1, Infallible, Infallible>;

struct Parts<'w> {
    sample: Option<&'w mut ResourceSlot<Sample>>,
    total: Option<&'w mut ResourceSlot<Total>>,
}

impl<'w> Locate<'w, SAMPLE> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Sample>;
    fn take(&mut self) -> Option<Self::Store> {
        self.sample.take()
    }
}

impl<'w> Locate<'w, TOTAL> for Parts<'w> {
    type Store = &'w mut ResourceSlot<Total>;
    fn take(&mut self) -> Option<Self::Store> {
        self.total.take()
    }
}

struct HeadlessBinding;

impl Binding for HeadlessBinding {
    type World = Headless;
    type Buffer = Buffer;
    type Parts<'w> = Parts<'w>;

    fn parts<'w>(world: &'w mut Headless, _buffer: &'w mut Buffer) -> Parts<'w> {
        let ((), globals) = world.split_mut();
        Parts {
            sample: Some(&mut globals.sample),
            total: Some(&mut globals.total),
        }
    }
}

type ProduceSample<'w> = Produce<SAMPLE, <Parts<'w> as Locate<'w, SAMPLE>>::Store>;
type ReadSample<'w> = Read<SAMPLE, <Parts<'w> as Locate<'w, SAMPLE>>::Store>;
type WriteTotal<'w> = Write<TOTAL, <Parts<'w> as Locate<'w, TOTAL>>::Store>;

/// Phase 0: sets the sample to zero.
struct Clear;

impl System<HeadlessBinding> for Clear {
    type Access<'w> = (ProduceSample<'w>,);

    fn run((mut sample,): Self::Access<'_>) -> Result<(), BufferFull> {
        sample.insert(Sample(0));
        Ok(())
    }
}

/// Phase 1: adds `K` to the sample.
struct Contribute<const K: u32>;

impl<const K: u32> System<HeadlessBinding> for Contribute<K> {
    type Access<'w> = (ProduceSample<'w>,);

    fn run((mut sample,): Self::Access<'_>) -> Result<(), BufferFull> {
        if let Some(sample) = sample.get_mut() {
            sample.0 += K;
        }
        Ok(())
    }
}

/// Phase 2: adds the sample to the total, the only writer of `TOTAL`.
struct Resolve;

impl System<HeadlessBinding> for Resolve {
    type Access<'w> = (ReadSample<'w>, WriteTotal<'w>);

    fn run((sample, mut total): Self::Access<'_>) -> Result<(), BufferFull> {
        let gained = sample.get().map_or(0, |sample| sample.0);
        let so_far = total.get().map_or(0, |total| total.0);
        total.insert(Total(so_far + gained));
        Ok(())
    }
}

type Systems = Cons<
    At<0, Clear>,
    Cons<At<1, Contribute<1>>, Cons<At<1, Contribute<2>>, Cons<At<2, Resolve>, Nil>>>,
>;

const TICKS: u32 = 5;
const PER_TICK: u32 = 1 + 2;

fn main() {
    let scheduler = Scheduler::<HeadlessBinding, Systems, 4, 3>::new().expect("a valid schedule");
    let mut world = Headless::new();
    let mut buffer = Buffer::new();

    for tick in 1..=TICKS {
        scheduler
            .run_tick(&mut world, &mut buffer)
            .expect("the tick completes");
        let total = world.resources().total.get().map(|total| total.0);
        assert_eq!(total, Some(tick * PER_TICK));
    }
}
