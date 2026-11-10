# Correções de compilação — 21/10/2026

Esta revisão corrige os erros reportados na compilação do port Rust:

- `src/render.rs`: importa `MoveKind` junto das demais definições de `fighter`.
- `src/render.rs`: passa as coordenadas de mundo do lutador (`f.x` e `f.feet_y`) explicitamente para `draw_rig_part`; o método passa a usar `fighter_x` e `fighter_feet_y`, removendo referências fora do escopo.
- `src/combat.rs`: remove import não utilizado de `Rect`.
- `src/game.rs`: remove parênteses redundantes no fallback de seleção de cenário.
- `src/config.rs`: remove a variável local `art`, que era lida mas não utilizada.

A ferramenta `cargo`/`rustc` não está disponível neste ambiente de edição. Por isso, as alterações foram conferidas estaticamente, mas a compilação completa ainda precisa ser executada no Windows com `cargo build --release` e `cargo test`.
