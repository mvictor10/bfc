# Verificação estática final — BFC Rust Studio

Data: 21 de outubro de 2026.

## Recursos verificados

| Verificação | Resultado |
|---|---:|
| Personagens registrados em `chars/chars.json` | 21 |
| Configurações `character.json` válidas dos personagens registrados | 21/21 |
| Cenários registrados em `data/stages.ini` | 26 |
| Trilhas registradas em `data/music_library.json` | 6 |
| Arquivos PNG que correspondem aos padrões de quadros nas pastas registradas | 71 |
| Arquivos JSON analisados em todo o projeto | 29 válidos, 0 inválidos |
| Imagens verificadas por decodificação | 260, 0 erros |
| Referências ausentes para sprites/faces registrados, imagens de cenários ou catálogo de músicas | 0 |

## Materiais adicionais

`chars/_modelo/` e `chars/_exemplo/` são mantidos como referência. `chars/agente-lya/`, `chars/prof-jeffy/` e `chars/robsom/` contêm fotografias JPG e JSON base, mas não constam em `chars/chars.json`; são materiais de origem não registrados, prontos para serem importados/configurados pelo BFC Studio.

## Validação do código e executáveis

O projeto inclui testes unitários em Rust e scripts `build.bat`, `test.bat` e `tools/build_studio.bat`. Não foi possível executar `cargo build --release`, `cargo test`, `cargo fmt` ou `cargo clippy` neste ambiente, pois `rustc` e `cargo` não estão instalados. Portanto, esta auditoria cobre arquivos, JSON, imagens e referências de assets; não comprova a compilação, linkagem Win32 ou o funcionamento interativo no Windows. O ZIP não inclui executáveis `.exe` fabricados ou de origem desconhecida. Ao compilar no Windows, `build.bat` gera `target\release\BrazilFightCombat.exe` e copia `target\release\BFCStudio.exe` para `tools\BFCStudio.exe`.
