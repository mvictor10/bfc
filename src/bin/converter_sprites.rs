use image::io::Reader as ImageReader;
use image::ImageFormat;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
enum Resultado {
    Convertida,
    JaEhPng,
    DestinoExistente,
    NaoEhImagem,
}

#[derive(Default)]
struct Estatisticas {
    personagens: usize,
    arquivos_analisados: usize,
    convertidos: usize,
    png_validos: usize,
    destinos_existentes: usize,
    ignorados: usize,
    erros: usize,
}

fn extensao_eh_png(caminho: &Path) -> bool {
    caminho
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("png"))
        .unwrap_or(false)
}

fn nome_arquivo(caminho: &Path) -> String {
    caminho
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

// Cria um arquivo temporário exclusivo no mesmo diretório do destino.
// Mantê-lo no mesmo volume permite finalizar a gravação por renomeação.
fn gravar_temporario(destino: &Path, dados: &[u8]) -> io::Result<PathBuf> {
    let diretorio = destino.parent().unwrap_or_else(|| Path::new("."));

    for _ in 0..100 {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nome = format!(
            ".bfc_sprite_{}_{}.tmp",
            std::process::id(),
            id
        );
        let temporario = diretorio.join(nome);

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporario)
        {
            Ok(mut arquivo) => {
                if let Err(erro) = arquivo.write_all(dados) {
                    let _ = fs::remove_file(&temporario);
                    return Err(erro);
                }

                if let Err(erro) = arquivo.flush() {
                    let _ = fs::remove_file(&temporario);
                    return Err(erro);
                }

                if let Err(erro) = arquivo.sync_all() {
                    let _ = fs::remove_file(&temporario);
                    return Err(erro);
                }

                return Ok(temporario);
            }
            Err(erro) if erro.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(erro) => return Err(erro),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "não foi possível criar um arquivo temporário exclusivo",
    ))
}

// Substitui um falso PNG preservando o arquivo original caso a troca falhe.
// O backup temporário também fica no mesmo diretório para evitar mudanças de volume.
fn substituir_arquivo(origem: &Path, temporario: &Path) -> io::Result<()> {
    let diretorio = origem.parent().unwrap_or_else(|| Path::new("."));
    let backup = loop {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidato = diretorio.join(format!(
            ".bfc_sprite_{}_{}.bak",
            std::process::id(),
            id
        ));

        if !candidato.exists() {
            break candidato;
        }
    };

    fs::rename(origem, &backup)?;

    match fs::rename(temporario, origem) {
        Ok(()) => {
            // A conversão foi concluída; o backup já não é necessário.
            let _ = fs::remove_file(backup);
            Ok(())
        }
        Err(erro_troca) => {
            // Tenta restaurar o original se a instalação do PNG falhar.
            match fs::rename(&backup, origem) {
                Ok(()) => Err(erro_troca),
                Err(erro_restauracao) => Err(io::Error::new(
                    erro_restauracao.kind(),
                    format!(
                        "falha ao instalar o PNG ({}) e ao restaurar o original ({}). Backup: {}",
                        erro_troca,
                        erro_restauracao,
                        backup.display()
                    ),
                )),
            }
        }
    }
}

fn converter_arquivo(caminho: &Path) -> Result<Resultado, Box<dyn Error>> {
    // A leitura tenta reconhecer o formato real pelos bytes, sem confiar na extensão.
    let leitor = ImageReader::open(caminho)?.with_guessed_format()?;
    let formato_detectado = leitor.format();

    // Se a extensão é PNG e o conteúdo também é PNG, valida e não altera o arquivo.
    // A decodificação abaixo é importante: apenas conferir a assinatura não basta.
    let imagem = match leitor.decode() {
        Ok(imagem) => imagem,
        Err(_) => return Ok(Resultado::NaoEhImagem),
    };

    if extensao_eh_png(caminho) && formato_detectado == Some(ImageFormat::Png) {
        return Ok(Resultado::JaEhPng);
    }

    // Todos os formatos de imagem reconhecidos são codificados como PNG.
    let mut cursor = Cursor::new(Vec::new());
    imagem.write_to(&mut cursor, ImageFormat::Png)?;
    let dados_png = cursor.into_inner();

    if extensao_eh_png(caminho) {
        // Ex.: arquivo chamado idle_01.png cujo conteúdo real era JPEG ou BMP.
        // Corrige o conteúdo no mesmo caminho e mantém o nome do sprite.
        let temporario = gravar_temporario(caminho, &dados_png)?;

        if let Err(erro) = substituir_arquivo(caminho, &temporario) {
            let _ = fs::remove_file(&temporario);
            return Err(Box::new(erro));
        }
    } else {
        // Ex.: idle_01.jpg passa a gerar idle_01.png; o original é preservado.
        let destino = caminho.with_extension("png");

        if destino.exists() {
            return Ok(Resultado::DestinoExistente);
        }

        let temporario = gravar_temporario(&destino, &dados_png)?;

        // A renomeação falha se outro arquivo criar o destino durante a conversão;
        // assim não sobrescrevemos um PNG que já exista.
        if let Err(erro) = fs::rename(&temporario, &destino) {
            let _ = fs::remove_file(&temporario);

            if destino.exists() {
                return Ok(Resultado::DestinoExistente);
            }

            return Err(Box::new(erro));
        }
    }

    Ok(Resultado::Convertida)
}

fn localizar_raiz_projeto() -> Result<PathBuf, Box<dyn Error>> {
    let executavel = std::env::current_exe()?;
    let pasta_executavel = executavel.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "não foi possível localizar a pasta do executável",
        )
    })?;

    // Quando copiado para <projeto>/tools/converter_sprites.exe.
    if pasta_executavel
        .file_name()
        .and_then(|nome| nome.to_str())
        .map(|nome| nome.eq_ignore_ascii_case("tools"))
        .unwrap_or(false)
    {
        if let Some(raiz) = pasta_executavel.parent() {
            return Ok(raiz.to_path_buf());
        }
    }

    // Quando executado diretamente pelo Cargo em target/debug ou target/release.
    let pasta_build = pasta_executavel
        .file_name()
        .and_then(|nome| nome.to_str())
        .unwrap_or_default();

    if pasta_build.eq_ignore_ascii_case("debug")
        || pasta_build.eq_ignore_ascii_case("release")
    {
        if let Some(target) = pasta_executavel.parent() {
            if let Some(raiz) = target.parent() {
                return Ok(raiz.to_path_buf());
            }
        }
    }

    // Fallback para o diretório do projeto registrado durante a compilação.
    let manifesto = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if manifesto.join("chars").is_dir() {
        return Ok(manifesto);
    }

    // Último fallback: diretório atual do terminal.
    Ok(std::env::current_dir()?)
}

fn processar_personagem(
    pasta_personagem: &Path,
    estatisticas: &mut Estatisticas,
) -> Result<(), Box<dyn Error>> {
    let mut arquivos = Vec::new();

    for entrada in fs::read_dir(pasta_personagem)? {
        let entrada = entrada?;
        let tipo = entrada.file_type()?;

        if !tipo.is_file() {
            continue;
        }

        let caminho = entrada.path();

        // character.json e outros metadados não são imagens.
        if caminho
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("json"))
            .unwrap_or(false)
        {
            continue;
        }

        arquivos.push(caminho);
    }

    // Processa primeiro os arquivos .png. Assim, um falso PNG é corrigido
    // antes de avaliar um eventual .jpg com o mesmo nome-base.
    arquivos.sort_by_key(|caminho| if extensao_eh_png(caminho) { 0 } else { 1 });

    for arquivo in arquivos {
        estatisticas.arquivos_analisados += 1;

        match converter_arquivo(&arquivo) {
            Ok(Resultado::Convertida) => {
                estatisticas.convertidos += 1;
                println!("[CONVERTIDO] {}", arquivo.display());
            }
            Ok(Resultado::JaEhPng) => {
                estatisticas.png_validos += 1;
                println!("[PNG VÁLIDO] {}", arquivo.display());
            }
            Ok(Resultado::DestinoExistente) => {
                estatisticas.destinos_existentes += 1;
                println!("[JÁ EXISTE] {}.png", arquivo.with_extension("").display());
            }
            Ok(Resultado::NaoEhImagem) => {
                estatisticas.ignorados += 1;
                println!("[IGNORADO] Não é uma imagem compatível: {}", arquivo.display());
            }
            Err(erro) => {
                estatisticas.erros += 1;
                eprintln!("[ERRO] {}: {}", arquivo.display(), erro);
            }
        }
    }

    Ok(())
}

fn executar() -> Result<(), Box<dyn Error>> {
    let raiz = localizar_raiz_projeto()?;
    let pasta_chars = raiz.join("chars");

    if !pasta_chars.is_dir() {
        return Err(format!(
            "pasta 'chars' não encontrada em: {}",
            pasta_chars.display()
        )
        .into());
    }

    let mut personagens = Vec::new();

    for entrada in fs::read_dir(&pasta_chars)? {
        let entrada = entrada?;

        if entrada.file_type()?.is_dir() {
            personagens.push(entrada.path());
        }
    }

    personagens.sort();

    let mut estatisticas = Estatisticas::default();

    println!("====================================================");
    println!("          BRAZIL FIGHT COMBAT - SPRITE PNG");
    println!("====================================================");
    println!("Projeto : {}", raiz.display());
    println!("Sprites : {}", pasta_chars.display());
    println!("----------------------------------------------------");

    for pasta_personagem in personagens {
        estatisticas.personagens += 1;

        println!("\n[PERSONAGEM] {}", nome_arquivo(&pasta_personagem));

        if let Err(erro) = processar_personagem(&pasta_personagem, &mut estatisticas) {
            estatisticas.erros += 1;
            eprintln!(
                "[ERRO] Pasta {}: {}",
                pasta_personagem.display(),
                erro
            );
        }
    }

    println!("\n====================================================");
    println!("                    RESUMO");
    println!("====================================================");
    println!("Personagens analisados : {}", estatisticas.personagens);
    println!("Arquivos analisados    : {}", estatisticas.arquivos_analisados);
    println!("Convertidos para PNG   : {}", estatisticas.convertidos);
    println!("PNGs já válidos        : {}", estatisticas.png_validos);
    println!("Destino já existente   : {}", estatisticas.destinos_existentes);
    println!("Ignorados              : {}", estatisticas.ignorados);
    println!("Erros                  : {}", estatisticas.erros);
    println!("====================================================");
    println!("Processamento concluído.");

    Ok(())
}

fn main() {
    if let Err(erro) = executar() {
        eprintln!("\n[FALHA] {}", erro);
        std::process::exit(1);
    }

    // Mantém a janela aberta quando o usuário executar o .exe por duplo clique.
    if cfg!(windows) {
        println!("\nPressione ENTER para fechar...");
        let mut entrada = String::new();
        let _ = io::stdin().read_line(&mut entrada);
    }
}
