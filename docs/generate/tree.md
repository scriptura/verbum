# Structure du Projet

**Généré le:** 2026-10-26 21:57:44

## Racine du Workspace

```text
/verbum
├── crates
│   ├── ecs
│   │   ├── src
│   │   │   ├── command.rs
│   │   │   ├── component_set.rs
│   │   │   ├── entity.rs
│   │   │   ├── fixtures.rs
│   │   │   ├── lib.rs
│   │   │   ├── query.rs
│   │   │   ├── resource.rs
│   │   │   ├── storage.rs
│   │   │   ├── values.rs
│   │   │   └── world.rs
│   │   ├── tests
│   │   │   ├── acceptance.rs
│   │   │   ├── command_buffer.rs
│   │   │   ├── components.rs
│   │   │   ├── query.rs
│   │   │   ├── resources.rs
│   │   │   └── world_identity.rs
│   │   └── Cargo.toml
│   ├── forge
│   │   ├── src
│   │   │   ├── batch.rs
│   │   │   ├── id.rs
│   │   │   ├── lib.rs
│   │   │   └── sprite.rs
│   │   ├── tests
│   │   │   └── pipeline.rs
│   │   └── Cargo.toml
│   ├── schedule
│   │   ├── examples
│   │   │   └── headless.rs
│   │   ├── src
│   │   │   ├── access.rs
│   │   │   ├── contract.rs
│   │   │   ├── data.rs
│   │   │   ├── lib.rs
│   │   │   ├── model.rs
│   │   │   ├── order.rs
│   │   │   ├── scheduler.rs
│   │   │   └── system.rs
│   │   ├── tests
│   │   │   ├── support
│   │   │   │   └── mod.rs
│   │   │   ├── acceptance.rs
│   │   │   ├── access.rs
│   │   │   └── tick.rs
│   │   └── Cargo.toml
│   └── verbum
│       ├── src
│       │   └── lib.rs
│       └── Cargo.toml
├── scripts
│   └── tree.sh
├── Cargo.lock
├── Cargo.toml
└── README.md

16 directories, 43 files
```

## Documentation du projet

```text
/verbum/docs
├── architecture
│   ├── adr
│   │   ├── ADR-000-language-and-material-invariants.md
│   │   ├── ADR-001-sparse-set.md
│   │   ├── ADR-002-query-model.md
│   │   ├── ADR-003-contrat-systeme.md
│   │   ├── ADR-004-modele-execution.md
│   │   ├── ADR-005-ressources.md
│   │   ├── ADR-006-command-buffer.md
│   │   ├── ADR-A01-assets.md
│   │   ├── ADR-AU01-audio.md
│   │   ├── ADR-N01-reseau.md
│   │   ├── ADR-R01-modele-donnee-rendu.md
│   │   ├── ADR-R02-pipeline-production.md
│   │   ├── ADR-R03-backend.md
│   │   ├── ADR-S01-shell-responsabilites.md
│   │   ├── ADR-S03-acquisition-evenements.md
│   │   ├── ADR-SV01-sauvegarde.md
│   │   └── ADR-Sc01-scripts.md
│   ├── concepts
│   │   ├── 00-principes.md
│   │   ├── 01-runtime-ecs.md
│   │   └── 02-concepts.md
│   ├── README.md
│   └── ROADMAP.md
├── drafts
│   ├── LPC_YAML
│   │   ├── action_resolution.yaml
│   │   ├── base_character.contract.yaml
│   │   ├── bow_recurve.contract.yaml
│   │   ├── build_manifest.yaml
│   │   ├── canonical_animations.yaml
│   │   ├── gnarled_staff.contract.yaml
│   │   ├── hammer.contract.yaml
│   │   ├── longsword.contract.yaml
│   │   └── trident.contract.yaml
│   ├── LPC-Paradigm.md
│   ├── bow_recurve.md
│   └── grid_64.svg
├── generate
│   └── tree.md
├── handoffs
│   ├── 03-verbum-shell-implementation-handoff.md
│   ├── HANDOFF-stage-0.md
│   ├── HANDOFF-stage-1.md
│   └── HANDOFF-stage-2-forge.md
├── implementation
│   ├── 00-verbum-ecs.md
│   ├── 01-verbum-scheduler-sessions.md
│   ├── 01-verbum-scheduler.md
│   ├── 02-verbum-forge-execution.md
│   ├── 02-verbum-forge.md
│   ├── 03-verbum-shell-execution.md
│   └── 03-verbum-shell.md
└── reflexivity
    ├── filiation.md
    ├── methodological-lineage-from-marius-to-verbum.md
    ├── scribe-conservateur.md
    ├── when-context-leaves-the-conversation-to-enter-the-corpus.md
    └── why-the-name-verbum.md

10 directories, 51 files
```

