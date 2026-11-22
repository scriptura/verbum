# Structure du Projet

**Généré le:** 2026-11-22 07:46:19

## Racine du Workspace

```text
/verbum
├── crates
│   ├── engine
│   │   ├── assets
│   │   │   ├── src
│   │   │   │   ├── font.rs
│   │   │   │   ├── format.rs
│   │   │   │   ├── header.rs
│   │   │   │   ├── lib.rs
│   │   │   │   ├── load.rs
│   │   │   │   ├── manifest.rs
│   │   │   │   ├── manifest_v2.rs
│   │   │   │   ├── map.rs
│   │   │   │   ├── package.rs
│   │   │   │   ├── script.rs
│   │   │   │   ├── sprite.rs
│   │   │   │   └── tileset.rs
│   │   │   ├── Cargo.toml
│   │   │   └── README.md
│   │   ├── ecs
│   │   │   ├── src
│   │   │   │   ├── command.rs
│   │   │   │   ├── component_set.rs
│   │   │   │   ├── entity.rs
│   │   │   │   ├── fixtures.rs
│   │   │   │   ├── lib.rs
│   │   │   │   ├── query.rs
│   │   │   │   ├── resource.rs
│   │   │   │   ├── storage.rs
│   │   │   │   ├── values.rs
│   │   │   │   └── world.rs
│   │   │   ├── tests
│   │   │   │   ├── acceptance.rs
│   │   │   │   ├── command_buffer.rs
│   │   │   │   ├── components.rs
│   │   │   │   ├── query.rs
│   │   │   │   ├── resources.rs
│   │   │   │   └── world_identity.rs
│   │   │   ├── Cargo.toml
│   │   │   └── README.md
│   │   ├── forge
│   │   │   ├── src
│   │   │   │   ├── batch.rs
│   │   │   │   ├── font.rs
│   │   │   │   ├── id.rs
│   │   │   │   ├── lib.rs
│   │   │   │   ├── map.rs
│   │   │   │   ├── package.rs
│   │   │   │   ├── script.rs
│   │   │   │   ├── sprite.rs
│   │   │   │   └── tileset.rs
│   │   │   ├── tests
│   │   │   │   ├── package.rs
│   │   │   │   └── pipeline.rs
│   │   │   ├── Cargo.toml
│   │   │   └── README.md
│   │   ├── render
│   │   │   ├── src
│   │   │   │   ├── accumulator.rs
│   │   │   │   ├── contribution.rs
│   │   │   │   ├── frame.rs
│   │   │   │   ├── lib.rs
│   │   │   │   └── representation.rs
│   │   │   ├── tests
│   │   │   │   └── pipeline.rs
│   │   │   ├── Cargo.toml
│   │   │   └── README.md
│   │   ├── schedule
│   │   │   ├── examples
│   │   │   │   └── headless.rs
│   │   │   ├── src
│   │   │   │   ├── access.rs
│   │   │   │   ├── contract.rs
│   │   │   │   ├── data.rs
│   │   │   │   ├── lib.rs
│   │   │   │   ├── model.rs
│   │   │   │   ├── order.rs
│   │   │   │   ├── scheduler.rs
│   │   │   │   └── system.rs
│   │   │   ├── tests
│   │   │   │   ├── support
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── acceptance.rs
│   │   │   │   ├── access.rs
│   │   │   │   └── tick.rs
│   │   │   ├── Cargo.toml
│   │   │   └── README.md
│   │   ├── shell
│   │   │   ├── examples
│   │   │   │   ├── console.rs
│   │   │   │   ├── headless_frame.rs
│   │   │   │   ├── level_headless.rs
│   │   │   │   └── script_headless.rs
│   │   │   ├── src
│   │   │   │   ├── clock.rs
│   │   │   │   ├── driver.rs
│   │   │   │   ├── inbox.rs
│   │   │   │   ├── lib.rs
│   │   │   │   ├── pacing.rs
│   │   │   │   ├── policy.rs
│   │   │   │   ├── progressor.rs
│   │   │   │   ├── schedule.rs
│   │   │   │   └── shell.rs
│   │   │   ├── tests
│   │   │   │   ├── script_support
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── temporal_support
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── acceptance.rs
│   │   │   │   ├── acquisition.rs
│   │   │   │   ├── borrowed_resource.rs
│   │   │   │   ├── frame_loop.rs
│   │   │   │   ├── level_loaded.rs
│   │   │   │   ├── lifecycle.rs
│   │   │   │   ├── pacing.rs
│   │   │   │   ├── script_loaded.rs
│   │   │   │   ├── script_suspend.rs
│   │   │   │   ├── stop_before_cut.rs
│   │   │   │   ├── temporal_drivers.rs
│   │   │   │   ├── temporal_equivalence.rs
│   │   │   │   ├── temporal_progress.rs
│   │   │   │   └── temporal_structure.rs
│   │   │   ├── Cargo.toml
│   │   │   └── README.md
│   │   └── verbum
│   │       ├── src
│   │       │   └── lib.rs
│   │       ├── Cargo.toml
│   │       └── README.md
│   ├── extensions
│   │   ├── collisions
│   │   │   └── README.md
│   │   ├── lpc
│   │   │   ├── drafts
│   │   │   │   ├── LPC_YAML
│   │   │   │   │   ├── action_resolution.yaml
│   │   │   │   │   ├── base_character.contract.yaml
│   │   │   │   │   ├── bow_recurve.contract.yaml
│   │   │   │   │   ├── build_manifest.yaml
│   │   │   │   │   ├── canonical_animations.yaml
│   │   │   │   │   ├── gnarled_staff.contract.yaml
│   │   │   │   │   ├── hammer.contract.yaml
│   │   │   │   │   ├── longsword.contract.yaml
│   │   │   │   │   └── trident.contract.yaml
│   │   │   │   ├── LPC-Paradigm.md
│   │   │   │   ├── bow_recurve.md
│   │   │   │   └── grid_64.svg
│   │   │   └── README.md
│   │   └── script
│   │       ├── src
│   │       │   ├── emissions.rs
│   │       │   ├── lib.rs
│   │       │   ├── state.rs
│   │       │   ├── step.rs
│   │       │   └── table.rs
│   │       ├── Cargo.toml
│   │       └── README.md
│   └── games
│       └── demo
│           └── README.md
├── scripts
│   └── tree.sh
├── Cargo.lock
├── Cargo.toml
└── README.md

37 directories, 128 files
```

## Documentation du projet

```text
/verbum/docs
├── architecture
│   ├── adr
│   │   ├── ADR-000-language-and-material-invariants.md
│   │   ├── ADR-001-sparse-set.md
│   │   ├── ADR-002-query-model.md
│   │   ├── ADR-003-system-contract.md
│   │   ├── ADR-004-execution-model.md
│   │   ├── ADR-005-resources.md
│   │   ├── ADR-006-command-buffer.md
│   │   ├── ADR-A01-assets.md
│   │   ├── ADR-AU01-audio.md
│   │   ├── ADR-N01-network.md
│   │   ├── ADR-R01-rendering-data-model.md
│   │   ├── ADR-R02-production-pipeline.md
│   │   ├── ADR-R03-backend.md
│   │   ├── ADR-S01-shell-responsibilities.md
│   │   ├── ADR-S02-loop-pattern-and-shell-control.md
│   │   ├── ADR-S03-event-acquisition.md
│   │   ├── ADR-S04-shell-s-temporal-policy.md
│   │   ├── ADR-SV01-backup.md
│   │   ├── ADR-Sc01-scripts.md
│   │   └── ADR-W01-repartition-workspace.md
│   ├── concepts
│   │   ├── 00-principes.md
│   │   ├── 01-runtime-ecs.md
│   │   └── 02-concepts.md
│   ├── README.md
│   └── ROADMAP.md
├── generate
│   └── tree.md
├── handoffs
│   ├── 00-verbum-ecs-handoff.md
│   ├── 01-verbum-scheduler-handoff.md
│   ├── 02-verbum-forge-handoff.md
│   ├── 03-verbum-shell-implementation-handoff.md
│   ├── 03-verbum-shell-temporal-policy-T6-handoff.md
│   ├── 03-verbum-shell-temporal-policy-implementation-handoff.md
│   ├── 04-verbum-renderer-implementation-handoff.md
│   ├── 05-verbum-assets-implementation-handoff.md
│   ├── 06-verbum-scripts-implementation-handoff-u3.md
│   ├── 06-verbum-scripts-implementation-handoff-u4.md
│   ├── 06-verbum-scripts-implementation-handoff-u5.md
│   └── 06-verbum-scripts-implementation-handoff.md
├── implementation
│   ├── 00-verbum-ecs.md
│   ├── 01-verbum-scheduler-execution.md
│   ├── 01-verbum-scheduler.md
│   ├── 02-verbum-forge-execution.md
│   ├── 02-verbum-forge.md
│   ├── 03-verbum-shell-execution-ADR-S04-T5-section.md
│   ├── 03-verbum-shell-execution.md
│   ├── 03-verbum-shell-temporal-policy-remaining-work.md
│   ├── 03-verbum-shell-temporal-policy-roadmap.md
│   ├── 03-verbum-shell-temporal-policy-vman-v2-annex-v2c.md
│   ├── 03-verbum-shell-temporal-policy-vman-v2-validation.md
│   ├── 03-verbum-shell-temporal-policy-vman-v2.md
│   ├── 03-verbum-shell-temporal-policy.md
│   ├── 03-verbum-shell.md
│   ├── 04-verbum-renderer-execution.md
│   ├── 04-verbum-renderer.md
│   ├── 05-verbum-assets-execution.md
│   ├── 05-verbum-assets.md
│   ├── 06-verbum-scripts-execution.md
│   ├── 06-verbum-scripts-report-u3.md
│   ├── 06-verbum-scripts.md
│   └── 07-verbum-demo-0.md
├── notes
│   ├── adr-w01-workspace-report.md
│   ├── page-axes-shell-boucle-presentation.md
│   ├── page-axes-shell-s-temporal-policy.md
│   └── technology-selection-note.md
└── reflexivity
    ├── filiation.md
    ├── methodological-lineage-from-marius-to-verbum.md
    ├── scribe-conservateur.md
    ├── when-context-leaves-the-conversation-to-enter-the-corpus.md
    └── why-the-name-verbum.md

9 directories, 69 files
```

