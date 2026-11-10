#!/usr/bin/env python3
"""Organiza e normaliza os arquivos de imagem de chars/<personagem>/.

Este programa NÃO remove fundo, NÃO aplica chroma key e NÃO altera as cores.
Ele identifica o formato real pelo conteúdo do arquivo e garante que os arquivos
resultantes sejam PNGs válidos. Os originais alterados são copiados para
backup_sprite/ antes de qualquer renomeação, conversão ou exclusão.
"""

from __future__ import annotations

import argparse
import filecmp
import os
import re
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from PIL import Image, ImageOps, UnidentifiedImageError


@dataclass
class Estatisticas:
    personagens: int = 0
    arquivos_analisados: int = 0
    imagens_identificadas: int = 0
    pngs_validos: int = 0
    renomeados: int = 0
    convertidos: int = 0
    ja_corretos: int = 0
    nao_imagens: int = 0
    conflitos: int = 0
    erros: int = 0


@dataclass
class ImagemAnalisada:
    formato_real: str
    largura: int
    altura: int
    quadros: int
    imagem_rgba: Image.Image
    icc_profile: Optional[bytes]
    dpi: object
    orientacao_exif: int


def localizar_raiz_projeto(raiz_informada: Optional[Path] = None) -> Path:
    """Localiza a raiz do BFC pela pasta chars/ e, preferencialmente, Cargo.toml."""
    if raiz_informada is not None:
        raiz = raiz_informada.expanduser().resolve()
        if not (raiz / "chars").is_dir():
            raise FileNotFoundError(f"A pasta chars/ não existe em: {raiz}")
        return raiz

    script = Path(__file__).resolve()
    candidatos = list(script.parents) + [Path.cwd().resolve()]

    # Prioriza uma raiz que contenha tanto chars/ quanto Cargo.toml.
    for candidato in candidatos:
        if (candidato / "chars").is_dir() and (candidato / "Cargo.toml").is_file():
            return candidato

    # Fallback para projetos nos quais Cargo.toml esteja em outro local.
    for candidato in candidatos:
        if (candidato / "chars").is_dir():
            return candidato

    raise FileNotFoundError(
        "Não foi possível localizar a raiz do BFC. "
        "Execute o programa dentro do projeto ou informe --root CAMINHO."
    )


def analisar_imagem(caminho: Path) -> ImagemAnalisada:
    """Detecta o formato real e carrega o primeiro quadro como RGBA.

    O formato é identificado pelo conteúdo, não pela extensão do arquivo.
    Para GIFs e outros formatos animados, é usado o primeiro quadro, pois o
    formato de destino solicitado é PNG estático.
    """
    with Image.open(caminho) as original:
        formato = original.format
        if not formato:
            raise UnidentifiedImageError("Formato interno não identificado")

        quantidade_quadros = int(getattr(original, "n_frames", 1))
        original.seek(0)

        exif = original.getexif()
        orientacao = int(exif.get(274, 1)) if exif else 1
        icc_profile = original.info.get("icc_profile")
        dpi = original.info.get("dpi")

        # Corrige a orientação EXIF antes de calcular a proporção.
        orientada = ImageOps.exif_transpose(original)
        orientada.load()  # Força a decodificação completa para validar o arquivo.

        largura, altura = orientada.size
        rgba = orientada.convert("RGBA")

    return ImagemAnalisada(
        formato_real=formato.upper(),
        largura=largura,
        altura=altura,
        quadros=quantidade_quadros,
        imagem_rgba=rgba,
        icc_profile=icc_profile,
        dpi=dpi,
        orientacao_exif=orientacao,
    )


def eh_imagem_whatsapp(caminho: Path) -> bool:
    """Reconhece nomes iniciados por 'WhatsApp Image', sem diferenciar maiúsculas."""
    return caminho.name.casefold().startswith("whatsapp image ")


PREFIXOS_ANIMACAO_BFC = {
    "attack", "crouch", "dead", "defense", "idle", "jump",
    "kick", "run", "special", "walk", "hurt", "hit",
    "block", "punch", "throw", "fall", "getup", "victory",
    "win", "lose", "combo",
}


def eh_frame_de_animacao(caminho: Path) -> bool:
    """Reconhece nomes de frames para não transformar sprites quadrados em face.png.

    O projeto BFC contém frames de animação quadrados (por exemplo,
    attack_01.png e jump_01.jpg). O nome da ação é um sinal mais forte do que
    a proporção: renomeá-los para face.png quebraria a sequência de frames.
    """
    nome = caminho.stem.casefold().replace("-", "_")
    correspondencia = re.match(r"^([a-z]+)_\d{1,4}$", nome)
    return bool(correspondencia and correspondencia.group(1) in PREFIXOS_ANIMACAO_BFC)


def nome_destino(
    caminho: Path,
    largura: int,
    altura: int,
    forcar_quadrados_como_face: bool = False,
) -> str:
    """Define o nome final conforme proporção, nome WhatsApp e semântica de sprite."""
    whatsapp = eh_imagem_whatsapp(caminho)

    # WhatsApp é tratado conforme a regra especial solicitada.
    if whatsapp and largura == altura:
        return "face.png"
    if whatsapp and largura * 16 == altura * 9:
        return "sprite.png"

    # Regra quadrada para imagens genéricas. Frames de animação do BFC
    # permanecem com seu nome, mesmo quando a tela do frame é 1:1.
    if largura == altura and (forcar_quadrados_como_face or not eh_frame_de_animacao(caminho)):
        return "face.png"

    # Preserva o nome-base dos sprites e de imagens com outras proporções.
    return f"{caminho.stem}.png"


def mesmo_arquivo(origem: Path, destino: Path) -> bool:
    """Compara arquivos corretamente também no Windows (nomes/caminhos equivalentes)."""
    try:
        return os.path.samefile(origem, destino)
    except (OSError, FileNotFoundError):
        return False


def criar_backup(origem: Path, pasta_backup: Path) -> Path:
    """Copia o original para backup_sprite, sem sobrescrever outro backup."""
    pasta_backup.mkdir(parents=True, exist_ok=True)
    destino = pasta_backup / origem.name

    if destino.exists():
        # Reutiliza um backup idêntico, evitando cópias duplicadas ao executar novamente.
        try:
            if filecmp.cmp(origem, destino, shallow=False):
                return destino
        except OSError:
            pass

        contador = 1
        while True:
            candidato = pasta_backup / f"{origem.stem}__backup_{contador:03d}{origem.suffix}"
            if not candidato.exists():
                destino = candidato
                break
            contador += 1

    shutil.copy2(origem, destino)
    return destino


def caminho_temporario(pasta: Path, sufixo: str) -> Path:
    """Cria um nome temporário exclusivo na mesma pasta do arquivo final."""
    fd, nome = tempfile.mkstemp(prefix=".bfc_sprite_", suffix=sufixo, dir=pasta)
    os.close(fd)
    return Path(nome)


def salvar_png_temporario(info: ImagemAnalisada, pasta: Path) -> Path:
    """Grava e valida um PNG temporário antes de substituir qualquer arquivo."""
    temporario = caminho_temporario(pasta, ".png")
    opcoes = {}
    if info.icc_profile:
        opcoes["icc_profile"] = info.icc_profile
    if info.dpi:
        opcoes["dpi"] = info.dpi

    try:
        info.imagem_rgba.save(temporario, format="PNG", optimize=True, **opcoes)
        with Image.open(temporario) as validacao:
            if validacao.format != "PNG":
                raise OSError("O arquivo temporário não foi reconhecido como PNG")
            validacao.load()
        return temporario
    except Exception:
        temporario.unlink(missing_ok=True)
        raise


def renomear_com_seguranca(origem: Path, destino: Path) -> None:
    """Renomeia também quando a única diferença é maiúscula/minúscula no Windows."""
    if origem.name == destino.name:
        return

    intermediario = caminho_temporario(origem.parent, ".rename_tmp")
    intermediario.unlink(missing_ok=True)
    os.replace(origem, intermediario)
    try:
        os.replace(intermediario, destino)
    except Exception:
        # Tenta restaurar o nome original caso a segunda etapa falhe.
        if intermediario.exists() and not origem.exists():
            os.replace(intermediario, origem)
        raise


def processar_arquivo(
    arquivo: Path,
    estatisticas: Estatisticas,
    dry_run: bool = False,
    forcar_quadrados_como_face: bool = False,
) -> None:
    """Inspeciona e normaliza um arquivo de imagem de um personagem."""
    estatisticas.arquivos_analisados += 1

    try:
        info = analisar_imagem(arquivo)
    except (UnidentifiedImageError, OSError, ValueError) as erro:
        estatisticas.nao_imagens += 1
        print(f"  [IGNORADO] {arquivo.name} — não é uma imagem válida ({erro})")
        return

    estatisticas.imagens_identificadas += 1
    if info.quadros > 1:
        print(f"  [AVISO] {arquivo.name} possui {info.quadros} quadros; será usado somente o primeiro.")
    if info.formato_real == "PNG":
        estatisticas.pngs_validos += 1

    destino = arquivo.with_name(
        nome_destino(arquivo, info.largura, info.altura, forcar_quadrados_como_face)
    )
    destino_existe = destino.exists()
    destino_e_origem = destino_existe and mesmo_arquivo(arquivo, destino)

    # Nunca substitui outro arquivo que já ocupe o nome de destino.
    if destino_existe and not destino_e_origem:
        estatisticas.conflitos += 1
        print(
            f"  [CONFLITO] {arquivo.name} -> {destino.name}: "
            "o destino já existe. Original preservado para revisão."
        )
        return

    nome_correto = arquivo.name == destino.name
    png_real = info.formato_real == "PNG"
    precisa_regravar = (not png_real) or (info.orientacao_exif not in (0, 1))

    if png_real and arquivo.suffix == ".png" and nome_correto and not precisa_regravar:
        estatisticas.ja_corretos += 1
        print(
            f"  [OK] {arquivo.name} — PNG válido, "
            f"{info.largura}x{info.altura}; sem alteração."
        )
        return

    acao = "CONVERTER" if precisa_regravar else "RENOMEAR"
    print(
        f"  [{ 'SIMULAÇÃO' if dry_run else acao }] {arquivo.name} "
        f"({info.formato_real}, {info.largura}x{info.altura}) -> {destino.name}"
    )

    if dry_run:
        return

    pasta_backup = arquivo.parent / "backup_sprite"
    temporario: Optional[Path] = None

    try:
        # Backup sempre ocorre antes da primeira alteração no original.
        criar_backup(arquivo, pasta_backup)

        if precisa_regravar:
            temporario = salvar_png_temporario(info, arquivo.parent)
            os.replace(temporario, destino)
            temporario = None

            # Se destino e origem eram arquivos diferentes, remove a origem
            # somente depois de o PNG novo estar validado e gravado.
            if not mesmo_arquivo(arquivo, destino) and arquivo.exists():
                arquivo.unlink()
            estatisticas.convertidos += 1
        else:
            if mesmo_arquivo(arquivo, destino):
                # Normaliza extensão .PNG para .png em sistemas case-insensitive.
                renomear_com_seguranca(arquivo, destino)
            else:
                os.replace(arquivo, destino)
            estatisticas.renomeados += 1

        print(f"  [CONCLUÍDO] Backup: backup_sprite/{arquivo.name}")

    except Exception:
        if temporario is not None:
            temporario.unlink(missing_ok=True)
        raise


def ordenar_arquivos(arquivos: list[Path]) -> list[Path]:
    """Prioriza nomes canônicos existentes e depois PNGs antes de outros formatos."""
    def prioridade(caminho: Path) -> tuple[int, str]:
        nome = caminho.name.casefold()
        if nome in {"face.png", "sprite.png"}:
            return (0, nome)
        if caminho.suffix.casefold() == ".png":
            return (1, nome)
        return (2, nome)

    return sorted(arquivos, key=prioridade)


def processar_personagem(
    pasta_personagem: Path,
    estatisticas: Estatisticas,
    dry_run: bool = False,
    forcar_quadrados_como_face: bool = False,
) -> None:
    """Processa somente os arquivos diretamente dentro da pasta do personagem."""
    arquivos = [
        item for item in pasta_personagem.iterdir()
        if item.is_file() and item.name.casefold() != "desktop.ini"
    ]

    print(f"\n[PERSONAGEM] {pasta_personagem.name}")
    if not arquivos:
        print("  [INFO] Nenhum arquivo encontrado.")
        return

    for arquivo in ordenar_arquivos(arquivos):
        try:
            processar_arquivo(
                arquivo, estatisticas, dry_run=dry_run,
                forcar_quadrados_como_face=forcar_quadrados_como_face,
            )
        except Exception as erro:
            estatisticas.erros += 1
            print(f"  [ERRO] {arquivo.name}: {erro}")


def executar() -> int:
    parser = argparse.ArgumentParser(
        description="Normaliza os nomes e formatos das imagens de chars/<personagem>/ do BFC."
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=None,
        help="Caminho da raiz do projeto BFC. Se omitido, o programa tenta localizá-la.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Mostra as alterações planejadas sem modificar nenhum arquivo.",
    )
    parser.add_argument(
        "--force-square-to-face",
        action="store_true",
        help=(
            "Aplica literalmente 1:1 -> face.png também aos frames de animação. "
            "Pode gerar conflitos e não é recomendado para o BFC atual."
        ),
    )
    args = parser.parse_args()

    raiz = localizar_raiz_projeto(args.root)
    pasta_chars = raiz / "chars"
    personagens = sorted(
        pasta for pasta in pasta_chars.iterdir()
        if pasta.is_dir() and pasta.name.casefold() != "backup_sprite"
    )

    if not personagens:
        print(f"[AVISO] Nenhuma pasta de personagem encontrada em: {pasta_chars}")
        return 0

    estatisticas = Estatisticas()

    print("============================================================")
    print("       BRAZIL FIGHT COMBAT - SPRITE ORGANIZER (PYTHON)")
    print("============================================================")
    print(f"Projeto : {raiz}")
    print(f"Sprites : {pasta_chars}")
    print("Modo    : SIMULAÇÃO (nenhum arquivo será alterado)" if args.dry_run else "Modo    : PROCESSAMENTO")
    print("Fundo   : NÃO será removido nem modificado")
    print("Frames  : nomes de animação conhecidos serão preservados, mesmo em 1:1")
    print("------------------------------------------------------------")

    for pasta_personagem in personagens:
        estatisticas.personagens += 1
        processar_personagem(
            pasta_personagem, estatisticas, dry_run=args.dry_run,
            forcar_quadrados_como_face=args.force_square_to_face,
        )

    print("\n============================================================")
    print("                         RESUMO")
    print("============================================================")
    print(f"Personagens analisados : {estatisticas.personagens}")
    print(f"Arquivos analisados    : {estatisticas.arquivos_analisados}")
    print(f"Imagens identificadas : {estatisticas.imagens_identificadas}")
    print(f"PNG válidos detectados : {estatisticas.pngs_validos}")
    print(f"Convertidos para PNG   : {estatisticas.convertidos}")
    print(f"Renomeados             : {estatisticas.renomeados}")
    print(f"Já estavam corretos    : {estatisticas.ja_corretos}")
    print(f"Arquivos não-imagem    : {estatisticas.nao_imagens}")
    print(f"Conflitos de destino   : {estatisticas.conflitos}")
    print(f"Erros                  : {estatisticas.erros}")
    print("============================================================")

    if args.dry_run:
        print("Simulação concluída. Execute novamente sem --dry-run para aplicar.")
    else:
        print("Processamento concluído. Originais alterados estão em backup_sprite/.")

    return 0 if estatisticas.erros == 0 else 1


def main() -> None:
    try:
        codigo = executar()
    except Exception as erro:
        print(f"\n[FALHA GLOBAL] {erro}")
        codigo = 1

    raise SystemExit(codigo)


if __name__ == "__main__":
    main()
