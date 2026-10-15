# Structure du Projet

**Généré le:** 2026-10-15 11:56:47

## Racine du Workspace

```text
/verbum
├── crates
│   ├── verbum
│   │   ├── src
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── verbum-ecs
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
│   └── verbum-schedule
│       ├── src
│       │   ├── access.rs
│       │   ├── contract.rs
│       │   ├── data.rs
│       │   ├── lib.rs
│       │   ├── model.rs
│       │   └── order.rs
│       ├── tests
│       │   ├── support
│       │   │   └── mod.rs
│       │   └── access.rs
│       └── Cargo.toml
├── scripts
│   └── tree.sh
├── Cargo.lock
├── Cargo.toml
└── README.md

12 directories, 32 files
```

## Documentation du projet

```text
/docs
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
│   ├── HANDOFF-Stage-0.md
│   └── HANDOFF-Stage-1.md
├── implementation
│   ├── 00-verbum-ecs.md
│   ├── 01-verbum-scheduler-sessions.md
│   └── 01-verbum-scheduler.md
└── reflexivity
    ├── filiation.md
    ├── methodological-lineage-from-marius-to-verbum.md
    ├── scribe-conservateur.md
    ├── when-context-leaves-the-conversation-to-enter-the-corpus.md
    └── why-the-name-verbum.md

10 directories, 44 files
```

