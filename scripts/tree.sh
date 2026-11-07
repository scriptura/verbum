#!/bin/bash
export LC_ALL=C.UTF-8
export LANG=C.UTF-8
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_NAME="$(basename "$PROJECT_ROOT")"
DOCS_DIR="$PROJECT_ROOT/docs"
GEN_DIR="$DOCS_DIR/generate"
OUTPUT="$GEN_DIR/tree.md"
TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")

mkdir -p "$GEN_DIR"

cat > "$OUTPUT" << EOF
# Structure du Projet

**Généré le:** $TIMESTAMP

## Racine du Workspace

\`\`\`text
EOF

declare -A PRUNE_NAMES=(
    [crates]="target build artifacts assets"
    [scripts]=""   # rien à pruner
)

(
    cd "$PROJECT_ROOT"
    {
        find . -maxdepth 1 -type f -print
        for d in "${!PRUNE_NAMES[@]}"; do
            [ -d "$d" ] || continue
            names=${PRUNE_NAMES[$d]}
            if [ -z "$names" ]; then
                find "$d" -print 2>/dev/null
                continue
            fi
            prune_expr=()
            for n in $names; do prune_expr+=( -type d -name "$n" -o ); done
            unset 'prune_expr[${#prune_expr[@]}-1]'
            find "$d" \
                \( "${prune_expr[@]}" \) \
                    ! -exec test -f '{}/Cargo.toml' \; \
                    -prune -o \
                -print 2>/dev/null
        done
    } | tree --fromfile --dirsfirst
) | sed "1s|.*|/$PROJECT_NAME|" >> "$OUTPUT"

cat >> "$OUTPUT" << EOF
\`\`\`

## Documentation du projet

\`\`\`text
EOF

if [ -d "$DOCS_DIR" ]; then
    tree "$DOCS_DIR" --dirsfirst | sed "1s|.*|/$PROJECT_NAME/docs|" >> "$OUTPUT"
else
    echo "⚠️ Dossier $DOCS_DIR introuvable." >> "$OUTPUT"
fi

cat >> "$OUTPUT" << EOF
\`\`\`

EOF

echo -e "\033[0;32m✓\033[0m Arborescence générée : docs/generate/tree.md"
